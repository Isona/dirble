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
extern crate clap;
use std::process::exit;
use std::env::current_exe;
use clap::{App, Arg, AppSettings, ArgGroup, crate_version};
use crate::wordlist::lines_from_file;
use atty::Stream;

pub struct GlobalOpts {
    pub hostnames: Vec<String>,
    pub wordlist_files: Vec<String>,
    pub prefixes: Vec<String>,
    pub extensions: Vec<String>,
    pub max_threads: u32,
    pub proxy_enabled: bool,
    pub proxy_address: String,
    pub proxy_auth_enabled: bool, 
    pub ignore_cert: bool,
    pub show_htaccess: bool,
    pub throttle: u32,
    pub max_recursion_depth: Option<i32>,
    pub user_agent: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub output_file: Option<String>,
    pub json_file: Option<String>,
    pub xml_file: Option<String>,
    pub verbose: bool,
    pub silent: bool,
    pub timeout: u32,
    pub max_errors: u32,
    pub wordlist_split: u32,
    pub scan_listable: bool,
    pub cookies: Option<String>,
    pub headers: Option<Vec<String>>,
    pub scrape_listable: bool,
    pub whitelist: bool,
    pub code_list: Vec<u32>,
    pub is_terminal: bool,
    pub no_color:bool,
    pub disable_validator:bool,
    pub http_verb:HttpVerb,
    pub scan_opts: ScanOpts
}

pub struct ScanOpts {
    pub scan_401: bool,
    pub scan_403: bool
}

arg_enum!{
    pub enum HttpVerb {
        Get,
        Head,
        Post
    }
}

pub fn get_args() -> GlobalOpts
{
    // Defines all the command line arguments with the Clap module
    let args = App::new("Dirble")
        .version(crate_version!())
        .author("Developed by Izzy Whistlecroft <Izzy(dot)Whistlecroft(at)nccgroup(dot).com>")
        .about("Fast directory scanning and scraping tool")
        .after_help("OUTPUT FORMAT:
    + [url] - File
    D [url] - Directory
    L [url] - Listable Directory\n
EXAMPLE USE:
    - Run against a website using the default dirble_wordlist.txt from the
      current directory:
        dirble [address]\n
    - Run with a different wordlist and including .php and .html extensions:
        dirble [address] -w example_wordlist.txt -x .php,.html\n
    - With listable directory scraping enabled:
        dirble [address] --scrape-listable\n
    - Providing a list of extensions and a list of URIs:
        dirble [address] -X wordlists/web.lst -U uri-list.txt\n
    - Providing multiple hosts to scan via command line:
        dirble [address] -u [address] -u [address]")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("host")
             .display_order(10)
             .help(
"The URI of the host to scan, optionally supports basic auth with
http://user:pass@host:port")
             .index(1)
             .next_line_help(true)
             .takes_value(true)
             .validator(starts_with_http)
             .value_name("uri"))
        .arg(Arg::with_name("extra_hosts")
             .alias("host")
             .display_order(10)
             .help(
"Additional hosts to scan")
             .long("uri")
             .multiple(true)
             .next_line_help(true)
             .short("u")
             .takes_value(true)
             .validator(starts_with_http)
             .value_name("uri")
             .visible_alias("url"))
        .arg(Arg::with_name("host_file")
             .alias("host-file")
             .display_order(10)
             .help(
"The filename of a file containing a list of URIs to scan - cookies and
headers set will be applied to all URIs")
             .long("uri-file")
             .multiple(true)
             .next_line_help(true)
             .short("U")
             .takes_value(true)
             .value_name("uri-file")
             .visible_alias("url-file"))
        .group(ArgGroup::with_name("hosts")
               .args(&["host", "host_file", "extra_hosts"])
               .multiple(true)
               .required(true))
        .arg(Arg::with_name("http_verb")
             .default_value("Get")
             .display_order(11)
             .help(
"Specify which HTTP verb to use
") // Newline is needed for the enumeration of possible values
             .long("verb")
             .next_line_help(true)
             .possible_values(&HttpVerb::variants())
             .takes_value(true))
        .arg(Arg::with_name("wordlist")
             .display_order(20)
             .help(
"Sets which wordlist to use, defaults to dirble_wordlist.txt in the same
folder as the executable")
             .long("wordlist")
             .multiple(true)
             .next_line_help(true)
             .short("w")
             .takes_value(true)
             .value_name("wordlist"))
        .arg(Arg::with_name("extensions")
             .display_order(30)
             .help(
"Provides comma separated extensions to extend queries with")
             .long("extensions")
             .min_values(1)
             .multiple(true)
             .next_line_help(true)
             .short("x")
             .value_delimiter(",")
             .value_name("extensions"))
        .arg(Arg::with_name("extension_file")
             .display_order(30)
             .help(
"The name of a file containing extensions to extend queries with, one
per line")
             .long("extension-file")
             .multiple(true)
             .next_line_help(true)
             .short("X")
             .value_name("extension-file"))
        .arg(Arg::with_name("prefixes")
             .display_order(30)
             .help(
"Provides comma separated prefixes to extend queries with")
             .long("prefixes")
             .min_values(1)
             .multiple(true)
             .next_line_help(true)
             .short("p")
             .value_delimiter(","))
        .arg(Arg::with_name("prefix_file")
             .display_order(30)
             .help(
"The name of a file containing extensions to extend queries with, one
per line")
             .long("prefix-file")
             .multiple(true)
             .next_line_help(true)
             .short("P")
             .value_name("prefix-file"))
        .arg(Arg::with_name("output_file")
             .display_order(40)
             .help(
"Sets the file to write the report to")
             .long("output-file")
             .next_line_help(true)
             .short("o")
             .takes_value(true)
             .visible_alias("oN"))
        .arg(Arg::with_name("json_file")
             .display_order(40)
             .help(
"Sets a file to write JSON output to")
             .long("json-file")
             .next_line_help(true)
             .takes_value(true)
             .visible_alias("oJ"))
        .arg(Arg::with_name("xml_file")
             .display_order(40)
             .help(
"Sets a file to write XML output to")
             .long("xml-file")
             .next_line_help(true)
             .takes_value(true)
             .visible_alias("oX"))
        .arg(Arg::with_name("output_all")
             .display_order(41)
             .help(
"Stores all output types respectively as .txt, .json and .xml")
             .long("output-all")
             .next_line_help(true)
             .takes_value(true)
             .visible_alias("oA"))
        .arg(Arg::with_name("proxy")
             .display_order(50)
             .help(
"The proxy address to use, including type and port, can also include a
username and password in the form 
\"http://username:password@proxy_url:proxy_port\"")
             .long("proxy")
             .value_name("proxy"))
        .arg(Arg::with_name("burp")
             .conflicts_with("proxy")
             .display_order(50)
             .help(
"Sets the proxy to use the default burp proxy values
(http://localhost:8080)")
             .long("burp")
             .next_line_help(true)
             .takes_value(false))
        .arg(Arg::with_name("no_proxy")
             .conflicts_with("burp")
             .conflicts_with("proxy")
             .display_order(50)
             .help(
"Disables proxy use even if there is a system proxy")
             .long("no-proxy")
             .next_line_help(true)
             .takes_value(false))
        .arg(Arg::with_name("max_threads")
             .default_value("10")
             .display_order(60)
             .help(
"Sets the maximum number of request threads that will be spawned")
             .long("max-threads")
             .next_line_help(true)
             .short("t")
             .takes_value(true)
             .validator(positive_int_check)
             .value_name("max-threads"))
        .arg(Arg::with_name("wordlist_split")
             .default_value("3")
             .display_order(60)
             .help(
"The number of threads to run for each folder/extension combo")
             .long("wordlist-split")
             .next_line_help(true)
             .short("T")
             .validator(positive_int_check))
        .arg(Arg::with_name("throttle")
             .display_order(61)
             .help(
"Time each thread will wait between requests, given in milliseconds")
             .long("throttle")
             .next_line_help(true)
             .short("z")
             .takes_value(true)
             .validator(positive_int_check)
             .value_name("milliseconds"))
        .arg(Arg::with_name("username")
             .display_order(70)
             .help(
"Sets the username to authenticate with")
             .long("username")
             .next_line_help(true)
             .requires("password")
             .takes_value(true))
        .arg(Arg::with_name("password")
             .display_order(71)
             .help(
"Sets the password to authenticate with")
             .long("password")
             .next_line_help(true)
             .requires("username")
             .takes_value(true))
        .arg(Arg::with_name("disable_recursion")
             .display_order(80)
             .help(
"Disable discovered subdirectory scanning")
             .long("disable-recursion")
             .next_line_help(true)
             .short("r"))
        .arg(Arg::with_name("max_recursion_depth")
             .display_order(80)
             .help(
"Sets the maximum directory depth to recurse to, 0 will disable
recursion")
             .long("max-recursion-depth")
             .next_line_help(true)
             .takes_value(true)
             .validator(int_check))
        .arg(Arg::with_name("scan_listable")
             .display_order(80)
             .help(
"Scan listable directories")
             .long("scan-listable")
             .next_line_help(true)
             .short("l")
             .takes_value(false))
        .arg(Arg::with_name("scrape_listable")
             .display_order(80)
             .help(
"Enable scraping of listable directories for urls, often produces large
amounts of output")
             .long("scrape-listable")
             .next_line_help(true)
             .takes_value(false))
        .arg(Arg::with_name("cookie")
             .display_order(90)
             .help(
"Provide a cookie in the form \"name=value\", can be used multiple times")
             .long("cookie")
             .multiple(true)
             .next_line_help(true)
             .short("c")
             .takes_value(true))
        .arg(Arg::with_name("header")
             .display_order(90)
             .help(
"Provide an arbitrary header in the form \"header:value\" - headers with
no value must end in a semicolon")
             .long("header")
             .multiple(true)
             .next_line_help(true)
             .short("H")
             .takes_value(true))
        .arg(Arg::with_name("user_agent")
             .display_order(90)
             .help(
"Set the user-agent provided with requests, by default it isn't set")
             .long("user-agent")
             .next_line_help(true)
             .short("a")
             .takes_value(true))
        .arg(Arg::with_name("verbose")
             .display_order(100)
             .help(
"Print information when a thread starts and finishes scanning")
             .long("verbose")
             .next_line_help(true)
             .short("v")
             .takes_value(false))
        .arg(Arg::with_name("silent")
             .display_order(100)
             .help(
"Don't output information during the scan, only output the report at
the end")
             .long("silent")
             .next_line_help(true)
             .short("S")
             .takes_value(false))
        .arg(Arg::with_name("code_whitelist")
             .display_order(110)
             .help(
"Provide a comma separated list of response codes to show in output,
also disables detection of not found codes")
             .long("code-whitelist")
             .min_values(1)
             .multiple(true)
             .next_line_help(true)
             .short("W")
             .validator(positive_int_check)
             .value_delimiter(","))
        .arg(Arg::with_name("code_blacklist")
             .conflicts_with("code_whitelist")
             .display_order(110)
             .help(
"Provide a comma separated list of response codes to not show in output")
             .long("code-blacklist")
             .min_values(1)
             .multiple(true)
             .next_line_help(true)
             .short("B")
             .validator(positive_int_check)
             .value_delimiter(","))
        .arg(Arg::with_name("disable_validator")
             .display_order(110)
             .help(
"Disable automatic detection of not found codes")
             .long("disable-validator")
             .next_line_help(true)
             .takes_value(false))
        .arg(Arg::with_name("scan_401")
             .display_order(120)
             .help(
"Scan folders even if they return 401 - Unauthorized frequently")
             .long("scan-401")
             .next_line_help(true))
        .arg(Arg::with_name("scan_403")
             .display_order(120)
             .help(
"Scan folders if they return 403 - Forbidden frequently")
             .long("scan-403")
             .next_line_help(true))
        .arg(Arg::with_name("ignore_cert")
             .help(
"Ignore the certificate validity for HTTPS")
             .long("ignore-cert")
             .short("k"))
        .arg(Arg::with_name("show_htaccess")
             .help(
"Enable display of items containing .ht when they return 403 responses")
             .long("show-htaccess")
             .next_line_help(true))
        .arg(Arg::with_name("timeout")
             .default_value("5")
             .help(
"Maximum time to wait for a response before giving up, given in seconds
") // Newline is needed to force the [default: 5] onto the next line
             .long("timeout")
             .next_line_help(true)
             .validator(positive_int_check))
        .arg(Arg::with_name("max_errors")
             .default_value("5")
             .help(
"The number of consecutive errors a thread can have before it exits,
set to 0 to disable")
             .long("max-errors")
             .next_line_help(true)
             .validator(int_check))
        .arg(Arg::with_name("no_color")
             .alias("no-colour")
             .help("Disable coloring of terminal output")
             .long("no-color")
             .next_line_help(true))
        .get_matches();

    

    let mut hostnames:Vec<String> = Vec::new();

    // Get from host arguments
    if args.is_present("host") {
        hostnames.push(String::from(args.value_of("host").unwrap()))
    }
    if args.is_present("host_file") {
        for host_file in args.values_of("host_file").unwrap() {
            let hosts = lines_from_file(String::from(host_file));
            for hostname in hosts {
                if hostname.starts_with("https://") || hostname.starts_with("http://") { 
                    hostnames.push(String::from(hostname));
                }
                else {
                    println!("{} doesn't start with \"http://\" or \"https://\" - skipping", hostname);
                }
            }

        }
    }
    if args.is_present("extra_hosts") {
        for hostname in args.values_of("extra_hosts").unwrap() {
            hostnames.push(String::from(hostname));
        }
    }

    if hostnames.len() == 0 {
        println!("No valid hosts were provided - exiting");
        exit(2);
    }
    hostnames.sort();
    hostnames.dedup();

    // Parse wordlist file names into a vector
    let mut wordlists:Vec<String> = Vec::new();

    if args.is_present("wordlist") {
        for wordlist_file in args.values_of("wordlist").unwrap() {
            wordlists.push(String::from(wordlist_file));
        }
    }
    else {
        let mut exe_path = current_exe()
            .unwrap_or_else(|error| { println!("Getting directory of exe failed: {}", error); exit(2);});
        exe_path.set_file_name("dirble_wordlist.txt");
        wordlists.push(String::from(exe_path.to_str().unwrap()));
    }

    // Parse the prefixes into a vector
    let mut prefixes = vec![String::from("")];
    if args.is_present("prefixes") {
        for prefix in args.values_of("prefixes").unwrap() {
            prefixes.push(String::from(prefix));
        }
    }
    if args.is_present("prefix_file") {
        for prefixes_file in args.values_of("prefix_file").unwrap() {
            let prefixes_from_file = lines_from_file(String::from(prefixes_file));
            for prefix in prefixes_from_file {
                prefixes.push(String::from(prefix));
            }
        }
    }
    
    prefixes.sort();
    prefixes.dedup();

    // Parse the extensions into a vector, then sort it and remove duplicates
    let mut extensions = vec![String::from("")];
    if args.is_present("extensions") {
        for extension in args.values_of("extensions").unwrap() {
            extensions.push(String::from(extension));
        }
    }

    // Read in extensions from a file
    if args.is_present("extension_file") {
        for extensions_file in args.values_of("extension_file").unwrap() {
            let extensions_from_file = lines_from_file(String::from(extensions_file));
            for extension in extensions_from_file {
                extensions.push(String::from(extension));
            }
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
    else if args.is_present("output_all") {
        output_file = Some(format!("{}{}", args.value_of("output_all").unwrap(), ".txt"));
    }

    // Read the name of the json file if provided
    let mut json_file = None;
    if args.is_present("json_file") {
        json_file = Some(String::from(args.value_of("json_file").unwrap()));
    }
    else if args.is_present("output_all") {
        json_file = Some(format!("{}{}", args.value_of("output_all").unwrap(), ".json"));
    }

    let mut xml_file = None;
    if args.is_present("xml_file") {
        xml_file = Some(String::from(args.value_of("xml_file").unwrap()));
    }
    else if args.is_present("output_all") {
        xml_file = Some(format!("{}{}", args.value_of("output_all").unwrap(), ".xml"));
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

    let mut whitelist = false;
    let mut code_list:Vec<u32> = Vec::new();
    
    if args.is_present("code_whitelist") {
        whitelist = true;
        for code in args.values_of("code_whitelist").unwrap() {
            code_list.push(code.parse::<u32>().unwrap());
        }
    }
    else if args.is_present("code_blacklist") {
        whitelist = false;
        for code in args.values_of("code_blacklist").unwrap() {
            code_list.push(code.parse::<u32>().unwrap());
        }
    }

    let mut max_recursion_depth = None;
    if args.is_present("disable_recursion") {
        max_recursion_depth = Some(0);

    }
    else if args.is_present("max_recursion_depth") {
        let string_recursion_depth = args.value_of("max_recursion_depth").unwrap();
        max_recursion_depth = Some(string_recursion_depth.parse::<i32>().unwrap());
    }

    let mut scan_opts = ScanOpts{scan_401:false, scan_403:false};
    if args.is_present("scan_401") || (whitelist && code_list.contains(&401)) {
        scan_opts.scan_401 = true;
    }

    if args.is_present("scan_403") || (whitelist && code_list.contains(&403)) {
        scan_opts.scan_403 = true;
    }

    // Create the GlobalOpts struct and return it
    GlobalOpts {
        hostnames,
        wordlist_files: wordlists,
        prefixes,
        extensions,
        max_threads: args.value_of("max_threads").unwrap().parse::<u32>().unwrap(),
        proxy_enabled,
        proxy_address: proxy,
        proxy_auth_enabled: false,   
        ignore_cert: args.is_present("ignore_cert"),
        show_htaccess: args.is_present("show_htaccess"),
        throttle:
            if args.is_present("throttle") {
                args.value_of("throttle").unwrap().parse::<u32>().unwrap()
            } else { 0 },
        max_recursion_depth,
        user_agent,
        username,
        password,
        output_file,
        json_file,
        xml_file,
        verbose: args.is_present("verbose"),
        silent: args.is_present("silent"),
        timeout: args.value_of("timeout").unwrap().parse::<u32>().unwrap(),
        max_errors: args.value_of("max_errors").unwrap().parse::<u32>().unwrap(),
        wordlist_split: args.value_of("wordlist_split").unwrap().parse::<u32>().unwrap(),
        scan_listable: args.is_present("scan_listable"),
        cookies,
        headers,
        scrape_listable:args.is_present("scrape_listable"),
        whitelist,
        code_list,
        is_terminal: atty::is(Stream::Stdout),
        no_color: args.is_present("no_color"),
        disable_validator: args.is_present("disable_validator"),
        http_verb: value_t!(args.value_of("http_verb"), HttpVerb).unwrap(),
        scan_opts
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
