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
    sync::{Arc, mpsc::{self, Sender, Receiver}},
    thread,
    time::Duration,
};
extern crate curl;
mod arg_parse;
mod request;
mod wordlist;
mod output;
mod content_parse;
mod output_format;
mod request_thread;
mod output_thread;
mod target_validation_thread;

fn main() {
    // Read the arguments in using the arg_parse module
    let global_opts = Arc::new(arg_parse::get_args());

    output::startup_text(global_opts.clone());

    // Get the wordlist file from the arguments and open it
    let mut wordlist:Vec<String> = Vec::new();
    for wordlist_file in global_opts.wordlist_files.clone() {
        wordlist.append(&mut wordlist::lines_from_file(wordlist_file));
    }
    wordlist.sort();
    wordlist.dedup();
    
    let wordlist = Arc::new(wordlist);

    // Create a queue for URIs that need to be scanned
    let mut scan_queue: VecDeque<wordlist::UriGenerator> = VecDeque::new();

    // Push the host URI to the scan queue
    for hostname in &global_opts.hostnames {
        let mut depth = hostname.matches("/").count() as u32;
        if hostname.ends_with("/") {
            depth -= 1;
        }

        for prefix in &global_opts.prefixes {
            for extension in &global_opts.extensions {
                for start_index in 0..global_opts.wordlist_split {
                    scan_queue.push_back(
                        wordlist::UriGenerator::new(hostname.clone(), String::from(prefix.clone()),
                            String::from(extension.clone()), wordlist.clone(), 
                            start_index, global_opts.wordlist_split, depth));
                }
            }
        }
    }
    // Create a channel for threads to communicate with the parent on
    // This is used to send information about ending threads and information on responses
    let (output_tx, output_rx): (Sender<request::RequestResponse>, Receiver<request::RequestResponse>) = mpsc::channel();
    let (main_tx, main_rx): (Sender<request::RequestResponse>, Receiver<request::RequestResponse>) = mpsc::channel();

    // Define the max number of threads and the number of threads currently in use
    let mut threads_in_use = 0;

    let file_handles = output::create_files(global_opts.clone());
    let output_global_opts = global_opts.clone();

    let output_thread = thread::spawn(|| output_thread::output_thread(output_rx, output_global_opts, file_handles));    

    // Loop of checking for messages from the threads,
    // spawning new threads on items in the scan queue
    // and checking if the program is done
    loop {

        // Check for messages from the threads
        let reply = main_rx.try_recv();
        match reply {
            Ok(message) => {
                // If a thread has sent end, then we can reduce the threads in use count
                if message.url == "END" {
                    threads_in_use -= 1; }

                // If a thread sent anything else, then call the print_response function to deal with output
                // If the response was a directory, create generators with each extension and add it to the scan queue
                else { 
                    if message.is_directory && (!message.is_listable || global_opts.scan_listable) && !global_opts.disable_recursion {
                        for prefix in &global_opts.prefixes {
                            for extension in &global_opts.extensions {
                                for start_index in 0..global_opts.wordlist_split {
                                    scan_queue.push_back(
                                        wordlist::UriGenerator::new(message.url.clone(), String::from(prefix.clone()),
                                            String::from(extension.clone()), wordlist.clone(), 
                                            start_index, global_opts.wordlist_split, message.parent_depth));
                                }
                            }
                        }
                    }
                    else if message.is_listable && global_opts.verbose && !global_opts.scan_listable 
                    { println!("{} is listable, skipping scanning", message.url); }
                }
            },
            // Ignore any errors - this happens if the message queue is empty, that's okay
            Err(_) => {},
        };

        // If there are items in the scan queue and available threads
        // Spawn a new thread to scan an item
        if threads_in_use < global_opts.max_threads && scan_queue.len() > 0 {

            // Clone a new sender to the channel and a new wordlist reference
            // Then pop the scan target from the queue
            let main_tx_clone = mpsc::Sender::clone(&main_tx);
            let output_tx_clone = mpsc::Sender::clone(&output_tx);
            let list_gen = scan_queue.pop_front().unwrap();
            let arg_clone = global_opts.clone();

            // Spawn a thread with the arguments and increment the in use counter
            thread::spawn(|| request_thread::thread_spawn(main_tx_clone, output_tx_clone, list_gen, arg_clone));
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
    output_thread.join().unwrap();


}


fn generate_end() -> request::RequestResponse {
    request::RequestResponse {
        url: String::from("REPORT"),
        code: 0,
        content_len: 0,
        is_directory:false,
        is_listable: false,
        redirect_url: String::from(""),
        found_from_listable: false,
        parent_depth: 0
    }
}