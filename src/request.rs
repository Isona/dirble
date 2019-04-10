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

use curl::Error;
use std::sync::Arc;
use std::time::Duration;
use crate::arg_parse::GlobalOpts;
use percent_encoding::percent_decode;
extern crate curl;
use curl::easy::{Easy2, Handler, WriteError};
use crate::content_parse;

pub struct Collector
{
    pub contents: Vec<u8>,
    pub content_len: usize
}

impl Collector {
    fn clear_buffer(&mut self) {
        self.contents = Vec::new();
        self.content_len = 0;
    }
}

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.contents.extend_from_slice(data);
        let data_len = data.len();
        self.content_len += data_len;
        Ok(data_len)
    }
}

// Struct which contains information about a response
// This is sent back to the main thread
pub struct RequestResponse {
    pub url: String,
    pub code: u32,
    pub content_len: usize,
    pub is_directory: bool,
    pub is_listable: bool,
    pub redirect_url: String,
    pub found_from_listable: bool,
    pub parent_depth: u32
}

// This function takes an instance of "Easy2", a base URL and a suffix
// It then makes the request, if the response was not a 404
// then it will return a RequestResponse struct
pub fn make_request(mut easy: &mut Easy2<Collector>, url: String) -> RequestResponse{

    // Set the url in the Easy2 instance
    easy.url(&url).unwrap();

    // Perform the request and check if it's empty
    // If it's empty then return a RequestResponse struct
    match perform(&mut easy) {
        Ok(_v) => {}
        Err(_e) => {
            let req_response = RequestResponse {
                url: url.clone(),
                code: 0,
                content_len: 0,
                is_directory:false,
                is_listable: false,
                redirect_url: String::from(""),
                found_from_listable: false,
                parent_depth: 0
            };
            return req_response; 
        }
    }

    // Get the response code
    let code = easy.response_code().unwrap();

    // Declare the RequestResponse for the current request
    let mut req_response = RequestResponse {
        url: url.clone(),
        code: code,
        content_len: 0,
        is_directory:false,
        is_listable: false,
        redirect_url: String::from(""),
        found_from_listable: false,
        parent_depth: 0
    };

    // If the response was a redirect, check if it's a directory
    // Also add the redirect url to the struct
    // Generally, directories will redirect requests to them with no trailing /
    // so that they have a trailing /
    if code == 301 || code == 302 {

        // Obtain and url decode the redirect destination
        let redir_dest = easy.redirect_url().unwrap().unwrap();
        let redir_dest = percent_decode(redir_dest.as_bytes()).decode_utf8().unwrap();

        // Clone and url decode the url
        let dir_url = url.clone() + "/";
        let dir_url = percent_decode(dir_url.as_bytes()).decode_utf8().unwrap();

        if dir_url == redir_dest {
            req_response.is_directory = true;
        }

        req_response.redirect_url = dir_url.to_string();
    }

    // Get the contents of the response and set the length in the struct
    let contents = easy.get_ref();
    req_response.content_len = contents.content_len;

    req_response
}

pub fn listable_check(easy: &mut Easy2<Collector>, original_url: String, disable_recursion: bool, scrape_listable: bool) -> Vec<RequestResponse> {
    // Formulate the directory name and make a request to get the contents of the page
    let mut dir_url = String::from(original_url.clone());
    if !dir_url.ends_with("/") {
        dir_url = dir_url + "/";
    }
    let mut response = make_request(easy, dir_url.clone());
    let content = get_content(easy).to_lowercase();
    let mut output_list:Vec<RequestResponse> = Vec::new();

    match response.code {
        // If a found response was returned then check if the directory is listable or not
        200 => {
            let listable = content.contains("parent directory") || content.contains("up to ") 
                || content.contains("directory listing for");

            if listable{
                response.is_listable = true;
                response.is_directory = true;
                output_list.push(response);
            }
            else{
                response.is_listable = false;
                response.is_directory = true;
                
                output_list.push(response);
                return output_list
            }
        }
        // If the code returned was not a 200 then create a struct
        // indicating that this is a folder, then return it
        _ => {
            output_list.push(fabricate_request_response(
                original_url, true, false));
            return output_list
        }
    }

    // If scraping of listables is disabled then just return from the function
    if !scrape_listable { return output_list }

    // Get urls scraped from the response
    let scraped_urls:Vec<String> = content_parse::scrape_urls(content, dir_url);

    for scraped_url in scraped_urls {
        // If the scraped url doesn't end in a /, it's unlikely to be a folder
        // Add it to the list of found URLs to be returned
        if !scraped_url.ends_with("/") {
            output_list.push(fabricate_request_response(
                scraped_url, false, false));
        }
        // If the url ends in a /, it is likely to be a folder
        else {
            // If recursion is enabled then call this function on the discovered folder
            // Append the discovered items to the current output
            if !disable_recursion {
                output_list.append(&mut listable_check(easy, scraped_url, disable_recursion, scrape_listable));
            }
            // If recursion is disabled then just add the url to the values to be returned
            else {
                output_list.push(fabricate_request_response(scraped_url, true, false));
            }
        }
    }

    output_list
}

// Creates an easy2 instance based on the parameters provided by the user
pub fn generate_easy(global_opts: Arc<GlobalOpts>) -> Easy2<Collector>
{
    // Create a new curl Easy2 instance and set it to use GET requests
    let mut easy = Easy2::new(Collector{contents: Vec::new(), content_len: 0});
    easy.get(true).unwrap();

    // Set the timeout of the easy
    easy.timeout(Duration::from_secs(global_opts.timeout as u64)).unwrap();

    // Use proxy settings if they have been provided
    if global_opts.proxy_enabled {
        easy.proxy(&global_opts.proxy_address).unwrap();
    }

    // If the ignore cert flag is enabled, ignore cert validity
    if global_opts.ignore_cert {
        easy.ssl_verify_host(false).unwrap();
        easy.ssl_verify_peer(false).unwrap();
    }

    // Set the user agent
    match &global_opts.user_agent {
        Some(user_agent) => { easy.useragent(&user_agent.clone()).unwrap(); },
        None => {}
    }

    // Set http basic auth options
    match &global_opts.username {
        Some(username) => {
            easy.username(&username.clone()).unwrap();
            easy.password(&global_opts.password.clone().unwrap()).unwrap();
        },
        None => {}
    }

    // Set cookies
    match &global_opts.cookies {
        Some(cookies) => {
            easy.cookie(cookies).unwrap();
        },
        None => {}
    }

    // Set headers
    match &global_opts.headers {
        Some(headers) => {
            let mut header_list = curl::easy::List::new();
            for header in headers {
                header_list.append(header).unwrap();
            }
            easy.http_headers(header_list).unwrap();
        },
        None => {}
    }

    easy
}

// Before each request, the buffer should be cleared
// This provides support for chunked http responses
fn perform(easy: &mut Easy2<Collector>) -> Result<(), Error>
{
    easy.get_mut().clear_buffer();
    easy.perform()
}

// Get the current content of the given easy and return it as a string
fn get_content(easy: &mut Easy2<Collector>) -> String
{
    let contents = easy.get_ref();
    String::from_utf8_lossy(&contents.contents).to_string()
}

// Generate a struct for a response for use when a request hasn't been made
// Used when items were discovered via scraping
fn fabricate_request_response(url: String, is_directory: bool, is_listable: bool) -> RequestResponse
{
    let mut new_url = url.clone();
    if new_url.ends_with("/") {
        new_url.pop();
    }
    
    RequestResponse {
        url: url.clone(),
        code: 0,
        content_len: 0,
        is_directory: is_directory,
        is_listable: is_listable,
        redirect_url: String::from(""),
        found_from_listable: true,
        parent_depth: 0
    }
}