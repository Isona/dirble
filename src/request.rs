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
pub fn make_request(easy: &mut Easy2<Collector>, url: String) -> Option<RequestResponse>{

    // Set the url in the Easy2 instance
    easy.url(&url).unwrap();

    // Perform the request and check if it's empty
    // If it's empty then output info and return
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
        301 | 302 => {
            // Get and url decode the redirect destination
            let redir_dest = easy.redirect_url().unwrap().unwrap();
            let redir_dest = percent_decode(redir_dest.as_bytes()).decode_utf8().unwrap();

            // Clone and url decode the url
            let dir_url = url.clone() + "/";
            let dir_url = percent_decode(dir_url.as_bytes()).decode_utf8().unwrap();

            req_response.redirect_url = dir_url.to_string();

            if dir_url == redir_dest {
                req_response.is_directory = true;
            }
        },
        _ => {
        },

    }

    // Get the contents of the response
    let contents = easy.get_ref();
    req_response.content_len = String::from_utf8_lossy(&contents.0).len() as u32;
    Some(req_response)
}