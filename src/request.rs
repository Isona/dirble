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

// This function takes an instance of "Easy2", a base URL and a suffix
// It then makes the request, if the response was not a 404
// then it will print the URI it requested and the response
pub fn make_request(easy: &mut Easy2<Collector>, url: String) -> u32{

    // Set the url in the Easy2 instance
    easy.url(&url).unwrap();

    // Perform the request and check if it's empty
    // If it's empty then output info and return
    match easy.perform() {
        Ok(_v) => {}
        Err(_e) => {   println!("- {} (CODE: 0|SIZE: 0)", url);
            return 0; 
        }
    }

    // Get the response code
    let code = easy.response_code().unwrap();
    // Get the contents of the response
    let contents = easy.get_ref();
    
    // Determine what to do based on the code
    match code {
        // If a 404 code, just return the response code
        404 => return code,
        // If it's a redirect, then check if the redirection destination is
        // to the same URL with a / on the end, if so return 1 so that the
        // calling thread knows this is a discovered folder
        // Otherwise, just print some output and return the response code
        301 | 302 => {
            let content_len = String::from_utf8_lossy(&contents.0).len();

            // Get and url decode the redirect destination
            let redir_dest = easy.redirect_url().unwrap().unwrap();
            let redir_dest = percent_decode(redir_dest.as_bytes()).decode_utf8().unwrap();

            // Clone and url decode the url
            let dir_url = url.clone() + "/";
            let dir_url = percent_decode(dir_url.as_bytes()).decode_utf8().unwrap();

            if dir_url == redir_dest {
                println!("==> DIRECTORY: {}", url);
                1
            }
            else {
                println!("+ {} (CODE: {}|SIZE:{:#?}|DEST:{})", url, code, content_len, redir_dest);
                code
            }
        },
        // If it's anything else, print out some information and return the response code
        _ => {
            let content_len = String::from_utf8_lossy(&contents.0).len();
            println!("+ {} (CODE:{}|SIZE:{:#?})", url, code, content_len); 
            code
        },
    }
}