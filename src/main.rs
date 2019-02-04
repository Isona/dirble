use std::{
    collections::VecDeque,
    sync::{Arc, mpsc::{self, Sender, Receiver}},
    thread,
    time::Duration,
};
extern crate curl;
use curl::easy::{Easy2};
mod arg_parse;
mod request;
mod wordlist;

fn main() {
    // Read the arguments in using the arg_parse module
    let global_opts = Arc::new(arg_parse::get_args());

    // Get the wordlist file from the arguments and open it
    let wordlist = Arc::new(wordlist::lines_from_file(global_opts.wordlist_file.clone()).unwrap());

    // Create a queue for URIs to be scanned
    let mut scan_queue: VecDeque<wordlist::UriGenerator> = VecDeque::new();


    // Push the host URI to the queue to be scanned
    for extension in global_opts.extensions.clone() {
        scan_queue.push_back(
            wordlist::UriGenerator::new(global_opts.hostname.clone(), String::from(extension), wordlist.clone()));
    }
    
    // Create a channel for threads to communicate with the parent on
    // This is used to send information about ending threads and discovered folders
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // Define the max number of threads and the number of threads currently in use
    let mut threads_in_use = 0;

    // Loop of checking for messages from the threads,
    // spawning new threads on items in the scan queue
    // and checking if the program is done
    loop {

        // Check for messages from the threads
        let reply = rx.try_recv();
        match reply {
            Ok(message) => {
                // If a thread has sent end, then we can reduce the threads in use count
                if message == "END" { threads_in_use -= 1; }
                // If a thread sent anything else, then it's a discovered directory
                // Create new generators with the folder and each extension, and push them to the scan queue
                else { 
                    for extension in global_opts.extensions.clone() {
                        scan_queue.push_back(
                            wordlist::UriGenerator::new(message.clone(), String::from(extension), wordlist.clone()));
                    }
                }
            },
            // Ignore any errors - this happens if the queue is empty, that's okay
            Err(_) => {},
        };

        // If there are items in the scan queue and available threads
        // Spawn a new thread to scan an item
        if threads_in_use < global_opts.max_threads && scan_queue.len() > 0 {

            // Clone a new sender to the channel and a new wordlist reference
            // Then pop the scan target from the queue
            let tx_clone = mpsc::Sender::clone(&tx);
            let list_gen = scan_queue.pop_front().unwrap();
            let arg_clone = global_opts.clone();
            // Spawn a thread with the arguments and increment the in use counter
            thread::spawn(|| thread_spawn(tx_clone, list_gen, arg_clone));
            threads_in_use += 1;
        }

        // If there are no threads in use and the queue is empty then stop
        if threads_in_use == 0 && scan_queue.len() == 0 {
            break;
        }

        // Sleep to reduce CPU cycles used by main
        thread::sleep(Duration::from_millis(1));
    }
}

fn thread_spawn(tx: mpsc::Sender<String>, uri_gen: wordlist::UriGenerator, global_opts: Arc<arg_parse::GlobalOpts>) {
    let hostname = uri_gen.hostname.clone();
    println!("Scanning {}/", hostname);
    // Create a new curl Easy2 instance and set it to use GET requests
    let mut easy = Easy2::new(request::Collector(Vec::new()));
    easy.get(true).unwrap();

    if global_opts.proxy_enabled {
        easy.proxy(&global_opts.proxy_address).unwrap();
    }

    // For each item in the wordlist, call the request function on it
    for uri in uri_gen {
        let code = request::make_request(&mut easy, uri.clone());
        match code {
            1 => {
                tx.send(uri).unwrap();
            },
            _ => {},
        }
    }
    println!("Finished scanning {}/", hostname);
    // Send an end message to the main thread
    tx.send(String::from("END")).unwrap();
}

