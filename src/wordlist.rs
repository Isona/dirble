// This file is part of Dirble - https://www.github.com/nccgroup/dirble
// Copyright (C) 2019 Izzy Whistlecroft <Izzy(dot)Whistlecroft(at)nccgroup(dot)com>
// Released as open source by NCC Group Plc - https://www.nccgroup.com/
//
// Dirble is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Dirble is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Dirble.  If not, see <https://www.gnu.org/licenses/>.

use std::{
    process::exit,
    sync::Arc,
    fs::File,
    io::prelude::*
};
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use chardet::{detect, charset2encoding};
use encoding::{
    DecoderTrap,
    label::encoding_from_whatwg_label
};
use crate::validator_thread::TargetValidator;
use log::error;


// Struct for a UriGenerator, it needs the hostname, the suffix to append, a wordlist and an index into that wordlist
pub struct UriGenerator {
    pub hostname: String,
    prefix: String,
    suffix: String,
    current_index: usize,
    wordlist: Arc<Vec<String>>,
    step_size: usize,
    pub parent_depth: u32,
    pub validator:Option<TargetValidator>
}

// Generates a new UriGenerator given various options
impl UriGenerator {
    pub fn new(mut hostname: String, prefix: String, suffix: String, 
        wordlist: Arc<Vec<String>>, index: u32, step: u32, parent_depth:u32,
        validator:Option<TargetValidator>) -> UriGenerator{
        // Remove a trailing / characters from the url if there is one
        if hostname.ends_with("/") {
            hostname.pop();
        }
        
        UriGenerator { 
            hostname,
            prefix,
            suffix,
            current_index: index as usize,
            wordlist,
            step_size: step as usize,
            parent_depth,
            validator
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
        let uri = self.hostname.clone() + "/" + &self.prefix + &self.wordlist[self.current_index].clone() + &self.suffix;
        let uri = utf8_percent_encode(&uri, DEFAULT_ENCODE_SET).to_string();

        // Maintain the index into the wordlist
        self.current_index += self.step_size;
        // Return the generated Uri
        Some(uri)

    }
}

// Function used to read in lines from the wordlist file
pub fn lines_from_file(filename: &String) -> Vec<String>
{
    let mut file = File::open(filename.clone())
        .unwrap_or_else(|error| { error!("Opening file \"{}\" failed: {}", filename, error); exit(2);});
    let mut reader: Vec<u8> = Vec::new();

    // Read the raw file in as a vector of bytes
    file.read_to_end(&mut reader)
        .expect("Error reading file");

    // Detect the charset of the file
    let result = detect(&reader);
    // result.0 Encode
    // result.1 Confidence
    // result.2 Language

    // Decode the file into UTF-8 from the guessed encoding
    let coder = encoding_from_whatwg_label(charset2encoding(&result.0));
    match coder {
        Some(coding) => {
            return coding.decode(&reader, DecoderTrap::Ignore)
                .expect("Error decoding to UTF-8")
                .lines()
                .map(|s| String::from(s))
                .collect();
        },
        None => {
            panic!("Error detecting file encoding");
        }
    }
}
