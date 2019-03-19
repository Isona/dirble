extern crate clap;
use clap::{App, Arg, AppSettings};
use crate::wordlist::lines_from_file;

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
    pub disable_recursion: bool,
    pub user_agent: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub output_file: Option<String>,
    pub verbose: bool,
    pub silent: bool,
    pub timeout: u32,
    pub max_errors: u32,
    pub wordlist_split: u32,
    pub scan_listable: bool,
    pub cookies: Option<String>,
    pub headers: Option<Vec<String>>,
    pub scrape_listable: bool
}

pub fn get_args() -> GlobalOpts
{
    // Defines all the command line arguments with the Clap module
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
                            .help("The URI of the host to scan, optionally supports basic auth with http://user:pass@host:port")
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
                        .arg(Arg::with_name("extension_file")
                            .short("x")
                            .long("extension-file")
                            .value_name("extension-file")
                            .help("The name of a file containing extensions to extend queries with, one per line"))
                        .arg(Arg::with_name("proxy")
                            .long("proxy")
                            .value_name("proxy")
                            .help("The proxy address to use, including type and port, \
                                     can also include a username and password in the form \
                                     \"http://username:password@proxy_url:proxy_port\""))
                        .arg(Arg::with_name("burp")
                            .long("burp")
                            .help("Sets the proxy to use the default burp proxy values (http://localhost:8080)")
                            .takes_value(false)
                            .conflicts_with("proxy"))
                        .arg(Arg::with_name("no_proxy")
                            .long("no-proxy")
                            .help("Disables proxy use even if there is a system proxy")
                            .takes_value(false)
                            .conflicts_with("burp")
                            .conflicts_with("proxy"))
                        .arg(Arg::with_name("max_threads")
                            .long("max-threads")
                            .value_name("max-threads")
                            .help("Sets the maximum number of request threads that will be spawned")
                            .takes_value(true)
                            .default_value("10")
                            .validator(positive_int_check))
                        .arg(Arg::with_name("ignore_cert")
                            .long("ignore-cert")
                            .short("k")
                            .help("Ignore the certificate validity for HTTPS"))
                        .arg(Arg::with_name("show_htaccess")
                            .long("show-htaccess")
                            .help("Enable display of items containing .ht when they return 403 responses"))
                        .arg(Arg::with_name("throttle")
                            .short("z")
                            .long("throttle")
                            .help("Time each thread will wait between requests, given in milliseconds")
                            .value_name("milliseconds")
                            .validator(positive_int_check)
                            .takes_value(true))
                        .arg(Arg::with_name("disable_recursion")
                            .long("disable-recursion")
                            .short("r")
                            .help("Disable discovered subdirectory scanning"))
                        .arg(Arg::with_name("user_agent")
                            .long("user-agent")
                            .short("a")
                            .help("Set the user-agent provided with requests, by default it isn't set")
                            .takes_value(true))
                        .arg(Arg::with_name("username")
                            .long("username")
                            .help("Sets the username to authenticate with")
                            .takes_value(true)
                            .requires("password"))
                        .arg(Arg::with_name("password")
                            .long("password")
                            .help("Sets the password to authenticate with")
                            .takes_value(true)
                            .requires("username"))
                        .arg(Arg::with_name("output_file")
                            .short("o")
                            .long("output-file")
                            .visible_alias("oN")
                            .help("Sets the file to write the report to")
                            .takes_value(true))
                        .arg(Arg::with_name("verbose")
                            .long("verbose")
                            .short("v")
                            .help("Print information when a thread starts and finishes scanning")
                            .takes_value(false))
                        .arg(Arg::with_name("silent")
                            .long("silent")
                            .short("S")
                            .help("Don't output information during the scan, only output the report at the end")
                            .takes_value(false))
                        .arg(Arg::with_name("timeout")
                            .long("timeout")
                            .help("Maximum time to wait for a response before giving up, given in seconds")
                            .validator(positive_int_check)
                            .default_value("5"))
                        .arg(Arg::with_name("max_errors")
                            .long("max-errors")
                            .help("The number of consecutive errors a thread can have before it exits, set to 0 to disable")
                            .validator(int_check)
                            .default_value("5"))
                        .arg(Arg::with_name("wordlist_split")
                            .long("wordlist-split")
                            .help("The number of threads to run for each folder/extension combo")
                            .validator(positive_int_check)
                            .default_value("3"))
                        .arg(Arg::with_name("scan_listable")
                            .long("scan-listable")
                            .short("l")
                            .help("Scan listable directories")
                            .takes_value(false))
                        .arg(Arg::with_name("cookie")
                            .long("cookie")
                            .short("c")
                            .help("Provide a cookie in the form \"name=value\", can be used multiple times")
                            .multiple(true)
                            .takes_value(true))
                        .arg(Arg::with_name("header")
                            .long("header")
                            .short("H")
                            .help("Provide an arbitrary header in the form \"header:value\" - headers with no value must end in a semicolon")
                            .multiple(true)
                            .takes_value(true))
                        .arg(Arg::with_name("scrape_listable")
                            .long("scrape-listable")
                            .help("Enable scraping of listable directories for urls, often produces large amounts of output")
                            .takes_value(false))
                        .get_matches();

    // Parse the extensions into a vector, then sort it and remove duplicates
    let mut extensions = vec![String::from("")];
    for extension in args.values_of("extensions").unwrap() {
        extensions.push(String::from(extension));
    }

    // Read in extensions from a file
    if args.is_present("extension_file") {
        let extensions_file = String::from(args.value_of("extension_file").unwrap());
        let extensions_from_file = lines_from_file(extensions_file).unwrap();
        for extension in extensions_from_file {
            extensions.push(String::from(extension));
        }
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

    // Reads in how long each thread should wait after each request
    let mut throttle = 0;
    if args.is_present("throttle") {
        throttle = args.value_of("throttle").unwrap().parse::<u32>().unwrap();
    }

    // Read user agent from arguments
    let mut user_agent = None;
    if args.is_present("user_agent") {
        user_agent = Some(String::from(args.value_of("user_agent").unwrap()));
    }

    // Get http basic auth related arguments
    let mut username = None;
    let mut password = None;
    if args.is_present("username") {
        username = Some(String::from(args.value_of("username").unwrap()));
        password = Some(String::from(args.value_of("password").unwrap()));
    }

    // Read the name of the output file if provided
    let mut output_file = None;
    if args.is_present("output_file") {
        output_file = Some(String::from(args.value_of("output_file").unwrap()));
    }

    // Read provided cookie values into a vector
    let mut cookies = None;
    if args.is_present("cookie") {
        let mut temp_cookies: Vec<String> = Vec::new();
        for cookie in args.values_of("cookie").unwrap() {
            temp_cookies.push(String::from(cookie));
        }
        
        cookies = Some(temp_cookies.join("; "));
    }

    // Read provided headers into a vector
    let mut headers = None;
    if args.is_present("header") {
        let mut temp_headers: Vec<String> = Vec::new();
        for header in args.values_of("header").unwrap() {
            temp_headers.push(String::from(header));
        }
        headers = Some(temp_headers);
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
        disable_recursion: args.is_present("disable_recursion"),
        user_agent: user_agent,
        username: username,
        password: password,
        output_file: output_file,
        verbose: args.is_present("verbose"),
        silent: args.is_present("silent"),
        timeout: args.value_of("timeout").unwrap().parse::<u32>().unwrap(),
        max_errors: args.value_of("max_errors").unwrap().parse::<u32>().unwrap(),
        wordlist_split: args.value_of("wordlist_split").unwrap().parse::<u32>().unwrap(),
        scan_listable: args.is_present("scan_listable"),
        cookies: cookies,
        headers: headers,
        scrape_listable:args.is_present("scrape_listable")
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

// Validator for arguments including the --max-threads flag
// Ensures that the value is a positive integer (not 0)
fn positive_int_check(value: String) -> Result<(), String> {
    let int_val = value.parse::<u32>();
    match int_val {
        Ok(max) => {
            if max > 0 {
                return Ok(())
            }
        },
        Err(_) => {},
    };
    return Err(String::from("The number given must be a positive integer."))
}

// Validator for various arguments, ensures that value is a
// positive integer, including 0
fn int_check(value: String) -> Result<(), String> {
    let int_val = value.parse::<u32>();
    match int_val {
        Ok(_) => {
            return Ok(())
        },
        Err(_) => {},
    };
    return Err(String::from("The number given must be an integer."))
}
