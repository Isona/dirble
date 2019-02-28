use std::{
    sync::Arc,
    path::Path,
    fs::File,
    io::{self, BufRead, BufReader},
};
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

// Struct for a UriGenerator, it needs the hostname, the suffix to append, a wordlist and an index into that wordlist
pub struct UriGenerator {
    pub hostname: String,
    suffix: String,
    current_index: usize,
    wordlist: Arc<Vec<String>>,
    step_size: usize
}

// Implement a function that generates a new UriGenerator
impl UriGenerator {
    pub fn new(hostname: String, suffix: String, wordlist: Arc<Vec<String>>, index: u32, step: u32) -> UriGenerator{
        UriGenerator { 
            hostname: hostname,
            suffix: suffix,
            current_index: index as usize,
            wordlist: wordlist,
            step_size: step as usize
        }
    }
}

// Defines iterating over a UriGenerator
impl Iterator for UriGenerator {
    type Item = (String);

    fn next(&mut self) -> Option<Self::Item> {
        
        // If we're at the end of the wordlist then return None
        if self.current_index >= self.wordlist.len() {
            return None;
        }
        // Concatenate the hostname with the current wordlist item and the suffix, then url encode
        let uri = self.hostname.clone() + "/" + &self.wordlist[self.current_index].clone() + &self.suffix;
        let uri = utf8_percent_encode(&uri, DEFAULT_ENCODE_SET).to_string();

        // Maintain the index into the wordlist
        self.current_index += self.step_size;
        // Return the generated Uri
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