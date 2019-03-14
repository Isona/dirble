use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use crate::request::RequestResponse;
use crate::arg_parse::GlobalOpts;
use std::error::Error;
use std::io::{LineWriter, Write};


pub struct FileHandles {
    pub output_file: Option<LineWriter<File>>
}

pub fn print_response(response: &RequestResponse, global_opts: Arc<GlobalOpts>, folder_line: bool) -> Option<String> {
    match response.code {
        403 => {
            if !global_opts.show_htaccess && response.url.contains("/.ht") { None }
            else {
                Some(format!("+ {} (CODE:{}|SIZE:{:#?})", response.url, response.code, response.content_len))
            }
        }
        301 | 302 => {
            if response.is_directory {
                if response.is_listable {
                    if folder_line { Some(format!("\n==> LISTABLE DIRECTORY: {}", response.url)) }
                    else { Some(format!("==> LISTABLE DIRECTORY: {}", response.url)) }
                }
                else {
                    if folder_line { Some(format!("\n==> DIRECTORY: {}", response.url)) }
                    else { Some(format!("==> DIRECTORY: {}", response.url)) }
                }
            }
            else {
                Some(format!("+ {} (CODE: {}|SIZE:{:#?}|DEST:{})", 
                    response.url, response.code, response.content_len, response.redirect_url))
            }
        }
        _ => {
            Some(format!("+ {} (CODE:{}|SIZE:{:#?})", response.url, response.code, response.content_len)) 
        }

    }
}

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
        match print_response(&response, global_opts.clone(), true) {
            Some(line) => { 
                println!("{}", line);
                let file_line = format!("{}\n", line);
                file_handle = write_file(file_handle, file_line);
            },
            None => {}
        }
    }
}

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

pub fn sort_responses(mut responses: Vec<RequestResponse>) -> Vec<RequestResponse> {
    responses.sort_by(|a, b| {
        directory_name(&a).cmp(&directory_name(&b))
            .then(a.url.cmp(&b.url))
    });

    return responses;
}

pub fn directory_name(response:&RequestResponse) -> String
{
    if response.is_directory { response.url.clone() }
    else {
        let last_slash = response.url.rfind("/").unwrap();
        String::from(&response.url[0..last_slash])
    }
}

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