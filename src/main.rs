use std::fs::File;
use std::io::{BufRead, BufReader};
extern crate curl;
use curl::easy::{Easy2, Handler, WriteError};
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
#[macro_use]
extern crate clap;
use clap::App;

struct Collector(Vec<u8>);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0 = data.to_vec();
        Ok(data.len())
    }
}


fn main() {
    // Load the yaml file containing argument definitions
    let yml = load_yaml!("args.yml");
    let m = App::from_yaml(yml).get_matches();

    // Get the wordlist file from the arguments and open it
    let file = File::open(m.value_of("wordlist").unwrap()).unwrap();

    // Get the host URI from the arguments
    let hostname = m.value_of("host").unwrap();
    assert!(hostname.starts_with("https://") || hostname.starts_with("http://"), 
        "The provided target URI must start with http:// or https://");

    // Create a new curl Easy2 instance and set it to use GET requests
    let mut easy = Easy2::new(Collector(Vec::new()));
    easy.get(true).unwrap();

    // For each line in the file, call the request function on it
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        request(&mut easy, hostname, &line)
    }
}

// This function takes an instance of "Easy2", a base URL and a suffix
// It then makes the request, if the response was not a 404
// then it will print the URI it requested and the response
fn request(easy: &mut Easy2<Collector>, base: &str, end: &str) {

    //Concatenate and url encode the url, then set it in the Easy2 instance
    let url = format!("{}/{}", base, end);
    let url = utf8_percent_encode(&url, DEFAULT_ENCODE_SET).to_string();
    easy.url(&url).unwrap();

    //Perform the request and check if it's empty
    //If it's empty then output info and return
    match easy.perform() {
        Ok(_v) => {}
        Err(_e) => {   println!("- {} (CODE: 0|SIZE: 0)", url);
            return(); 
        }
    }

    // Get the response code
    let code = easy.response_code().unwrap();
    // Get the contents of the response
    let contents = easy.get_ref();
    
    // Print some output if the 
    match code {
        404 => return,
        301 | 302 => {
            let content_len = String::from_utf8_lossy(&contents.0).len();
            let redir_dest = easy.redirect_url().unwrap().unwrap();
            let dir_url = url.clone() + "/";
            if dir_url == redir_dest {
                println!("==> DIRECTORY: {}", dir_url);
            }
            else {
                println!("+ {} (CODE: {}|SIZE:{:#?}|DEST:{})", url, code, content_len, redir_dest);
            }
        },
        _ => {
            let content_len = String::from_utf8_lossy(&contents.0).len();
            println!("+ {} (CODE:{}|SIZE:{:#?})", url, code, content_len); 
        },
    }

    // if code != 404 
    // { 
    //     let content_len = String::from_utf8_lossy(&contents.0).len();
    //     println!("+ {} (CODE:{}|SIZE:{:#?})", url, code, content_len); 
    // }


}