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

use crate::{arg_parse, output, request};
use log::debug;
use simplelog::LevelFilter;
use std::sync::{Arc, mpsc};
use url::Url;

pub fn output_thread(
    rx: mpsc::Receiver<request::RequestResponse>,
    global_opts: Arc<arg_parse::GlobalOpts>,
    file_handles: output::FileHandles,
) {
    let mut response_list: Vec<Vec<request::RequestResponse>> =
        vec![Vec::new(); global_opts.hostnames.len()];

    loop {
        if let Ok(response) = rx.try_recv() {
            if response.url == Url::parse("data:MAIN ENDING").unwrap() {
                debug!("Received signal to end, generating the report");
                break;
            }
            if global_opts.log_level >= LevelFilter::Info {
                if let Some(output) = output::print_response(
                    &response,
                    global_opts.clone(),
                    false,
                    false,
                    global_opts.is_terminal && !global_opts.no_color,
                ) {
                    println!("{}", output);
                }
            }
            response_list[response.parent_index].push(response);
        }
    }

    output::print_report(response_list, global_opts, file_handles);
}
