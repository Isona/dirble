use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write, stdout}; //, Result};
extern crate curl;
use curl::easy::Easy;
//use std::io::{stdout, Write};

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args);
    let mut easy = Easy::new();

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

// This function takes an instance of "Easy", a base URL and a suffix
// It then makes the request and will print the page it requested and the response
// if the response was not a 404
fn request(easy: &mut Easy, base: &str, end: &str) {
    let url = format!("{}/{}", base, end);

    easy.url(&url).unwrap();
    // let result_len = easy.write_function(|data| {
    //     //stdout().write_all(data).unwrap();
    //     //println!("{}", data.len());
    //     //Ok(data.len())
    //     Ok(data.len())
    // }).unwrap();
    easy.perform().unwrap();
    let code = easy.response_code().unwrap();
    let result_len = easy.size_download().unwrap();
    if code != 404 { println!("+ {} (CODE:{}|SIZE:{:#?})", url, code, result_len); }
}