use std::{
    sync::Arc,
    path::Path,
    fs::File,
    io::{self, BufRead, BufReader},
};
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

pub struct UriGenerator {
    pub hostname: String,
    suffix: String,
    current_index: usize,
    wordlist: Arc<Vec<String>>,
}

impl UriGenerator {
    pub fn new(hostname: String, suffix: String, wordlist: Arc<Vec<String>>) -> UriGenerator{
        UriGenerator { hostname: hostname, suffix: suffix, current_index:0, wordlist: wordlist }
    }
}

impl Iterator for UriGenerator {
    type Item = (String);

    fn next(&mut self) -> Option<Self::Item> {
        
        if self.current_index >= self.wordlist.len() {
            return None;
        }
        let uri = self.hostname.clone() + "/" + &self.wordlist[self.current_index].clone() + &self.suffix;
        let uri = utf8_percent_encode(&uri, DEFAULT_ENCODE_SET).to_string();
        self.current_index += 1;
        Some(uri)

    }
}

// Function used to read in lines from the wordlist file
pub fn lines_from_file<P>(filename: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    BufReader::new(File::open(filename)?).lines().collect()
}