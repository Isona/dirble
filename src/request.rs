use curl::Error;
use std::sync::Arc;
use std::time::Duration;
use crate::arg_parse::GlobalOpts;
use percent_encoding::percent_decode;
extern crate curl;
use curl::easy::{Easy2, Handler, WriteError};
use crate::content_parse;

pub struct Collector(pub Vec<u8>);

impl Collector {
    fn clear_buffer(&mut self) {
        self.0 = Vec::new();
    }
}

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

// Struct which contains information about a response
// This is sent back to the main thread
pub struct RequestResponse {
    pub url: String,
    pub code: u32,
    pub content_len: u32,
    pub is_directory: bool,
    pub is_listable: bool,
    pub redirect_url: String,
    pub found_from_listable: bool
}

// This function takes an instance of "Easy2", a base URL and a suffix
// It then makes the request, if the response was not a 404
// then it will return a RequestResponse struct
pub fn make_request(mut easy: &mut Easy2<Collector>, url: String) -> Option<RequestResponse>{

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
                found_from_listable: false
            };
            return Some(req_response); 
        }
    }

    // Get the response code
    let code = easy.response_code().unwrap();
    
    // If the code was a 404, return none
    if code == 404 {
        return None;
    }

    // Declare the RequestResponse for the current request
    let mut req_response = RequestResponse {
        url: url.clone(),
        code: code,
        content_len: 0,
        is_directory:false,
        is_listable: false,
        redirect_url: String::from(""),
        found_from_listable: false
    };

    // If the response was a redirect, check if it's a directory
    // Also add the redirect url to the struct
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
    req_response.content_len = String::from_utf8_lossy(&contents.0).len() as u32;

    Some(req_response)
}

pub fn listable_check(easy: &mut Easy2<Collector>, original_url: String, disable_recursion: bool, scrape_listable: bool) -> Vec<RequestResponse> {
    //Make a request
    let mut dir_url = String::from(original_url.clone());
    if !dir_url.ends_with("/") {
        dir_url = dir_url + "/";
    }
    let response = make_request(easy, dir_url.clone());
    let content = get_content(easy).to_lowercase();
    let mut output_list:Vec<RequestResponse> = Vec::new();

    match response {
        Some(mut resp) => {
            let listable = content.contains("parent directory") || content.contains("up to ") 
                || content.contains("directory listing for");

            if listable{
                resp.is_listable = true;
                resp.is_directory = true;
                output_list.push(resp);
            }
            else{
                resp.is_listable = false;
                resp.is_directory = true;
                
                output_list.push(resp);
                return output_list
            }
        }
        None => {
            output_list.push(fabricate_request_response(
                original_url, true, false));
            return output_list
        }
    }

    if !scrape_listable { return output_list }

    let scraped_urls:Vec<String> = content_parse::scrape_urls(content, dir_url);

    for scraped_url in scraped_urls {
        if !scraped_url.ends_with("/") {
            output_list.push(fabricate_request_response(
                scraped_url, false, false));
        }
        else {
            if !disable_recursion {
                output_list.append(&mut listable_check(easy, scraped_url, disable_recursion, scrape_listable));
            }
            else {
                output_list.push(fabricate_request_response(scraped_url, true, false))
            }
        }
    }
    //Pull out URLs and put them in a list (function call)


    //If it ends in / then make a request

    // otherwise add it to the list

    //For each folder found

    output_list
}

pub fn generate_easy(global_opts: Arc<GlobalOpts>) -> Easy2<Collector>
{
    // Create a new curl Easy2 instance and set it to use GET requests
    let mut easy = Easy2::new(Collector(Vec::new()));
    easy.get(true).unwrap();

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

    match &global_opts.user_agent {
        Some(user_agent) => { easy.useragent(&user_agent.clone()).unwrap(); },
        None => {}
    }

    match &global_opts.username {
        Some(username) => {
            easy.username(&username.clone()).unwrap();
            easy.password(&global_opts.password.clone().unwrap()).unwrap();
        },
        None => {}
    }

    match &global_opts.cookies {
        Some(cookies) => {
            easy.cookie(cookies).unwrap();
        },
        None => {}
    }

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

fn perform(easy: &mut Easy2<Collector>) -> Result<(), Error>
{
    easy.get_mut().clear_buffer();
    easy.perform()
}

fn get_content(easy: &mut Easy2<Collector>) -> String
{
    let contents = easy.get_ref();
    String::from_utf8_lossy(&contents.0).to_string()
}

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
        found_from_listable: true
    }
}