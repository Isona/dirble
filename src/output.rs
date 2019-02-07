use std::sync::Arc;
use crate::request::RequestResponse;
use crate::arg_parse::GlobalOpts;

pub fn print_response(response: &RequestResponse, global_opts: Arc<GlobalOpts>) {
    match response.code {
        403 => {
            if !global_opts.show_htaccess && ( response.url.ends_with("/.htaccess") || response.url.ends_with("/.hta") 
                || response.url.ends_with("/.htpasswd") ) { }
            else {
            println!("+ {} (CODE:{}|SIZE:{:#?})", response.url, response.code, response.content_len); 
            }
        }
        301 | 302 => {
            if response.is_directory {
                println!("==> DIRECTORY: {}", response.url);
            }
            else {
                println!("+ {} (CODE: {}|SIZE:{:#?}|DEST:{})", 
                    response.url, response.code, response.content_len, response.redirect_url);
            }
        }
        _ => {
            println!("+ {} (CODE:{}|SIZE:{:#?})", response.url, response.code, response.content_len); 
        }

    }
}