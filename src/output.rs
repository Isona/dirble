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

use crate::arg_parse::{get_version_string, GlobalOpts};
use crate::output_format;
use crate::request::RequestResponse;
use std::fs::File;
use std::io::{LineWriter, Write};
use std::path::Path;
use std::sync::Arc;

// Struct giving access to each current file handle
// Will be extended in future with handles for different formats
pub struct FileHandles {
    pub output_file: Option<LineWriter<File>>,
    pub json_file: Option<LineWriter<File>>,
    pub xml_file: Option<LineWriter<File>>,
}

pub fn print_response(
    response: &RequestResponse,
    global_opts: Arc<GlobalOpts>,
    print_newlines: bool,
    indentation: bool,
    colour: bool,
) -> Option<String> {
    if response.code == 403
        && !global_opts.show_htaccess
        && response.url.path().contains("/.ht")
    {
        return None;
    }

    let mut output = String::new();
    output += &output_format::output_indentation(
        response,
        print_newlines,
        indentation,
    );

    output += &output_format::output_letter(response);

    output += &output_format::output_url(response);

    output += &output_format::output_suffix(response, colour);

    Some(output)
}

// Called after a scan to print the discovered items in a sorted way -
// deals with saving to files too
pub fn print_report(
    mut responses: Vec<Vec<RequestResponse>>,
    global_opts: Arc<GlobalOpts>,
    file_handles: FileHandles,
) {
    for response_list in &mut responses {
        //*response_list =
        sort_responses(response_list);
    }

    // If stdout is a terminal then write a report to it
    if global_opts.is_terminal {
        for (index, response_list) in responses.iter().enumerate() {
            println!(
                "\nDirble Scan Report for {}:",
                global_opts.hostnames[index]
            );
            for response in response_list {
                if let Some(line) = print_response(
                    response,
                    global_opts.clone(),
                    true,
                    true,
                    !global_opts.no_color,
                ) {
                    println!("{}", line);
                }
            }
        }
    }

    // If it was provided, write to a normally formatted output file
    if let Some(mut handle) = file_handles.output_file {
        for (index, response_list) in responses.iter().enumerate() {
            let report_string = format!(
                "Dirble Scan Report for {}:\n",
                global_opts.hostnames[index]
            );
            write_file(&mut handle, report_string);
            for response in response_list {
                if let Some(line) = print_response(
                    response,
                    global_opts.clone(),
                    true,
                    false,
                    false,
                ) {
                    let file_line = format!("{}\n", line);
                    write_file(&mut handle, file_line);
                }
            }
            write_file(&mut handle, "\n".to_string());
        }
    }

    if !responses.is_empty() {
        if let Some(mut handle) = file_handles.json_file {
            write_file(&mut handle, String::from("["));
            for response_list in &responses[0..responses.len() - 1] {
                for response in response_list {
                    let line =
                        format!("{},\n", output_format::output_json(response));
                    write_file(&mut handle, line);
                }
            }
            let final_response_list = &responses[responses.len() - 1];
            for response in
                &final_response_list[0..final_response_list.len() - 1]
            {
                let line =
                    format!("{},\n", output_format::output_json(response));
                write_file(&mut handle, line);
            }
            let final_line = format!(
                "{}]",
                output_format::output_json(
                    &final_response_list[final_response_list.len() - 1]
                )
            );
            write_file(&mut handle, final_line);
        }
    }

    if let Some(mut handle) = file_handles.xml_file {
        write_file(
            &mut handle,
            String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"),
        );
        write_file(&mut handle, String::from("<dirble_scan>\n"));
        for response_list in responses {
            for response in &response_list {
                write_file(&mut handle, output_format::output_xml(response));
            }
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

// Sorts responses so that files in a directory come first, followed by
// the subdirs
pub fn sort_responses(responses: &mut [RequestResponse]) {
    responses.sort_by(|a, b| {
        directory_name(a)
            .cmp(&directory_name(b))
            .then(a.url.cmp(&b.url))
    });
}

// Gets the base directory name of the requested url of the given struct
#[inline]
pub fn directory_name(response: &RequestResponse) -> String {
    let url = response.url.as_str();
    if response.is_directory {
        if url.ends_with('/') {
            String::from(&url[0..url.len() - 1])
        } else {
            String::from(url)
        }
    } else {
        let last_slash = url.rfind('/').unwrap();
        String::from(&url[0..last_slash])
    }
}

// Returns a FileHandles struct with LineWriters for each specified
// output type
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
        output_file,
        json_file,
        xml_file,
    }
}

#[inline]
fn generate_handle(filename: &str) -> Option<LineWriter<File>> {
    let path = Path::new(&filename);
    match File::create(path) {
        Err(why) => panic!("couldn't create {}: {}", path.display(), why),
        Ok(file) => Some(LineWriter::new(file)),
    }
}

// Prints out start up information
pub fn startup_text(
    global_opts: Arc<GlobalOpts>,
    wordlist_file: &str,
) -> Option<String> {
    if !global_opts.is_terminal {
        return None;
    }

    let text = format!("Dirble {}\n", get_version_string());
    let text = format!("{}Developed by Izzy Whistlecroft\n", text);

    let text = format!(
        "{}Targets:{}\n",
        text,
        global_opts
            .hostnames
            .clone()
            .iter()
            .fold(String::from(""), |acc, x| acc + " " + x.as_str())
    );

    let text = if let Some(globalopts_wordlists) =
        global_opts.wordlist_files.clone()
    {
        format!("{}Wordlists: {}\n", text, globalopts_wordlists.join(" "))
    } else {
        format!("{}Wordlist: {}\n", text, wordlist_file)
    };

    let text = if global_opts.prefixes.len() == 1
        && global_opts.prefixes[0].is_empty()
    {
        format!("{}No Prefixes\n", text)
    } else {
        format!(
            "{}Prefixes: {}\n",
            text,
            global_opts.prefixes.clone()[1..].join(" ")
        )
    };

    let text = if global_opts.extensions.len() == 1
        && global_opts.extensions[0].is_empty()
    {
        format!("{}No Extensions\n", text)
    } else {
        format!(
            "{}Extensions: {}\n",
            text,
            global_opts.extensions.clone()[1..].join(" ")
        )
    };

    let text = if global_opts.length_blacklist.is_empty() {
        format!("{}No lengths hidden\n", text)
    } else {
        format!("{}Hidden lengths: {}\n", text, global_opts.length_blacklist)
    };

    Some(text)
}

#[cfg(test)]
mod test {
    use crate::arg_parse::{GlobalOpts, LengthRange, LengthRanges};
    use crate::output::{
        directory_name, print_response, sort_responses, startup_text,
    };
    use crate::request::RequestResponse;
    use std::sync::Arc;
    use url::Url;

    #[test]
    fn test_print_response() {
        // Only the hidden and not hidden parts of output::print_response
        // need to be tested - the rest is covered by the testing in
        // tests/output_format.rs.
        let rr = RequestResponse {
            url: Url::parse("http://example.com/.htaccess").unwrap(),
            code: 403,
            content_len: 234,
            is_directory: false,
            is_listable: false,
            redirect_url: Default::default(),
            found_from_listable: false,
            parent_index: 0,
            parent_depth: 0,
        };
        let mut globalopts: GlobalOpts = Default::default();

        // Verify that htaccess files are hidden when the option is set in
        // globalopts
        globalopts.show_htaccess = false;
        let output = print_response(
            &rr,
            Arc::new(globalopts.clone()),
            false,
            false,
            false,
        );
        assert_eq!(output, None, ".htaccess is not being hidden");
        // And check that they are not hidden otherwise
        globalopts.show_htaccess = true;
        let output =
            print_response(&rr, Arc::new(globalopts), false, false, false);
        assert_eq!(
            output,
            Some(String::from(
                "+ http://example.com/.htaccess (CODE:403|SIZE:234)"
            )),
            ".htaccess is not being hidden"
        );
    }

    //#[test]
    //fn test_print_report() {}

    //#[test]
    //fn test_write_file() {}

    #[test]
    fn test_sort_responses() {
        let mut rr = RequestResponse {
            url: Url::parse("http://example.com/").unwrap(),
            code: 200,
            content_len: 200,
            is_directory: false,
            is_listable: false,
            redirect_url: "".into(),
            found_from_listable: false,
            parent_index: 0,
            parent_depth: 0,
        };

        // Generate a Vec of RequestResponses to sort
        let num_test_cases: usize = 4;
        let mut rr_vec: Vec<RequestResponse> =
            Vec::with_capacity(num_test_cases);

        rr_vec.push(rr.clone());

        rr.url = Url::parse("http://example.com/two").unwrap();
        rr_vec.push(rr.clone());

        rr.url = Url::parse("http://example.com/one/three").unwrap();
        rr_vec.push(rr.clone());

        rr.url = Url::parse("http://example.com/one/").unwrap();
        rr.is_directory = true;
        rr_vec.push(rr.clone());
        assert_eq!(
            num_test_cases,
            rr_vec.len(),
            "Length of test vector does not match expected number of cases"
        );

        dbg!(&rr_vec);

        let mut sorted = rr_vec.clone();
        sort_responses(&mut sorted);

        dbg!(&sorted);

        let sorted_urls: Vec<Url> = vec![
            Url::parse("http://example.com/").unwrap(),
            Url::parse("http://example.com/two").unwrap(),
            Url::parse("http://example.com/one/").unwrap(),
            Url::parse("http://example.com/one/three").unwrap(),
        ];

        assert_eq!(&sorted.len(), &sorted_urls.len());

        for pair in sorted.iter().zip(sorted_urls) {
            assert_eq!(pair.0.url, pair.1);
        }
    }

    #[test]
    fn test_directory_name() {
        let mut rr: RequestResponse = Default::default();
        // First case: rr is a directory ending with slash
        rr.url = Url::parse("http://example.com/test/dir/").unwrap();
        rr.is_directory = true;
        assert_eq!(
            directory_name(&rr),
            String::from("http://example.com/test/dir")
        );

        // Second case: rr is a directory not ending with slash
        rr.url = Url::parse("http://example.com/test/dir").unwrap();
        assert_eq!(
            directory_name(&rr),
            String::from("http://example.com/test/dir")
        );

        // Second case: rr is not a directory
        rr.is_directory = false;
        assert_eq!(
            directory_name(&rr),
            String::from("http://example.com/test")
        );
    }

    //#[test]
    //fn test_create_files() {}

    //#[test]
    //fn test_generate_handle() {}

    #[test]
    fn test_startup_text() {
        let mut globalopts: GlobalOpts = Default::default();

        // Startup text should be None when the stdout is not going to a
        // terminal
        let text =
            startup_text(Arc::new(globalopts.clone()), &String::from("foo"));
        assert_eq!(text, None);

        // Startup text should have a simple value for default globalopts
        globalopts.is_terminal = true;
        let text =
            startup_text(Arc::new(globalopts.clone()), &String::from("foo"));
        // Version string changes with each build, so we need to get the
        // current value before validating the startup text. If the text is
        // not a Some(text) then the unwrap will panic and the test will
        // fail.
        let version = crate::arg_parse::get_version_string();
        let suffix = String::from(
            "\nDeveloped by Izzy Whistlecroft\nTargets:\nWordlist: foo\n\
         No Prefixes\nNo Extensions\nNo lengths hidden\n",
        );
        assert_eq!(text.unwrap(), format!("Dirble {}{}", version, suffix));

        // Set all of the optional parameters, output text should display
        // them.
        globalopts.hostnames = vec![
            Url::parse("http://example.com").unwrap(),
            Url::parse("http://example.org").unwrap(),
        ];
        globalopts.wordlist_files = Some(vec!["foo".into(), "bar".into()]);
        globalopts.prefixes = vec!["".into(), "~".into()];
        globalopts.extensions = vec!["".into(), ".txt".into(), ".com".into()];
        globalopts.length_blacklist = LengthRanges {
            ranges: vec![
                LengthRange {
                    start: 2400,
                    end: Some(3000),
                },
                LengthRange {
                    start: 400,
                    end: None,
                },
            ],
        };
        let text =
            startup_text(Arc::new(globalopts.clone()), &String::from("foo"));
        let suffix = String::from(
            "\nDeveloped by Izzy Whistlecroft\n\
         Targets: http://example.com/ http://example.org/\n\
         Wordlists: foo bar\nPrefixes: ~\nExtensions: .txt .com\n\
         Hidden lengths: [400, 2400-3000]\n",
        );
        assert_eq!(text.unwrap(), format!("Dirble {}{}", version, suffix));
    }
}
