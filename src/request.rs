use std::sync::Arc;
use percent_encoding::percent_decode;
extern crate curl;
use curl::easy::{Easy2, Handler, WriteError};

pub struct Collector(pub Vec<u8>);

use crate::arg_parse::GlobalOpts;

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0 = data.to_vec();
        Ok(data.len())
    }
}

pub struct RequestResponse {
    pub url: String,
    pub code: u32,
    pub content_len: u32,
    pub is_directory: bool,
    pub is_listable: bool,
    pub redirect_url: String
}

// This function takes an instance of "Easy2", a base URL and a suffix
// It then makes the request, if the response was not a 404
// then it will print the URI it requested and the response
pub fn make_request(easy: &mut Easy2<Collector>, url: String, global_opts: Arc<GlobalOpts>) -> Option<RequestResponse>{

    // Set the url in the Easy2 instance
    easy.url(&url).unwrap();

    // Perform the request and check if it's empty
    // If it's empty then output info and return
    match easy.perform() {
        Ok(_v) => {}
        Err(_e) => {   //println!("- {} (CODE: 0|SIZE: 0)", url);
            let req_response = RequestResponse {
                url: url.clone(),
                code: 0,
                content_len: 0,
                is_directory:false,
                is_listable: false,
                redirect_url: String::from("")
            };
            return Some(req_response); 
        }
    }

    // Get the response code
    let code = easy.response_code().unwrap();
    
    // Determine what to do based on the code

    if code == 404 {
        return None;
    }

    let mut req_response = RequestResponse {
        url: url.clone(),
        code: code,
        content_len: 0,
        is_directory:false,
        is_listable: false,
        redirect_url: String::from("")
    };

    match req_response.code {
        // If a 404 code, just return the response code
        //404 => return None,

        // 403 => {

        //     if !global_opts.show_htaccess && ( url.ends_with("/.htaccess") || url.ends_with("/.hta") 
        //         || url.ends_with("/.htpasswd") ) { return code }
            
        //     let content_len = String::from_utf8_lossy(&contents.0).len();
        //     println!("+ {} (CODE:{}|SIZE:{:#?})", url, code, content_len); 
        //     code

        // }
        // If it's a redirect, then check if the redirection destination is
        // to the same URL with a / on the end, if so return 1 so that the
        // calling thread knows this is a discovered folder
        // Otherwise, just print some output and return the response code
        301 | 302 => {
            // Get and url decode the redirect destination
            let redir_dest = easy.redirect_url().unwrap().unwrap();
            let redir_dest = percent_decode(redir_dest.as_bytes()).decode_utf8().unwrap();

            // Clone and url decode the url
            let dir_url = url.clone() + "/";
            let dir_url = percent_decode(dir_url.as_bytes()).decode_utf8().unwrap();

            req_response.redirect_url = dir_url.to_string();

            if dir_url == redir_dest {
                //println!("==> DIRECTORY: {}", url);
                req_response.is_directory = true;
                //1
            }
            //else {
                //println!("+ {} (CODE: {}|SIZE:{:#?}|DEST:{})", url, code, content_len, redir_dest);
            //}
        },
        // If it's anything else, print out some information and return the response code
        _ => {
            //let content_len = String::from_utf8_lossy(&contents.0).len();
            //println!("+ {} (CODE:{}|SIZE:{:#?})", url, code, content_len); 
        },

    }

    // Get the contents of the response
    let contents = easy.get_ref();
    req_response.content_len = String::from_utf8_lossy(&contents.0).len() as u32;
    Some(req_response)
}