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

use crate::validator_thread::TargetValidator;
use chardet::{charset2encoding, detect};
use encoding::{label::encoding_from_whatwg_label, DecoderTrap};
use std::{fs, sync::Arc};
use url::Url;

// Struct for a UriGenerator, it needs the hostname, the suffix to
// append, a wordlist and an index into that wordlist
pub struct UriGenerator {
    pub base: Url,
    prefix: String,
    suffix: String,
    current_index: usize,
    wordlist: Arc<Vec<String>>,
    step_size: usize,
    pub parent_index: usize,
    pub parent_depth: u32,
    pub validator: Option<TargetValidator>,
    extension_substitution: bool,
}

// Generates a new UriGenerator given various options
impl UriGenerator {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        base: Url,
        prefix: String,
        suffix: String,
        wordlist: Arc<Vec<String>>,
        index: u32,
        step: u32,
        parent_index: usize,
        parent_depth: u32,
        validator: Option<TargetValidator>,
        extension_substitution: bool,
    ) -> Self {
        Self {
            base,
            prefix,
            suffix,
            current_index: index as usize,
            wordlist,
            step_size: step as usize,
            parent_index,
            parent_depth,
            validator,
            extension_substitution,
        }
    }
}

// Defines iterating over a UriGenerator
impl Iterator for UriGenerator {
    type Item = Url;

    fn next(&mut self) -> Option<Self::Item> {
        // If we're at the end of the wordlist then return None
        if self.current_index >= self.wordlist.len() {
            return None;
        }

        let uri = if !self.extension_substitution {
            // Append the prefixed and suffixed filename onto the URI
            self.base
                .join(
                    [
                        self.prefix.clone(),
                        self.wordlist[self.current_index].clone(),
                        self.suffix.clone(),
                    ]
                    .join("")
                    .as_str(),
                )
                .unwrap()
        } else {
            let word = self.wordlist[self.current_index]
                .replace("%EXT%", &self.suffix);
            self.base
                .join([self.prefix.clone(), word].join("").as_str())
                .unwrap()
        };

        // Maintain the index into the wordlist
        self.current_index += self.step_size;
        // Return the generated Uri
        Some(uri)
    }
}

// Function used to read in lines from the wordlist file
pub fn lines_from_file(filename: &str) -> Vec<String> {
    // Read the raw file in as a vector of bytes
    let reader = fs::read(filename).unwrap();

    // Detect the charset of the file
    let result = detect(&reader);
    // result.0 Encode
    // result.1 Confidence
    // result.2 Language

    // Decode the file into UTF-8 from the guessed encoding
    let coder = encoding_from_whatwg_label(charset2encoding(&result.0));
    match coder {
        Some(coding) => coding
            .decode(&reader, DecoderTrap::Ignore)
            .expect("Error decoding to UTF-8")
            .lines()
            .map(String::from)
            .collect(),
        None => {
            panic!(
                "Error detecting file encoding of {} - is the file empty?",
                filename
            );
        }
    }
}
