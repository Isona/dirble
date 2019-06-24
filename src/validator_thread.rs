// This file is part of Dirble - https://www.github.com/nccgroup/dirble
// Copyright (C) 2019 Izzy Whistlecroft <Izzy(dot)Whistlecroft(at)nccgroup(dot)com>
// Released as open source by NCC Group Plc - https://www.nccgroup.com/
//
// Dirble is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Dirble is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Dirble.  If not, see <https://www.gnu.org/licenses/>.

use crate::request;
use std::sync::{Arc, mpsc::self};
use std::fmt;
use crate::arg_parse;
use curl::easy::Easy2;
extern crate rand;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use log::{
    info,
    warn,
};

// Struct for passing information back to the main thread
pub struct DirectoryInfo {
    pub url:String,
    pub validator:Option<TargetValidator>,
    pub parent_depth: u32,
}

impl DirectoryInfo {
    pub fn new(url: String, validator: Option<TargetValidator>, 
                parent_depth:u32) -> DirectoryInfo {
        DirectoryInfo {
            url,
            validator,
            parent_depth
        }
    }

    // Used to inform the main thread that a request thread ended
    pub fn generate_end() -> DirectoryInfo {
        DirectoryInfo {
            url:String::from("END"),
            validator:None,
            parent_depth: 0
        }
    }
}

// Struct containing information to determine if a response
// was not found for a directory
#[derive(Clone)]
pub struct TargetValidator {
    response_code:u32,
    response_len:Option<i32>,
    diff_response_len:Option<i32>,
    redirect_url:Option<String>,
    pub validator_alert: Option<ValidatorAlert>
}

impl TargetValidator {
     pub fn new(response_code: u32, response_len: Option<i32>, 
                diff_response_len: Option<i32>, redirect_url: Option<String>,
                validator_alert: Option<ValidatorAlert>) -> TargetValidator{
        TargetValidator {
            response_code,
            response_len,
            diff_response_len,
            redirect_url,
            validator_alert
        }
     }

     // Function used to compare the validator to a RequestResponse,
     // Returns true if the given request matches the not found definition
     pub fn is_not_found(&self, response: &request::RequestResponse) -> bool {
        // If the responses codes don't match then it is "found"
        if self.response_code != response.code {
            return false
        }

        // If there's a redirect url set then check that
        if let Some(redirect_url) = &self.redirect_url {
            return redirect_url == &response.redirect_url;
        }

        // If there is a length in the validator then check against that,
        // otherwise it is "not found"
        if let Some(size) = self.response_len {
            return size == response.content_len as i32;
        }

        match self.diff_response_len {
            Some(size) => {
                let diff = (response.content_len as i32 - response.url.len() as i32).abs();
                return size == diff;
            }
            None => { return true; }
        }
     }

     // Return a string summary of this validator's definition
     // of a not found response
     pub fn summary_text(&self)  -> String {
        let mut output = format!("(CODE:{}", self.response_code);

        if let Some(redirect_url) = &self.redirect_url {
            output += &format!("|DEST:{}", redirect_url);
        }
        
        if let Some(length) = self.response_len {
            output += &format!("|SIZE:{}", length);
        }

        if let Some(length) = self.diff_response_len {
            output += &format!("|DIFF_SIZE:{}", length);
        }

        output + ")"
     }


     // Used to determine if things which may be undesirable to
     // scan should be scanned, checked against options provided by user
     // Returns true if the folder should be scanned
     pub fn scan_folder(&self, scan_opts: &arg_parse::ScanOpts) -> bool {
         if let Some(validator_alert) = &self.validator_alert {
            match validator_alert {
                ValidatorAlert::Code401 => { 
                    return scan_opts.scan_401
                }
                ValidatorAlert::Code403 => {
                    return scan_opts.scan_403
                }
                // Placeholder branch for future use
                ValidatorAlert::RedirectToHTTPS => { 
                    return true 
                }
            }
        }
        else {
            return true
        }
    }

    pub fn print_alert(&self) -> String {
        if let Some(validator_alert) = &self.validator_alert {
            format!(": {}", validator_alert)
        }
        // This branch should never happen because this function should
        // only be used if scan_folder returned false
        else {
            format!("")
        }
    }
}

#[derive(Clone)]
pub enum ValidatorAlert {
    Code401,
    Code403,
    RedirectToHTTPS
}

impl fmt::Display for ValidatorAlert {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidatorAlert::Code401 =>{
                write!(f, 
                    "\n    Scanning of directories returning 401 is disabled.\n    \
                    Use the --scan-401 flag to scan this directory,\n    \
                    or provide a valid session token or credentials.")
            },
            ValidatorAlert::Code403 => {
                write!(f, 
                    "\n    Scanning of directories returning 403 is disabled.\n    \
                    Use the --scan-403 flag to scan this directory,\n    \
                    or provide valid session token or credentials.")
            },
            // Placeholder branch
            ValidatorAlert::RedirectToHTTPS => {
                write!(f, 
                    "The directory redirected to HTTPS")
            }
        }
    }
}

pub fn validator_thread(rx: mpsc::Receiver<request::RequestResponse>, main_tx: mpsc::Sender<Option<DirectoryInfo>>,
    global_opts:Arc<arg_parse::GlobalOpts>)
{
    loop {
        // Get a RequestResponse from the receiver
        if let Ok(response) = rx.try_recv() {
            // If the main thread is trying to exit then stop
            if response.url == "END" {
                main_tx.send(Some(DirectoryInfo::generate_end())).unwrap();
                continue;
            }
            else if response.url == "MAIN ENDING" {
                break;
            }
            else {
                // Don't do anything if it's somehow not a directory
                // Also don't do anything if it's listable and we aren't scanning those
                if !response.is_directory ||
                        (response.is_listable && !global_opts.scan_listable) {
                    continue;
                }

                // If there is a max recursion depth set the check that
                if let Some(max_recursion_depth) = global_opts.max_recursion_depth {
                    // Calculate the depth
                    let mut depth = response.url.matches("/").count() as i32;

                    if response.url.ends_with("/") {
                        depth -= 1;
                    }

                    depth -= response.parent_depth as i32;

                    // If the depth exceeds the max_recursion_depth
                    // Skip scanning this directory
                    if depth > max_recursion_depth {
                        continue;
                    }
                }
                //println!("Parent depth: {}, current depth: {}", response.parent_depth, depth);

                // If validation is disabled or if whitelisting is enabled
                // return a validator of None
                // The validator is unused if whitelisting is enabled
                if global_opts.disable_validator || global_opts.whitelist {
                    let directory_info = DirectoryInfo::new(response.url, None, response.parent_depth);
                    main_tx.send(Some(directory_info)).unwrap();  
                    continue;
                }

                // Generate an easy and make 3 random requests to the folder
                let mut easy = request::generate_easy(&global_opts);
                let responses = make_requests(response.url.clone(), &mut easy);

                //Get a validator
                let validator_option = determine_not_found(responses);

                // If there is a validator then wrap it in a DirectoryInfo and send to main
                if let Some(validator) = validator_option {
                    info!("Detected nonexistent paths for {} are {}", &response.url, validator.summary_text());
                    let directory_info = DirectoryInfo::new(response.url, Some(validator), response.parent_depth);
                    main_tx.send(Some(directory_info)).unwrap();
                }
                // If there isn't a validator then send a none back to main
                // This will be ignored but is necessary during validation of initial directories
                else {
                    warn!("{} errored too often during validation, skipping scanning", response.url);
                    main_tx.send(None).unwrap();
                }
            }
        }
    }
}

// Makes a set of 3 requests to random strings of different lengths in the given folder
fn make_requests(mut base_url:String, easy: &mut Easy2<request::Collector>) -> Vec<request::RequestResponse> {
    let mut response_vector:Vec<request::RequestResponse> = Vec::new();

    if !base_url.ends_with("/")
    {
        base_url += "/";
    }

    for i in 1..=3 {
        let url = format!("{}{}", base_url, rand_string(10*i));
        response_vector.push(request::make_request(easy, url));
    }

    response_vector

}


// Generate a target validator for a given set of 3 responses
fn determine_not_found(responses:Vec<request::RequestResponse>) -> Option<TargetValidator> {

    if responses.len() < 3 {
        return Some(TargetValidator::new(404, None, None, None, None))
    }

    let mut validator_alert = None;

    let mut code = 404;
    if responses[0].code == responses[1].code 
            || responses[0].code == responses[2].code {
        code = responses[0].code;
    }
    else if responses[1].code == responses[2].code {
        code = responses[1].code;
    }

    match code {
        0 => {
            return None;
        }
        301 | 302 => {
            let mut redirect_url = None;
            if responses[0].redirect_url == responses[1].redirect_url
                    || responses[0].redirect_url == responses[2].redirect_url {
                redirect_url = Some(responses[0].redirect_url.clone());
            }
            else if responses[1].redirect_url == responses[2].redirect_url {
                redirect_url = Some(responses[1].redirect_url.clone());
            }

            return Some(TargetValidator::new(code, None, None, redirect_url, None))
        }
        401 => {
            validator_alert = Some(ValidatorAlert::Code401);
        }
        403 => {
            validator_alert = Some(ValidatorAlert::Code403);
        }
        404 => {
            return Some(TargetValidator::new(code, None, None, None, None));
        }
        _ => {}
    }

    let mut response_size = None;
    if responses[0].content_len == responses[1].content_len
            || responses[0].content_len == responses[2].content_len {
        response_size = Some(responses[0].content_len as i32);
    }
    else if responses[1].content_len == responses[2].content_len {
        response_size = Some(responses[1].content_len as i32);
    }

    let mut diff_response_size = None;
    if response_size == None {
        let diff_0 = ((responses[0].content_len as i32) - responses[0].url.len() as i32).abs();
        let diff_1 = ((responses[1].content_len as i32) - responses[1].url.len() as i32).abs();
        let diff_2 = ((responses[2].content_len as i32) - responses[2].url.len() as i32).abs();

        if diff_0 == diff_1
            || diff_0 == diff_2 {
            diff_response_size = Some(diff_0);
        }
        else if diff_1 == diff_2 {
            diff_response_size = Some(diff_1);
        }
    }

    Some(TargetValidator::new(code, response_size, diff_response_size, None, validator_alert))


}

// Based on https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html
// Generates a string of alphanumeric characters of the given length
fn rand_string(length: usize) -> String {
    thread_rng().sample_iter(&Alphanumeric)
        .take(length).collect()
}
