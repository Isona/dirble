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
    let (tx, rx): (Sender<request::RequestResponse>, Receiver<request::RequestResponse>) = mpsc::channel();

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
                if message.url == "END" {
                    threads_in_use -= 1; }
                // If a thread sent anything else, then it's a discovered directory
                // Create new generators with the folder and each extension, and push them to the scan queue
                else { 
                    match message.code {
                        403 => {
                            if !global_opts.show_htaccess && ( message.url.ends_with("/.htaccess") || message.url.ends_with("/.hta") 
                                || message.url.ends_with("/.htpasswd") ) { }
                            else {
                            println!("+ {} (CODE:{}|SIZE:{:#?})", message.url, message.code, message.content_len); 
                            }
                        }
                        301 | 302 => {
                            if message.is_directory {
                                println!("==> DIRECTORY: {}", message.url);
                                for extension in global_opts.extensions.clone() {
                                    scan_queue.push_back(
                                        wordlist::UriGenerator::new(message.url.clone(), String::from(extension), wordlist.clone()));
                                }
                            }
                            else {
                                println!("+ {} (CODE: {}|SIZE:{:#?}|DEST:{})", 
                                    message.url, message.code, message.content_len, message.redirect_url);
                            }
                        }
                        _ => {
                            println!("+ {} (CODE:{}|SIZE:{:#?})", message.url, message.code, message.content_len); 
                        }

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

fn thread_spawn(tx: mpsc::Sender<request::RequestResponse>, uri_gen: wordlist::UriGenerator, global_opts: Arc<arg_parse::GlobalOpts>) {
    let hostname = uri_gen.hostname.clone();
    println!("Scanning {}/", hostname);
    // Create a new curl Easy2 instance and set it to use GET requests
    let mut easy = Easy2::new(request::Collector(Vec::new()));
    easy.get(true).unwrap();

    if global_opts.proxy_enabled {
        easy.proxy(&global_opts.proxy_address).unwrap();
    }

    if global_opts.ignore_cert {
        easy.ssl_verify_host(false).unwrap();
        easy.ssl_verify_peer(false).unwrap();
    }

    // For each item in the wordlist, call the request function on it
    for uri in uri_gen {
        let req_response = request::make_request(&mut easy, uri.clone());

        match req_response{
            Some(response) => { tx.send(response).unwrap(); }
            None => {}
        }
    }
    println!("Finished scanning {}/", hostname);
    // Send an end message to the main thread

    let end = request::RequestResponse {
        url: String::from("END"),
        code: 0,
        content_len: 0,
        is_directory:false,
        is_listable: false,
        redirect_url: String::from("")
    };
    tx.send(end).unwrap();
}

