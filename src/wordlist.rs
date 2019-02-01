use std::path::Path;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

// Function used to read in lines from the wordlist file
pub fn lines_from_file<P>(filename: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    BufReader::new(File::open(filename)?).lines().collect()
}