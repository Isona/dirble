extern crate clap;
use clap::{App, Arg, AppSettings};

pub struct GlobalOpts {
    pub hostname: String,
    pub wordlist_file: String,
    pub extensions: Vec<String>,
    pub max_threads: u32,
    pub proxy_enabled: bool,
    pub proxy_address: String,
    pub proxy_auth_enabled: bool, 
    pub ignore_cert: bool,
    pub show_htaccess: bool,
    pub throttle: u32,
    pub disable_recursion: bool
}

pub fn get_args() -> GlobalOpts
{
    let args = App::new("dirble")
                        .version("0.1")
                        .author("Izzy Whistlecroft")
                        .about("Finds pages and folders on websites")
                        .setting(AppSettings::ArgRequiredElseHelp)
                        .arg(Arg::with_name("wordlist")
                            .short("w")
                            .long("wordlist")
                            .value_name("wordlist")
                            .help("Sets which wordlist to use")
                            .takes_value(true)
                            .default_value("dirbCommon.txt"))
                        .arg(Arg::with_name("host")
                            .short("t")
                            .long("target")
                            .value_name("host_uri")
                            .index(1)
                            .help("The URI of the host to scan")
                            .takes_value(true)
                            .required(true)
                            .validator(starts_with_http))
                        .arg(Arg::with_name("extensions")
                            .short("X")
                            .value_name("extensions")
                            .help("Provides comma separated extensions to extend queries with")
                            .min_values(1)
                            .default_value("")
                            .value_delimiter(","))
                        .arg(Arg::with_name("proxy")
                            .long("proxy")
                            .value_name("proxy")
                            .help("The proxy address to use, including type and port, \
                                     can also include a username and password in the form \
                                     \"http://username:password@proxy_url:proxy_port\"."))
                        .arg(Arg::with_name("burp")
                            .long("burp")
                            .help("Sets the proxy to use the default burp proxy values (http://localhost:8080).")
                            .takes_value(false)
                            .conflicts_with("proxy"))
                        .arg(Arg::with_name("no_proxy")
                            .long("no-proxy")
                            .help("Disables proxy use even if there is a system proxy.")
                            .takes_value(false)
                            .conflicts_with("burp")
                            .conflicts_with("proxy"))
                        .arg(Arg::with_name("max_threads")
                            .long("max-threads")
                            .value_name("max-threads")
                            .help("Sets the maximum number of request threads that will be spawned")
                            .takes_value(true)
                            .default_value("10")
                            .validator(max_thread_check))
                        .arg(Arg::with_name("ignore_cert")
                            .long("ignore-cert")
                            .short("k")
                            .help("Ignore the certificate validity for HTTPS."))
                        .arg(Arg::with_name("show_htaccess")
                            .long("show-htaccess")
                            .help("Enable display of .htaccess,.htaccess, and .htpasswd when they return 403 responses."))
                        .arg(Arg::with_name("throttle")
                            .short("z")
                            .long("throttle")
                            .help("Time each thread will wait between requests, given in milliseconds")
                            .value_name("milliseconds")
                            .validator(max_thread_check)
                            .takes_value(true))
                        .arg(Arg::with_name("disable_recursion")
                            .long("disable-recursion")
                            .short("r")
                            .help("Disable discovered subdirectory scanning"))
                        .get_matches();

    // Parse the extensions into a vector, then sort it and remove duplicates
    let mut extensions = vec![String::from("")];
    for extension in args.values_of("extensions").unwrap() {
        extensions.push(String::from(extension));
    }
    extensions.sort();
    extensions.dedup();

    // Check for proxy related flags
    let mut proxy_enabled = false;
    let mut proxy = "";
    if args.is_present("proxy") {
        proxy_enabled = true;
        proxy = args.value_of("proxy").unwrap();
        if proxy == "http://localhost:8080" {
            println!("You could use the --burp flag instead of the --proxy flag!");
        }
    }
    else if args.is_present("burp") {
        proxy_enabled = true;
        proxy = "http://localhost:8080";
    }
    else if args.is_present("no_proxy") {
        proxy_enabled = true;
        proxy = "";
    }
    let proxy = String::from(proxy);

    let mut throttle = 0;
    if args.is_present("throttle") {
        throttle = args.value_of("throttle").unwrap().parse::<u32>().unwrap();
    }

    // Create the GlobalOpts struct and return it
    GlobalOpts {
        hostname: String::from(args.value_of("host").unwrap().clone()),
        wordlist_file: String::from(args.value_of("wordlist").unwrap().clone()),
        extensions: extensions,
        max_threads: args.value_of("max_threads").unwrap().parse::<u32>().unwrap(),
        proxy_enabled: proxy_enabled,
        proxy_address: proxy,
        proxy_auth_enabled: false,   
        ignore_cert: args.is_present("ignore_cert"),
        show_htaccess: args.is_present("show_htaccess"),
        throttle: throttle,
        disable_recursion: args.is_present("disable_recursion")
    }
}

// Validator for the provided host name, ensures that the value begins with http:// or https://
fn starts_with_http(hostname: String) -> Result<(), String> {
    if hostname.starts_with("https://") || hostname.starts_with("http://") {
        Ok(())
    }
    else {
        Err(String::from("The provided target URI must start with http:// or https://"))
    }
}

// Validator for the number of threads provided in the --max-threads flag
// Ensures that the value is a positive integer
fn max_thread_check(value: String) -> Result<(), String> {
    let int_val = value.parse::<u32>();
    match int_val {
        Ok(max) => {
            if max > 0 {
                return Ok(())
            }
        },
        Err(_) => {},
    };
    return Err(String::from("The maximum number of threads must be a positive integer."))
}
