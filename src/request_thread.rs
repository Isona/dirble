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

use crate::{arg_parse, request, validator_thread, wordlist};
use log::{debug, trace, warn};
use std::{
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};
use url::Url;

pub fn thread_spawn(
    dir_tx: mpsc::Sender<request::RequestResponse>,
    output_tx: mpsc::Sender<request::RequestResponse>,
    uri_gen: wordlist::UriGenerator,
    global_opts: Arc<arg_parse::GlobalOpts>,
) {
    let uri = uri_gen.base.clone();

    debug!("Scanning {}", uri);

    let mut easy = request::generate_easy(&global_opts);

    let mut consecutive_errors = 0;
    let parent_index = uri_gen.parent_index;
    let parent_depth = uri_gen.parent_depth;

    let validator = uri_gen.validator.clone();

    // For each item in the wordlist, call the request function on it
    // Then if there is a response send it to main
    for uri in uri_gen {
        let mut response = request::make_request(&mut easy, uri.clone());

        let code = response.code;

        // If the url is a directory, then check if it's listable
        // This may also scrape listable directories if the parameter is set
        // Then return each discovered item to the main thread
        if response.is_directory {
            let mut response_list = request::listable_check(
                &mut easy,
                response.url,
                global_opts.max_recursion_depth,
                response.parent_depth as i32,
                global_opts.scrape_listable,
            );

            let mut original_response = response_list.remove(0);
            original_response.found_from_listable = false;
            original_response.parent_index = parent_index;
            original_response.parent_depth = parent_depth;
            send_response(
                &dir_tx,
                &output_tx,
                &global_opts,
                original_response,
                &validator,
            );

            for mut scraped_response in response_list {
                scraped_response.parent_index = parent_index;
                scraped_response.parent_depth = parent_depth;
                send_response(
                    &dir_tx,
                    &output_tx,
                    &global_opts,
                    scraped_response,
                    &validator,
                );
            }
        }
        // If it isn't a directory then just send the response to the main thread
        else {
            response.parent_index = parent_index;
            response.parent_depth = parent_depth;
            send_response(
                &dir_tx,
                &output_tx,
                &global_opts,
                response,
                &validator,
            );
        }

        // Detect consecutive errors and stop the thread if the count is exceeded
        if global_opts.max_errors != 0 {
            if code == 0 {
                consecutive_errors += 1;
                if consecutive_errors >= global_opts.max_errors {
                    warn!(
                        "Thread scanning {} stopping due to multiple \
                         consecutive errors received",
                        uri
                    );
                    break;
                }
            } else {
                consecutive_errors = 0;
            }
        }

        // Sleep if throttle is set
        if global_opts.throttle != 0 {
            thread::sleep(Duration::from_millis(u64::from(
                global_opts.throttle,
            )));
        }
    }

    debug!("Finished scanning {}", uri);

    // Send a message to the main thread so it knows the thread is done
    dir_tx.send(generate_end()).unwrap();
}

// Sends the given RequestResponse to the main thread
// dependent on whitelist/blacklist settings and response code
#[inline]
fn send_response(
    dir_tx: &mpsc::Sender<request::RequestResponse>,
    output_tx: &mpsc::Sender<request::RequestResponse>,
    global_opts: &arg_parse::GlobalOpts,
    response: request::RequestResponse,
    validator_opt: &Option<validator_thread::TargetValidator>,
) {
    if response.is_directory {
        dir_tx.send(response.clone()).unwrap();
        output_tx.send(response).unwrap();
        return;
    }
    if should_send_response(global_opts, &response, validator_opt) {
        output_tx.send(response).unwrap();
    }
}

#[inline]
pub fn should_send_response(
    global_opts: &arg_parse::GlobalOpts,
    response: &request::RequestResponse,
    validator_opt: &Option<validator_thread::TargetValidator>,
) -> bool {
    // Check each of the conditions for outputting the discovered file

    // Check the response code white/blacklist
    let contains_code = global_opts.code_list.contains(&response.code);
    if global_opts.whitelist && !contains_code {
        trace!(
            "[{}]: code {} not in whitelist",
            response.url, response.code
        );
        return false;
    }
    if !global_opts.whitelist && contains_code {
        trace!("[{}]: code {} in blacklist", response.url, response.code);
        return false;
    }
    if response.code == 0 {
        trace!("[{}]: code 0 detected", response.url);
        return false;
    }
    if let Some(validator) = validator_opt {
        if validator.is_not_found(response) {
            trace!("[{}]: matches Not Found condition", response.url);
            return false;
        }
    }

    // Check that the response size has not been blacklisted
    if global_opts.length_blacklist.contains(response.content_len) {
        trace!(
            "[{}]: length {} is in a blacklist range",
            response.url, response.content_len
        );
        return false;
    }

    // Return the response for outputting
    true
}

#[inline]
fn generate_end() -> request::RequestResponse {
    request::RequestResponse {
        url: Url::parse("data:END").unwrap(),
        code: 0,
        content_len: 0,
        is_directory: false,
        is_listable: false,
        redirect_url: String::from(""),
        found_from_listable: false,
        parent_index: 0,
        parent_depth: 0,
    }
}

#[cfg(test)]
mod test {

    use crate::{
        arg_parse::{GlobalOpts, LengthRange, LengthRanges},
        request::RequestResponse,
        request_thread::should_send_response,
        validator_thread::TargetValidator,
    };

    #[test]
    fn test_should_send_response() {
        let mut globalopts: GlobalOpts = Default::default();
        let mut rr: RequestResponse = Default::default();

        // Response code is in blacklist -> false
        globalopts.whitelist = false;
        globalopts.code_list = vec![200, 201];
        rr.code = 200;
        assert!(
            !should_send_response(&globalopts, &rr, &None),
            "Code in blacklist failed"
        );

        // Response code is not in blacklist -> true
        rr.code = 300;
        assert!(
            should_send_response(&globalopts, &rr, &None),
            "Code not in blacklist failed"
        );

        // Response code is in whitelist -> true
        globalopts.whitelist = true;
        rr.code = 200;
        assert!(
            should_send_response(&globalopts, &rr, &None),
            "Code in whitelist failed"
        );

        // Response code is not in whitelist -> false
        rr.code = 301;
        assert!(
            !should_send_response(&globalopts, &rr, &None),
            "Code not in whitelist failed"
        );

        // Response matches Not Found condition -> false
        globalopts.whitelist = false;
        let val = TargetValidator::new(301, None, None, None, None);
        assert!(
            !should_send_response(&globalopts, &rr, &Some(val)),
            "Not Found response failed"
        );

        // Response length exactly matches a blacklist item -> false
        rr.content_len = 500;
        globalopts.length_blacklist = LengthRanges {
            ranges: vec![
                LengthRange {
                    start: 5000,
                    end: Some(6000),
                },
                LengthRange {
                    start: 500,
                    end: None,
                },
            ],
        };
        assert!(
            !should_send_response(&globalopts, &rr, &None),
            "Length matches blacklist failed"
        );

        // Response length is within a blacklist range -> false
        rr.content_len = 5300;
        assert!(
            !should_send_response(&globalopts, &rr, &None),
            "Length within blacklist range failed"
        );

        // Response length_is outside of the blacklist ranges -> true
        rr.content_len = 5;
        assert!(
            should_send_response(&globalopts, &rr, &None),
            "Length outside blacklist range failed"
        );
    }
}
