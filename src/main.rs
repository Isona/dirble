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

use std::{
    collections::VecDeque,
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
        mpsc::{
            self,
            Sender,
            Receiver,
        },
    },
    thread,
    time::Duration,
    path::Path,
    env::current_exe,
};
use simplelog::{
    SimpleLogger,
    TermLogger,
};
use log::Level;
#[allow(unused)] use log::{
    trace,
    debug,
    info,
    warn,
    error,
};
use ctrlc;
#[macro_use]
extern crate clap;
extern crate curl;
mod arg_parse;
mod request;
mod wordlist;
mod output;
mod content_parse;
mod output_format;
mod request_thread;
mod output_thread;
mod validator_thread;
#[cfg(test)] mod tests;

fn main() {
    // Read the arguments in using the arg_parse module
    let global_opts = Arc::new(arg_parse::get_args());

    // Prepare the logging handler. Default to a pretty TermLogger,
    // but if the TermLogger initialisation fails (e.g. if we are not
    // connected to a TTY) then set up a SimpleLogger instead.
    let log_config = simplelog::Config {
        time: Some(Level::Debug),
        level: Some(Level::Error),
        target: None,
        location: None,
        time_format: Some("%T"),
    };
    match TermLogger::init(
        global_opts.log_level,
        log_config,
        ) {
        Err(_) => {
            SimpleLogger::init(
                global_opts.log_level,
                log_config,
                ).unwrap();
        },
        Ok(_) => (),
    }

    // Get the wordlist file from the arguments. If it has not been set
    // then try the default wordlist locations.
    let mut wordlist:Vec<String> = Vec::new();
    let wordlist_string: String;
    if let Some(wordlist_files) = global_opts.wordlist_files.clone() {
        // A wordlist has been set in the global opts
        for wordlist_file in wordlist_files {
            wordlist.append(&mut wordlist::lines_from_file(&wordlist_file));
        }
        wordlist_string = "".into();
    }
    else {
        // Otherwise try the directory containing the exe, then
        // /usr/share/dirble, then /usr/share/wordlists, then finally
        // /usr/share/wordlists/dirble before giving up.
        let mut exe_path = current_exe()
            .unwrap_or_else(|error| {
                println!("Getting directory of exe failed: {}", error);
                std::process::exit(2);
            });
        exe_path.set_file_name("dirble_wordlist.txt");
        let usr_share_dirble = Path::new(
            "/usr/share/dirble/dirble_wordlist.txt");
        let usr_share_wordlists = Path::new(
            "/usr/share/wordlists/dirble_wordlist.txt");
        let usr_share_wordlists_dirble = Path::new(
            "/usr/share/wordlists/dirble/dirble_wordlist.txt");

        debug!("Checking for wordlist in:\n - {}\n - {}\n - {}\n - {}",
               exe_path.to_str().unwrap(),
               usr_share_dirble.to_str().unwrap(),
               usr_share_wordlists.to_str().unwrap(),
               usr_share_wordlists_dirble.to_str().unwrap(),
               );
        let wordlist_file = if exe_path.exists() {
            // Prioritise the wordlist in the same directory as the exe
            String::from(exe_path.to_str().unwrap())
        }
        else if usr_share_dirble.exists() {
            String::from(usr_share_dirble.to_str().unwrap())
        }
        else if usr_share_wordlists.exists() {
            String::from(usr_share_wordlists.to_str().unwrap())
        }
        else {
            error!("Unable to find default wordlist");
            std::process::exit(1);
        };
        wordlist.append(&mut wordlist::lines_from_file(&wordlist_file));
        wordlist_string = wordlist_file;
    }

    output::startup_text(global_opts.clone(), &wordlist_string);

    // Remove leading and trailing slashes from words
    for i in 0..wordlist.len() {
        if wordlist[i].starts_with("/") {
            wordlist[i].remove(0);
        }

        if wordlist[i].ends_with("/") {
            wordlist[i].pop();
        }
    }

    wordlist.sort();
    wordlist.dedup();

    let wordlist = Arc::new(wordlist);


    // Create a channel for threads to communicate with the parent on
    // This is used to send information about ending threads and information on responses
    let (output_tx, output_rx): (Sender<request::RequestResponse>, Receiver<request::RequestResponse>) = mpsc::channel();
    let (to_validate_tx, to_validate_rx): (Sender<request::RequestResponse>, Receiver<request::RequestResponse>) = mpsc::channel();
    let (to_scan_tx, to_scan_rx): (Sender<Option<validator_thread::DirectoryInfo>>, Receiver<Option<validator_thread::DirectoryInfo>>) = mpsc::channel();

    let validator_global_opts = global_opts.clone(); 
    let validator_thread = thread::spawn(|| 
        validator_thread::validator_thread(to_validate_rx, to_scan_tx, validator_global_opts));


    for hostname in &global_opts.hostnames {
        let mut request = request::fabricate_request_response(hostname.clone(), true, false);
        let mut depth = hostname.matches("/").count() as u32;
        if hostname.ends_with("/") {
            depth -= 1;
        }
        request.parent_depth = depth;
        to_validate_tx.send(request).unwrap();
    }


    // Create a queue for URIs that need to be scanned
    let mut scan_queue: VecDeque<wordlist::UriGenerator> = VecDeque::new();

    // Push the host URI to the scan queue
    for _i in 0..global_opts.hostnames.len() {
        let response = to_scan_rx.recv().unwrap();

        match response {
            None => continue,
            Some(dir_info) => {
                match &dir_info.validator {
                    Some(validator) => {
                        if validator.scan_folder(&global_opts.scan_opts) {
                            add_dir_to_scan_queue(
                                &mut scan_queue,
                                &global_opts,
                                &dir_info,
                                &wordlist,
                                true,
                                );
                        }
                        else {
                            info!("Skipping {}{}", dir_info.url, &validator.print_alert())
                        }
                    }
                    // If there is no validator, then scan the folder
                    None => {
                        add_dir_to_scan_queue(
                            &mut scan_queue,
                            &global_opts,
                            &dir_info,
                            &wordlist,
                            true,
                            );
                    }
                }
            }
        }

    }
    // Define the max number of threads and the number of threads currently in use
    let mut threads_in_use = 0;

    let file_handles = output::create_files(global_opts.clone());
    let output_global_opts = global_opts.clone();

    let output_thread = thread::spawn(|| output_thread::output_thread(output_rx, output_global_opts, file_handles));

    let caught_ctrl_c = Arc::new(AtomicBool::new(false));
    let caught_ctrl_c_clone_for_handler = caught_ctrl_c.clone();
    ctrlc::set_handler(move || {
        warn!("Caught interrupt signal, cleaning up...");
        caught_ctrl_c_clone_for_handler.store(true, Ordering::SeqCst);
    }).expect("Unable to attach interrupt signal handler");

    // Loop of checking for messages from the threads,
    // spawning new threads on items in the scan queue
    // and checking if the program is done
    while !caught_ctrl_c.load(Ordering::SeqCst) {

        // Check for messages from the threads
        let to_scan = to_scan_rx.try_recv();

        // Ignore any errors - this happens if the message queue is empty, that's okay
        if let Ok(dir_info_opt) = to_scan {
            if let Some(dir_info) = dir_info_opt {
                // If a thread has sent end, then we can reduce the threads in use count
                if dir_info.url == "END" {
                    threads_in_use -= 1; }

                // Check the validator to see if the directory should be scanned
                else { 
                    match &dir_info.validator {
                        Some(validator) => {
                            if validator.scan_folder(&global_opts.scan_opts){
                                add_dir_to_scan_queue(
                                    &mut scan_queue,
                                    &global_opts,
                                    &dir_info,
                                    &wordlist,
                                    false,
                                    );
                            }
                            else {
                                info!("Skipping {}{}", dir_info.url, &validator.print_alert())
                            }
                        }
                        // If there is no validator, then scan the folder
                        None => {
                            add_dir_to_scan_queue(
                                &mut scan_queue,
                                &global_opts,
                                &dir_info,
                                &wordlist,
                                false,
                                );
                        }
                    }
                }
            }
        };

        // If there are items in the scan queue and available threads
        // Spawn a new thread to scan an item
        if threads_in_use < global_opts.max_threads && scan_queue.len() > 0 {

            // Clone a new sender to the channel and a new wordlist reference
            // Then pop the scan target from the queue
            let to_validate_tx_clone = mpsc::Sender::clone(&to_validate_tx);
            let output_tx_clone = mpsc::Sender::clone(&output_tx);
            let list_gen = scan_queue.pop_front().unwrap();
            let arg_clone = global_opts.clone();

            // Spawn a thread with the arguments and increment the in use counter
            thread::spawn(|| request_thread::thread_spawn(to_validate_tx_clone, output_tx_clone, list_gen, arg_clone));
            threads_in_use += 1;
        }

        // If there are no threads in use and the queue is empty then stop
        if threads_in_use == 0 && scan_queue.len() == 0 {
            break;
        }

        // Sleep to reduce CPU cycles used by main
        thread::sleep(Duration::from_millis(1));
    }

    // loop to check that report printing has ended
    output_tx.send(generate_end()).unwrap();
    to_validate_tx.send(generate_end()).unwrap();
    output_thread.join().unwrap();
    validator_thread.join().unwrap();
}

#[inline]
fn add_dir_to_scan_queue(
    scan_queue: &mut VecDeque<wordlist::UriGenerator>,
    global_opts: &Arc<arg_parse::GlobalOpts>,
    dir_info: &validator_thread::DirectoryInfo,
    wordlist: &Arc<Vec<String>>,
    first_run: bool) {

    // first_run is true when the initial scans are being initialised
    // on the base paths. We override the default wordlist_split to
    // improve performance of the initial discovery phase.
    let num_hosts = global_opts.hostnames.len() as u32;
    let wordlist_split;
    if first_run && (global_opts.wordlist_split * num_hosts) <
        (global_opts.max_threads - 2) {
            // If there's enough headroom to boost the split then do so
            wordlist_split = (global_opts.max_threads - 2) / num_hosts;
            info!(
                "Increasing wordlist-split for initial scan of {} to {}",
                dir_info.url,
                wordlist_split);
        }
    else {
        wordlist_split = global_opts.wordlist_split;
    }

    for prefix in &global_opts.prefixes {
        for extension in &global_opts.extensions {
            for start_index in 0..wordlist_split {
                scan_queue.push_back(
                    wordlist::UriGenerator::new(
                        dir_info.url.clone(),
                        String::from(prefix.clone()),
                        String::from(extension.clone()),
                        wordlist.clone(),
                        start_index,
                        wordlist_split,
                        dir_info.parent_depth,
                        dir_info.validator.clone()
                    )
                );
            }
        }
    }
}

fn generate_end() -> request::RequestResponse {
    request::RequestResponse {
        url: String::from("MAIN ENDING"),
        code: 0,
        content_len: 0,
        is_directory:false,
        is_listable: false,
        redirect_url: String::from(""),
        found_from_listable: false,
        parent_depth: 0
    }
}
