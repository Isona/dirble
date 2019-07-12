use crate::arg_parse::GlobalOpts;
use crate::request::RequestResponse;
use log::LevelFilter::Info;

mod argparse;
mod output;
mod output_format;
mod request_thread;

impl Default for GlobalOpts {
    fn default() -> Self {
        GlobalOpts {
            hostnames: Default::default(),
            wordlist_files: Default::default(),
            prefixes: vec!["".into()],
            extensions: vec!["".into()],
            max_threads: Default::default(),
            proxy_enabled: Default::default(),
            proxy_address: Default::default(),
            proxy_auth_enabled: Default::default(),
            ignore_cert: Default::default(),
            show_htaccess: Default::default(),
            throttle: Default::default(),
            max_recursion_depth: Default::default(),
            user_agent: Default::default(),
            username: Default::default(),
            password: Default::default(),
            output_file: Default::default(),
            json_file: Default::default(),
            xml_file: Default::default(),
            timeout: Default::default(),
            max_errors: Default::default(),
            wordlist_split: Default::default(),
            scan_listable: Default::default(),
            cookies: Default::default(),
            headers: Default::default(),
            scrape_listable: Default::default(),
            whitelist: Default::default(),
            code_list: Default::default(),
            is_terminal: Default::default(),
            no_color: Default::default(),
            disable_validator: Default::default(),
            http_verb: Default::default(),
            scan_opts: Default::default(),
            log_level: Info,
            length_blacklist: Default::default(),
        }
    }
}

impl Default for RequestResponse {
    fn default() -> Self {
        RequestResponse {
            url: "http://example.com/".into(),
            code: 200,
            content_len: 200,
            is_directory: false,
            is_listable: false,
            redirect_url: "".into(),
            found_from_listable: false,
            parent_depth: 0,
        }
    }
}
