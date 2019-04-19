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
use crate::arg_parse;
use curl::easy::Easy2;
extern crate rand;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

// Struct for passing information back to the main thread
pub struct DirectoryInfo {
    pub url:String,
    pub validator:TargetValidator,
    pub parent_depth: u32
}

impl DirectoryInfo {
    pub fn new(url: String, validator: TargetValidator, parent_depth:u32) -> DirectoryInfo {
        DirectoryInfo {
            url,
            validator,
            parent_depth
        }
    }

    pub fn generate_end() -> DirectoryInfo {
        DirectoryInfo {
            url:String::from("END"),
            validator:TargetValidator::new(0, None),
            parent_depth: 0
        }
    }
}

// Struct containing information to determine if a response
// was not found for a directory
#[derive(Clone)]
pub struct TargetValidator {
    response_code:u32,
    response_len:Option<usize>
}

impl TargetValidator {
     pub fn new(response_code: u32, response_len: Option<usize>) -> TargetValidator{
        TargetValidator {
            response_code,
            response_len
        }
     }

     // Function used to compare the validator to a RequestResponse,
     // Returns true if the given request matches the not found definition
     pub fn is_not_found(&self, response: &request::RequestResponse) -> bool {
        if !(self.response_code == response.code) {
            return false
        }

        match self.response_len {
            Some(size) => size == response.content_len,
            None => true,
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

                // Generate an easy and make 3 random requests to the folder
                let mut easy = request::generate_easy(&global_opts);
                let responses = make_requests(response.url.clone(), &mut easy);

                //Get a validator
                let validator_option = determine_not_found(responses);

                // If there is a validator then wrap it in a DirectoryInfo and send to main
                if let Some(validator) = validator_option {
                    let directory_info = DirectoryInfo::new(response.url, validator, response.parent_depth);
                    main_tx.send(Some(directory_info)).unwrap();
                }
                else {
                    println!("{} errored too often during validation, skipping scanning", response.url);
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

    let url1 = format!("{}{}", base_url, rand_string(10));
    response_vector.push(request::make_request(easy, url1));


    let url2 = format!("{}{}", base_url, rand_string(20));
    response_vector.push(request::make_request(easy, url2));

    let url3 = format!("{}{}", base_url, rand_string(30));
    response_vector.push(request::make_request(easy, url3));

    response_vector

}


// Generate a target validator for a given set of 3 responses
fn determine_not_found(responses:Vec<request::RequestResponse>) -> Option<TargetValidator> {

    if responses.len() < 3 {
        TargetValidator::new(404, None);
    }

    let mut code = 404;
    if responses[0].code == responses[1].code 
        || responses[0].code == responses[2].code {
        code = responses[0].code;
    }
    else if responses[1].code == responses[2].code {
        code = responses[1].code;
    }

    if code == 0 {
        return None
    }
    else if code == 404 {
        return Some(TargetValidator::new(code, None))        
    }

    let mut response_size = None;
    if responses[0].content_len == responses[1].content_len
        || responses[0].content_len == responses[2].content_len {
        response_size = Some(responses[0].content_len);
    }
    else if responses[1].content_len == responses[2].content_len {
        response_size = Some(responses[1].content_len);
    }

    Some(TargetValidator::new(code, response_size))


}

// Based on https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html
// Generates a string of alphanumeric characters of the given length
fn rand_string(length: usize) -> String {
    thread_rng().sample_iter(&Alphanumeric)
        .take(length).collect()
}