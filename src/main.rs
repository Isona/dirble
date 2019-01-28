use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
extern crate curl;
use curl::easy::{Easy2, Handler, WriteError};

struct Collector(Vec<u8>);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0 = data.to_vec();
        Ok(data.len())
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args);
    let mut easy = Easy2::new(Collector(Vec::new()));
    easy.get(true).unwrap();

    let file = File::open(config.filename).unwrap();
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        request(&mut easy, &config.hostname, &line)
    }
}

struct Config {
    hostname: String,
    filename: String,
}

impl Config {
    fn new(args: &[String]) -> Config {
        if args.len() < 3 {
            panic!("Not enough arguments");
        }
        
        let hostname = args[1].clone();
        let filename = args[2].clone();

        Config { hostname, filename }
    }
}

// This function takes an instance of "Easy2", a base URL and a suffix
// It then makes the request, if the response was not a 404
// then it will print the URI it requested and the response
fn request(easy: &mut Easy2<Collector>, base: &str, end: &str) {
    let url = format!("{}/{}", base, end);

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