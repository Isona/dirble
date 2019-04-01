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

use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use crate::request::RequestResponse;
use crate::arg_parse::GlobalOpts;
use crate::output_format;
use std::error::Error;
use std::io::{LineWriter, Write};

// Struct giving access to each current file handle
// Will be extended in future with handles for different formats
pub struct FileHandles {
    pub output_file: Option<LineWriter<File>>
}

pub fn print_response(response: &RequestResponse, global_opts: Arc<GlobalOpts>, 
    print_newlines: bool, indentation: bool) -> Option<String> {
    if response.code == 403 && !global_opts.show_htaccess && response.url.contains("/.ht") 
    {
        return None 
    }

    let mut output = String::new();
    output += &output_format::output_indentation(&response, print_newlines, indentation);

    output += &output_format::output_letter(&response);

    output += &output_format::output_url(&response);

    output += &output_format::output_suffix(&response);

    Some(output)
}

// Called after a scan to print the discovered items in a sorted way - deals with saving to files too
pub fn print_report(responses: Vec<RequestResponse>, global_opts: Arc<GlobalOpts>, file_handles: FileHandles) {
    let responses = sort_responses(responses);

    if !global_opts.silent || global_opts.verbose {
        println!("\n");
    }

    let mut file_handle = file_handles.output_file;
    let report_string = String::from("Dirble Scan Report: \n");
    println!("{}", report_string);
    file_handle = write_file(file_handle, report_string);

    for response in responses {
        match print_response(&response, global_opts.clone(), true, true) {
            Some(line) => { 
                println!("{}", line);
                let file_line = format!("{}\n", line);
                file_handle = write_file(file_handle, file_line);
            },
            None => {}
        }
    }
}

// If a file was provided to save normally formatted output, this will write a string to it
fn write_file(mut file_writer: Option<LineWriter<File>>, line: String) -> Option<LineWriter<File>>
{
    let file_writer = file_writer.take();
    match file_writer {
        Some(mut writer) => { 
            let write_line = line.as_bytes();
            writer.write_all(write_line).unwrap();
            Some(writer)
        },
        None => { None }
    }
}

// Sorts responses so that files in a directory come first, followed by the subdirs
pub fn sort_responses(mut responses: Vec<RequestResponse>) -> Vec<RequestResponse> {
    responses.sort_by(|a, b| {
        directory_name(&a).cmp(&directory_name(&b))
            .then(a.url.cmp(&b.url))
    });

    return responses;
}

// Gets the base directory name of the requested url of the given struct
pub fn directory_name(response:&RequestResponse) -> String
{
    if response.is_directory { 
        if response.url.ends_with("/") {
            String::from(&response.url[0..response.url.len()-1])
        }
        else {
            response.url.clone()
        }
    }
    else {
        let last_slash = response.url.rfind("/").unwrap();
        String::from(&response.url[0..last_slash])
    }
}

// Returns a FileHandles struct with LineWriters for each specified output type
pub fn create_files(global_opts: Arc<GlobalOpts>) -> FileHandles {
    let mut output_file = None;

    match &global_opts.output_file {
        Some(name) => {
            let path = Path::new(&name);
            let display = path.display();
            output_file = match File::create(&path) {
                Err(why) => panic!("couldn't create {}: {}",
                                   display,
                                   why.description()),
                Ok(file) => Some(LineWriter::new(file)),
            };

        },
        None => {}
    };

    FileHandles {
        output_file: output_file
    }
}