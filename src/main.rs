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
    let yml = load_yaml!("args.yml");
    let m = App::from_yaml(yml).get_matches();

    let file = File::open(m.value_of("wordlist").unwrap()).unwrap();

    let hostname = m.value_of("host").unwrap();

    let mut easy = Easy2::new(Collector(Vec::new()));
    easy.get(true).unwrap();

    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        request(&mut easy, hostname, &line)
    }
}

// This function takes an instance of "Easy2", a base URL and a suffix
// It then makes the request, if the response was not a 404
// then it will print the URI it requested and the response
fn request(easy: &mut Easy2<Collector>, base: &str, end: &str) {
    let url = format!("{}/{}", base, end);
    let url = utf8_percent_encode(&url, DEFAULT_ENCODE_SET).to_string();

    easy.url(&url).unwrap();
    match easy.perform() {
        Ok(_v) => {}
        Err(_e) => 
        {   println!("- {} (CODE: 0|SIZE: 0)", url);
            return(); 
        }
    }

    let code = easy.response_code().unwrap();
    let contents = easy.get_ref();
    if code != 404 { println!("+ {} (CODE:{}|SIZE:{:#?})", url, code, String::from_utf8_lossy(&contents.0).len()); }

}