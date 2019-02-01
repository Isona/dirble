use std::{
    collections::VecDeque,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    sync::{Arc, mpsc::{self, Sender, Receiver}},
    thread,
    time::Duration,
};
extern crate curl;
use curl::easy::{Easy2, Handler, WriteError};
use percent_encoding::{utf8_percent_encode, percent_decode, DEFAULT_ENCODE_SET};
mod arg_parse;

struct Collector(Vec<u8>);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0 = data.to_vec();
        Ok(data.len())
    }
}


fn main() {
    // Load the yaml file containing argument definitions
    //let yml = load_yaml!("args.yml");
    //let m = App::from_yaml(yml).get_matches();
    let m = arg_parse::get_args();

    // Get the wordlist file from the arguments and open it
    let wordlist = Arc::new(lines_from_file(m.value_of("wordlist").unwrap()).unwrap());

    
    // Get the host URI from the arguments
    let hostname = String::from(m.value_of("host").unwrap().clone());

    // Create a queue for URIs to be scanned
    let mut scan_queue: VecDeque<String> = VecDeque::new();
    // Push the host URI to the queue to be scanned
    scan_queue.push_back(hostname);
    
    // Create a channel for threads to communicate with the parent on
    // This is used to send information about ending threads and discovered folders
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // Define the max number of threads and the number of threads currently in use
    let max_threads = 5;
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
                // Push it to the scan queue
                else { scan_queue.push_back(String::from(message)); }
            },
            // Ignore any errors - this happens if the queue is empty, that's okay
            Err(_) => {},
        };

        // If there are items in the scan queue and available threads
        // Spawn a new thread to scan an item
        if threads_in_use < max_threads && scan_queue.len() > 0 {

            // Clone a new sender to the channel and a new wordlist reference
            // Then pop the scan target from the queue
            let tx_clone = mpsc::Sender::clone(&tx);
            let wordlist_clone = wordlist.clone();
            let target = scan_queue.pop_front().unwrap();

            // Spawn a thread with the arguments and increment the in use counter
            thread::spawn(|| 
                thread_spawn(tx_clone, target, wordlist_clone));
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

// This function takes an instance of "Easy2", a base URL and a suffix
// It then makes the request, if the response was not a 404
// then it will print the URI it requested and the response
fn request(easy: &mut Easy2<Collector>, base: &str, end: &str) -> u32{

    //Concatenate and url encode the url, then set it in the Easy2 instance
    let url = format!("{}/{}", base, end);
    let url = utf8_percent_encode(&url, DEFAULT_ENCODE_SET).to_string();
    easy.url(&url).unwrap();

    //Perform the request and check if it's empty
    //If it's empty then output info and return
    match easy.perform() {
        Ok(_v) => {}
        Err(_e) => {   println!("- {} (CODE: 0|SIZE: 0)", url);
            return 0; 
        }
    }

    // Get the response code
    let code = easy.response_code().unwrap();
    // Get the contents of the response
    let contents = easy.get_ref();
    
    // Determine what to do based on the code
    match code {
        // If a 404 code, just return the response code
        404 => return code,
        // If it's a redirect, then check if the redirection destination is
        // to the same URL with a / on the end, if so return 1 so that the
        // calling thread knows this is a discovered folder
        // Otherwise, just print some output and return the response code
        301 | 302 => {
            let content_len = String::from_utf8_lossy(&contents.0).len();

            // Get and url decode the redirect destination
            let redir_dest = easy.redirect_url().unwrap().unwrap();
            let redir_dest = percent_decode(redir_dest.as_bytes()).decode_utf8().unwrap();

            // Clone and url decode the url
            let dir_url = url.clone() + "/";
            let dir_url = percent_decode(dir_url.as_bytes()).decode_utf8().unwrap();

            if dir_url == redir_dest {
                println!("==> DIRECTORY: {}", url);
                1
            }
            else {
                println!("+ {} (CODE: {}|SIZE:{:#?}|DEST:{})", url, code, content_len, redir_dest);
                code
            }
        },
        // If it's anything else, print out some information and return the response code
        _ => {
            let content_len = String::from_utf8_lossy(&contents.0).len();
            println!("+ {} (CODE:{}|SIZE:{:#?})", url, code, content_len); 
            code
        },
    }
}

fn thread_spawn(tx: mpsc::Sender<String>, hostname: String, wordlist: Arc<Vec<String>>) {
    println!("Scanning {}/", hostname);
    // Create a new curl Easy2 instance and set it to use GET requests
    let mut easy = Easy2::new(Collector(Vec::new()));
    easy.get(true).unwrap();

    // For each item in the wordlist, call the request function on it
    for line in 0..wordlist.len() {
        let code = request(&mut easy, &hostname, &wordlist[line]);
        match code {
            1 => {
                let dir_url = format!("{}/{}", &hostname, &wordlist[line]);
                tx.send(dir_url).unwrap();
            },
            _ => {},
        }
    }
    println!("Finished scanning {}/", hostname);
    // Send an end message to the main thread
    tx.send(String::from("END")).unwrap();
}

// Function used to read in lines from the wordlist file
fn lines_from_file<P>(filename: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    BufReader::new(File::open(filename)?).lines().collect()
}