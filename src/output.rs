use std::sync::Arc;
use std::cmp::Ordering;
use crate::request::RequestResponse;
use crate::arg_parse::GlobalOpts;

pub fn print_response(response: &RequestResponse, global_opts: Arc<GlobalOpts>, folder_line: bool) -> Option<String> {
    match response.code {
        403 => {
            if !global_opts.show_htaccess && ( response.url.ends_with("/.htaccess") || response.url.ends_with("/.hta") 
                || response.url.ends_with("/.htpasswd") ) { None }
            else {
            Some(format!("+ {} (CODE:{}|SIZE:{:#?})", response.url, response.code, response.content_len))
            }
        }
        301 | 302 => {
            if response.is_directory {
                if folder_line { Some(format!("\n==> DIRECTORY: {}", response.url)) }
                else { Some(format!("==> DIRECTORY: {}", response.url)) }
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

pub fn print_report(responses: Vec<RequestResponse>, global_opts: Arc<GlobalOpts>) {
    let responses = sort_responses(responses);

    if !global_opts.silent || global_opts.verbose {
        println!("\n");
    }
    println!("Scan Report: ");
    println!("");

    for response in responses {
        match print_response(&response, global_opts.clone(), true) {
            Some(line) => { println!("{}", line) },
            None => {}
        }
    }
}

pub fn sort_responses(mut responses: Vec<RequestResponse>) -> Vec<RequestResponse> {
    responses.sort_by( |a, b| {

        if !a.is_directory && b.is_directory {
            if a.url.starts_with(&format!("{}/", b.url)) {
                Ordering::Greater
            }
            else {
                Ordering::Less
            }
        }
        else if a.is_directory && !b.is_directory {
            if b.url.starts_with(&format!("{}/", a.url)) {
                Ordering::Less
            }
            else {
                Ordering::Greater
            }

        }
        else {
            let a_depth = a.url.matches("/").count();
            let b_depth = b.url.matches("/").count();

            if a_depth == b_depth {
                a.url.cmp(&b.url)
            }
            else {
                a_depth.cmp(&b_depth)
            }
        }
    });

    return responses;
}