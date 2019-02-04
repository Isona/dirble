extern crate clap;
use clap::{App, Arg, AppSettings};

pub struct GlobalOpts {
    pub hostname: String,
    pub wordlist_file: String,
    pub extensions: Vec<String>,
    pub max_threads: u16,
    pub proxy_enabled: bool,
    pub proxy_address: String,
    pub proxy_auth_enabled: bool,
    pub proxy_username: String,
    pub proxy_password: String,   
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
                            .default_value("dirbcommon.txt"))
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
                            .value_name("burp")
                            .help("Sets the proxy to use the default burp proxy values (http://localhost:8080).")
                            .takes_value(false)
                            .conflicts_with("proxy"))
                        .arg(Arg::with_name("no_proxy")
                            .long("no-proxy")
                            .value_name("no-proxy")
                            .help("Disables proxy use even if there is a system proxy.")
                            .takes_value(false)
                            .conflicts_with("burp")
                            .conflicts_with("proxy"))
                        .get_matches();

    // Parse the extensions into a vector, then sort it and remove duplicates
    let mut extensions = vec![String::from("")];
    for extension in args.values_of("extensions").unwrap() {
        extensions.push(String::from(extension));
    }
    extensions.sort();
    extensions.dedup();

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

    // Create the GlobalOpts struct and return it
    GlobalOpts {
        hostname: String::from(args.value_of("host").unwrap().clone()),
        wordlist_file: String::from(args.value_of("wordlist").unwrap().clone()),
        extensions: extensions,
        max_threads: 5,
        proxy_enabled: proxy_enabled,
        proxy_address: proxy,
        proxy_auth_enabled: false,
        proxy_username: String::from(""),
        proxy_password: String::from(""),   
    }
}


fn starts_with_http(hostname: String) -> Result<(), String> {
    if hostname.starts_with("https://") || hostname.starts_with("http://") {
        Ok(())
    }
    else {
        Err(String::from("The provided target URI must start with http:// or https://"))
    }
}