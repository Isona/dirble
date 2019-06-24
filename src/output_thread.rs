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

use crate::arg_parse;
use std::{
    sync::{Arc, mpsc::self}
};
use crate::request;
use crate::output;
use simplelog::LevelFilter;

pub fn output_thread(rx: mpsc::Receiver<request::RequestResponse>,
    global_opts: Arc<arg_parse::GlobalOpts>, file_handles: output::FileHandles)
{
    let mut response_list: Vec<request::RequestResponse> = Vec::new();

    loop {
        if let Ok(response) = rx.try_recv() {
            if response.url == "MAIN ENDING" {
                break; 
            }
            if global_opts.log_level >= LevelFilter::Info {
                match output::print_response(&response, global_opts.clone(),
                    false, false, global_opts.is_terminal && !global_opts.no_color) {
                    Some(output) => { println!("{}", output) },
                    None => {}
                }
            }
            response_list.push(response);
        }
    }

    output::print_report(response_list, global_opts.clone(), file_handles);

}
