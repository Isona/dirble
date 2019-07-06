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
use crate::arg_parse::{GlobalOpts, get_version_string};
use crate::output_format;
use std::error::Error;
use std::io::{LineWriter, Write};
use simplelog::LevelFilter;

// Struct giving access to each current file handle
// Will be extended in future with handles for different formats
pub struct FileHandles {
    pub output_file: Option<LineWriter<File>>,
    pub json_file: Option<LineWriter<File>>,
    pub xml_file: Option<LineWriter<File>>
}

pub fn print_response(response: &RequestResponse, global_opts: Arc<GlobalOpts>, 
    print_newlines: bool, indentation: bool, colour: bool) -> Option<String> {
    if response.code == 403 && !global_opts.show_htaccess && response.url.contains("/.ht") 
    {
        return None 
    }

    let mut output = String::new();
    output += &output_format::output_indentation(&response, print_newlines, indentation);

    output += &output_format::output_letter(&response);

    output += &output_format::output_url(&response);

    output += &output_format::output_suffix(&response, colour);

    Some(output)
}

// Called after a scan to print the discovered items in a sorted way - deals with saving to files too
pub fn print_report(responses: Vec<RequestResponse>, global_opts: Arc<GlobalOpts>, file_handles: FileHandles) {
    let responses = sort_responses(responses);

    if global_opts.log_level >= LevelFilter::Info && global_opts.is_terminal {
        println!("\n");
    }

    let report_string = String::from("Dirble Scan Report: \n");

    // If stdout is a terminal then write a report to it
    if global_opts.is_terminal
    {
        println!("{}", report_string);
        for response in &responses {
            if let Some(line) = print_response(&response, global_opts.clone(), 
                true, true, !global_opts.no_color) {
                println!("{}", line);
            }
        }
    }
    
    
    // If it was provided, write to a normally formatted output file
    if let Some(mut handle) = file_handles.output_file {
        write_file(&mut handle, report_string);

        for response in &responses {
            if let Some(line) = print_response(&response, global_opts.clone()
                , true, false, false) {
                let file_line = format!("{}\n", line);
                write_file(&mut handle, file_line);
            }
        }
    }

    if responses.len() > 0 {
        if let Some(mut handle) = file_handles.json_file {
            write_file(&mut handle, String::from("["));
            for response in &responses[0..responses.len()-1] {
                let line = format!("{},\n", output_format::output_json(response));
                write_file(&mut handle, line);
            }
            let final_line = format!("{}]", output_format::output_json(&responses[responses.len()-1]));
            write_file(&mut handle, final_line);
        }
    }

    if let Some(mut handle) = file_handles.xml_file {
        write_file(&mut handle, String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"));
        write_file(&mut handle, String::from("<dirble_scan>\n"));
        for response in &responses {
            write_file(&mut handle, output_format::output_xml(response));
        }
        write_file(&mut handle, String::from("</dirble_scan>"));
    }
}

// Write a string to the provided LineWriter
#[inline]
fn write_file(file_writer: &mut LineWriter<File>, line: String) {
    let write_line = line.as_bytes();
    file_writer.write_all(write_line).unwrap();
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

    if let Some(filename) = &global_opts.output_file {
        output_file = generate_handle(filename);
    }

    let mut json_file = None;
    if let Some(filename) = &global_opts.json_file {
        json_file = generate_handle(filename);
    }

    let mut xml_file = None;
    if let Some(filename) = &global_opts.xml_file {
        xml_file = generate_handle(filename);
    }

    FileHandles {
        output_file: output_file,
        json_file: json_file,
        xml_file: xml_file
    }
}

#[inline]
fn generate_handle(filename: &String) -> Option<LineWriter<File>>
{
    let path = Path::new(&filename);
    let display = path.display();
    match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}",
                           display,
                           why.description()),
        Ok(file) => Some(LineWriter::new(file)),
    }
}

// Prints out start up information
pub fn startup_text(global_opts: Arc<GlobalOpts>, wordlist_file: &String) {
    if !global_opts.is_terminal { return }

    println!("Dirble {}", get_version_string());
    println!("Developed by Izzy Whistlecroft\n");

    println!("Targets: {}", global_opts.hostnames.clone().join(" "));
    if let Some(globalopts_wordlists) = global_opts.wordlist_files.clone() {
        println!("Wordlists: {}", globalopts_wordlists.join(" "));
    }
    else {
        println!("Wordlist: {}", wordlist_file);
    }

    if global_opts.prefixes.len() == 1 && global_opts.prefixes[0] == "" {
        println!("No Prefixes");
    }
    else {
        println!("Prefixes: {}", global_opts.prefixes.clone()[1..].join(" "));
    }

    if global_opts.extensions.len() == 1 && global_opts.extensions[0] == "" {
        println!("No Extensions");
    }
    else {
        println!("Extensions: {}", global_opts.extensions.clone()[1..].join(" "));
    }

    if global_opts.length_blacklist.is_empty() {
        println!("No lengths hidden");
    }
    else {
        println!("Hidden lengths: {}", global_opts.length_blacklist);
    }
    println!("");
}
