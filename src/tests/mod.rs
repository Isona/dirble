// This file is part of Dirble - https://www.github.com/nccgroup/dirble
// Copyright (C) 2019
//  * Izzy Whistlecroft <Izzy(dot)Whistlecroft(at)nccgroup(dot)com>
//  * David Young <David(dot)Young(at)nccgroup(dot)com>
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

use crate::arg_parse::GlobalOpts;
use crate::request::RequestResponse;
use log::LevelFilter::Info;
use url::Url;

mod argparse;
mod integration;
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
            extension_substitution: false,
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
            url: Url::parse("http://example.com/").unwrap(),
            code: 200,
            content_len: 200,
            is_directory: false,
            is_listable: false,
            redirect_url: "".into(),
            found_from_listable: false,
            parent_index: 0,
            parent_depth: 0,
        }
    }
}
