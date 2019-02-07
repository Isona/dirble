use percent_encoding::percent_decode;
extern crate curl;
use curl::easy::{Easy2, Handler, WriteError};

pub struct Collector(pub Vec<u8>);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0 = data.to_vec();
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
    pub redirect_url: String
}

// This function takes an instance of "Easy2", a base URL and a suffix
// It then makes the request, if the response was not a 404
// then it will return a RequestResponse struct
pub fn make_request(easy: &mut Easy2<Collector>, url: String) -> Option<RequestResponse>{

    // Set the url in the Easy2 instance
    easy.url(&url).unwrap();

    // Perform the request and check if it's empty
    // If it's empty then return a RequestResponse struct
    match easy.perform() {
        Ok(_v) => {}
        Err(_e) => {
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
        redirect_url: String::from("")
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