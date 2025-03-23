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

use crate::wordlist::lines_from_file;
use atty::Stream;
use clap::{
    Arg, ArgAction, ArgGroup, Command, ValueEnum, builder::EnumValueParser,
    crate_version, value_parser,
};
use simplelog::LevelFilter;
use std::{ffi::OsString, fmt, path::PathBuf, process::exit};
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalOpts {
    pub hostnames: Vec<Url>,
    pub wordlist_files: Option<Vec<String>>,
    pub prefixes: Vec<String>,
    pub extensions: Vec<String>,
    pub extension_substitution: bool,
    pub max_threads: u32,
    pub proxy_enabled: bool,
    pub proxy_address: String,
    #[allow(dead_code, reason = "TODO")]
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
    pub no_color: bool,
    pub disable_validator: bool,
    pub http_verb: HttpVerb,
    pub scan_opts: ScanOpts,
    pub log_level: LevelFilter,
    pub length_blacklist: LengthRanges,
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct LengthRange {
    pub start: usize,
    pub end: Option<usize>,
}

impl LengthRange {
    pub fn contains(&self, test: usize) -> bool {
        if let Some(end) = self.end {
            self.start <= test && test <= end
        } else {
            test == self.start
        }
    }
}

impl fmt::Debug for LengthRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output: String;
        if let Some(end) = self.end {
            output = format!("{}-{}", self.start, end);
        } else {
            output = format!("{}", self.start);
        }
        write!(f, "{}", output)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LengthRanges {
    pub ranges: Vec<LengthRange>,
}

impl LengthRanges {
    pub fn contains(&self, test: usize) -> bool {
        for range in &self.ranges {
            if range.contains(test) {
                return true;
            }
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }
}

impl fmt::Display for LengthRanges {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ranges = self.ranges.clone();
        ranges.sort();
        write!(f, "{:?}", ranges)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ScanOpts {
    pub scan_401: bool,
    pub scan_403: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum HttpVerb {
    Get,
    Head,
    Post,
}

#[expect(clippy::derivable_impls, reason = "Interaction with arg_enum!")]
impl Default for HttpVerb {
    fn default() -> Self {
        HttpVerb::Get
    }
}

/// The supported output file types
#[derive(Copy, Clone)]
enum FileTypes {
    Txt,
    Json,
    Xml,
}

impl From<FileTypes> for &'static str {
    fn from(file: FileTypes) -> Self {
        match file {
            FileTypes::Txt => "txt",
            FileTypes::Json => "json",
            FileTypes::Xml => "xml",
        }
    }
}

#[allow(clippy::cognitive_complexity)]
pub fn get_args<ArgsIter>(args: ArgsIter) -> GlobalOpts
where
    ArgsIter: IntoIterator,
    ArgsIter::Item: Into<OsString> + Clone,
{
    // Mutable to allow extractors to move CLI arguments and avoid
    // unnecessary clones
    let mut args = app().get_matches_from(args);

    let mut hostnames: Vec<Url> = Vec::new();

    // Get from host arguments
    if let Some(host) = args.remove_one::<Url>("host") {
        hostnames.push(host);
    }
    if let Some(host_files) = args.remove_many::<PathBuf>("host_file") {
        for host_file in host_files {
            let hosts = lines_from_file(&host_file);
            for hostname in hosts {
                if url_is_valid(&hostname).is_ok() {
                    if let Ok(host) = Url::parse(hostname.as_str()) {
                        hostnames.push(host);
                    }
                } else {
                    println!("Invalid URL: {}", hostname);
                }
            }
        }
    }
    if let Some(extra_hosts) = args.remove_many::<Url>("extra_hosts") {
        hostnames.extend(extra_hosts);
    }

    if hostnames.is_empty() {
        println!("No valid hosts were provided - exiting");
        exit(2);
    }
    hostnames.sort();
    hostnames.dedup();

    // Parse wordlist file names into a vector
    let wordlists: Option<Vec<String>> =
        args.remove_many("wordlist").map(Iterator::collect);

    // Check for proxy related flags
    let proxy_enabled;
    let proxy_address;
    if let Some(proxy_addr) = args.remove_one("proxy") {
        proxy_enabled = true;
        proxy_address = proxy_addr;
        if proxy_address == "http://localhost:8080" {
            println!(
                "You could use the --burp flag instead of the --proxy flag!"
            );
        }
    } else if args.get_flag("burp") {
        proxy_enabled = true;
        proxy_address = "http://localhost:8080".into();
    } else if args.get_flag("no_proxy") {
        proxy_enabled = true;
        proxy_address = String::new();
    } else {
        proxy_enabled = false;
        proxy_address = String::new();
    }

    // Read provided cookie values into a vector
    let cookies: Option<String> = args
        .remove_many("cookie")
        .map(Iterator::collect::<Vec<String>>)
        .map(|cookies| cookies.join("; "));

    // Read provided headers into a vector
    let headers: Option<Vec<String>> =
        args.remove_many("header").map(Iterator::collect);

    let mut whitelist = false;
    let mut code_list: Vec<u32> = Vec::new();

    if let Some(whitelist_values) = args.remove_many::<u32>("code_whitelist") {
        whitelist = true;
        code_list.extend(whitelist_values);
    } else if let Some(blacklist_values) =
        args.remove_many::<u32>("code_blacklist")
    {
        whitelist = false;
        code_list.extend(blacklist_values);
    }

    let max_recursion_depth = if args.get_flag("disable_recursion") {
        Some(0)
    } else {
        args.remove_one("max_recursion_depth")
    };

    let mut scan_opts = ScanOpts {
        scan_401: false,
        scan_403: false,
    };
    if args.get_flag("scan_401") || (whitelist && code_list.contains(&401)) {
        scan_opts.scan_401 = true;
    }

    if args.get_flag("scan_403") || (whitelist && code_list.contains(&403)) {
        scan_opts.scan_403 = true;
    }

    // Configure the logging level. The silent flag overrides any
    // verbose flags in use.
    let log_level = if args.get_flag("silent") {
        LevelFilter::Warn
    } else {
        match args.get_count("verbose") {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    };

    // Create the GlobalOpts struct and return it
    GlobalOpts {
        hostnames,
        wordlist_files: wordlists,
        prefixes: load_modifiers(&mut args, "prefixes"),
        extensions: load_modifiers(&mut args, "extensions"),
        extension_substitution: args.get_flag("extension_substitution"),
        max_threads: args
            .remove_one("max_threads")
            .expect("Max threads is set"),
        proxy_enabled,
        proxy_address,
        proxy_auth_enabled: false,
        ignore_cert: args.get_flag("ignore_cert"),
        show_htaccess: args.get_flag("show_htaccess"),
        throttle: args.remove_one("throttle").unwrap_or_default(),
        max_recursion_depth,
        user_agent: args.remove_one("user_agent"),
        // Dependency between username and password is handled by Clap
        username: args.remove_one("username"),
        // Dependency between username and password is handled by Clap
        password: args.remove_one("password"),
        output_file: filename_from_args(&args, FileTypes::Txt),
        json_file: filename_from_args(&args, FileTypes::Json),
        xml_file: filename_from_args(&args, FileTypes::Xml),
        timeout: args.remove_one("timeout").expect("Timeout is set"),
        max_errors: args
            .remove_one::<u32>("max_errors")
            .expect("Max errors is an integer"),
        wordlist_split: args
            .remove_one("wordlist_split")
            .expect("Wordlist split is set"),
        scan_listable: args.get_flag("scan_listable"),
        cookies,
        headers,
        scrape_listable: args.get_flag("scrape_listable"),
        whitelist,
        code_list,
        is_terminal: atty::is(Stream::Stdout),
        no_color: args.get_flag("no_color"),
        disable_validator: args.get_flag("disable_validator"),
        http_verb: *args.get_one("http_verb").expect("Must be valid HTTP verb"),
        scan_opts,
        log_level,
        length_blacklist: if let Some(lengths) =
            args.get_many("length_blacklist")
        {
            length_blacklist_parse(lengths)
        } else {
            Default::default()
        },
    }
}

// Defines all the command line arguments with the Clap module
fn app() -> Command {
    // For general compilation, include the current commit hash and
    // build date in the version string. When building releases via the
    // Makefile, only use the release number.
    let version_string = get_version_string();
    Command::new("Dirble")
        .version(version_string)
        .author(
            "Developed by Izzy Whistlecroft \
            <Izzy(dot)Whistlecroft(at)nccgroup(dot).com>")
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
        .arg_required_else_help(true)
        .arg(Arg::new("host")
             .action(ArgAction::Set)
             .display_order(10)
             .help(
"The URI of the host to scan, optionally supports basic auth with
http://user:pass@host:port")
             .index(1)
             .next_line_help(true)
             .value_parser(value_parser!(Url))
             .value_name("uri"))
        .arg(Arg::new("extra_hosts")
             .action(ArgAction::Append)
             .alias("host")
             .display_order(10)
             .help(
"Additional hosts to scan")
             .long("uri")
             .next_line_help(true)
             .short('u')
             .value_parser(value_parser!(Url))
             .value_name("uri")
             .visible_alias("url"))
        .arg(Arg::new("host_file")
             .action(ArgAction::Append)
             .alias("host-file")
             .display_order(10)
             .help(
"The filename of a file containing a list of URIs to scan - cookies and
headers set will be applied to all URIs")
             .long("uri-file")
             .next_line_help(true)
             .short('U')
             .value_name("uri-file")
             .value_parser(value_parser!(PathBuf))
             .visible_alias("url-file"))
        .group(ArgGroup::new("hosts")
               .args(["host", "host_file", "extra_hosts"])
               .multiple(true)
               .required(true))
        .arg(Arg::new("http_verb")
             .action(ArgAction::Set)
             .default_value("get")
             .display_order(11)
             .help(
"Specify which HTTP verb to use
") // Newline is needed for the enumeration of possible values
             .ignore_case(true)
             .long("verb")
             .next_line_help(true)
             .value_parser(EnumValueParser::<HttpVerb>::new()))
        .arg(Arg::new("wordlist")
             .action(ArgAction::Append)
             .display_order(20)
             .help(
"Sets which wordlist to use, defaults to dirble_wordlist.txt in the same
folder as the executable")
             .long("wordlist")
             .next_line_help(true)
             .short('w')
             .value_name("wordlist"))
        .arg(Arg::new("extensions")
             .action(ArgAction::Append)
             .display_order(30)
             .help(
"Provides comma separated extensions to extend queries with")
             .long("extensions")
             .num_args(1..)
             .next_line_help(true)
             .short('x')
             .value_delimiter(',')
             .value_name("extensions"))
        .arg(Arg::new("extension_file")
             .action(ArgAction::Append)
             .display_order(30)
             .help(
"The name of a file containing extensions to extend queries with, one
per line")
             .long("extension-file")
             .next_line_help(true)
             .short('X').value_parser(value_parser!(PathBuf))
             .value_name("extension-file"))
        .group(ArgGroup::new("extension-options")
               .args(["extensions", "extension_file"])
               .multiple(true))
        .arg(Arg::new("extension_substitution")
             .action(ArgAction::SetTrue)
            .display_order(31)
            .help(
"Indicates whether the string \"%EXT%\" in a wordlist file should be 
substituted with the current extension")
            .long("ext-sub")
            .requires("extension-options"))
        .arg(Arg::new("force_extension")
             .action(ArgAction::SetTrue)
            .display_order(31)
            .help("Only scan with provided extensions")
            .requires("extension-options")
            .short('f')
            .long("force-extension"))
        .arg(Arg::new("prefixes")
             .action(ArgAction::Append)
             .display_order(30)
             .help(
"Provides comma separated prefixes to extend queries with")
             .long("prefixes")
             .num_args(1..)
             .next_line_help(true)
             .short('p')
             .value_delimiter(','))
        .arg(Arg::new("prefix_file")
             .action(ArgAction::Append)
             .display_order(30)
             .help(
"The name of a file containing extensions to extend queries with, one
per line")
             .long("prefix-file")
             .next_line_help(true)
             .short('P').value_parser(value_parser!(PathBuf))
             .value_name("prefix-file"))
        .arg(Arg::new("output_file")
             .action(ArgAction::Set)
             .display_order(40)
             .help(
"Sets the file to write the report to")
             .long("output-file")
             .next_line_help(true)
             .short('o')
             .visible_alias("oN"))
        .arg(Arg::new("json_file")
             .action(ArgAction::Set)
             .display_order(40)
             .help(
"Sets a file to write JSON output to")
             .long("json-file")
             .next_line_help(true)
                         .visible_alias("oJ"))
        .arg(Arg::new("xml_file")
             .action(ArgAction::Set)
             .display_order(40)
             .help(
"Sets a file to write XML output to")
             .long("xml-file")
             .next_line_help(true)
             .visible_alias("oX"))
        .arg(Arg::new("output_all")
             .action(ArgAction::Set)
             .display_order(41)
             .help(
"Stores all output types respectively as .txt, .json and .xml")
             .long("output-all")
             .next_line_help(true)
                         .visible_alias("oA"))
        .arg(Arg::new("proxy")
             .display_order(50)
             .help(
"The proxy address to use, including type and port, can also include a
username and password in the form 
\"http://username:password@proxy_url:proxy_port\"")
             .long("proxy")
             .value_name("proxy"))
        .arg(Arg::new("burp")
             .action(ArgAction::SetTrue)
             .conflicts_with("proxy")
             .display_order(50)
             .help(
"Sets the proxy to use the default burp proxy values
(http://localhost:8080)")
             .long("burp")
             .next_line_help(true)
             )
        .arg(Arg::new("no_proxy")
             .action(ArgAction::SetTrue)
             .conflicts_with("burp")
             .conflicts_with("proxy")
             .display_order(50)
             .help(
"Disables proxy use even if there is a system proxy")
             .long("no-proxy")
             .next_line_help(true)
            )
        .arg(Arg::new("max_threads")
             .action(ArgAction::Set)
             .default_value("10")
             .display_order(60)
             .help(
"Sets the maximum number of request threads that will be spawned")
             .long("max-threads")
             .next_line_help(true)
             .short('t')
             .value_parser(value_parser!(u32).range(1..))
             .value_name("max-threads"))
        .arg(Arg::new("wordlist_split")
             .default_value("3")
             .display_order(60)
             .help(
"The number of threads to run for each folder/extension combo")
             .long("wordlist-split")
             .next_line_help(true)
             .short('T')
            .value_parser(value_parser!(u32)))
        .arg(Arg::new("throttle")
             .action(ArgAction::Set)
             .display_order(61)
             .help(
"Time each thread will wait between requests, given in milliseconds")
             .long("throttle")
             .next_line_help(true)
             .short('z')
                         .value_parser(value_parser!(u32))
             .value_name("milliseconds"))
        .arg(Arg::new("username")
             .action(ArgAction::Set)
             .display_order(70)
             .help(
"Sets the username to authenticate with")
             .long("username")
             .next_line_help(true)
             .requires("password")
            )
        .arg(Arg::new("password")
             .action(ArgAction::Set)
             .display_order(71)
             .help(
"Sets the password to authenticate with")
             .long("password")
             .next_line_help(true)
             .requires("username")
             )
        .arg(Arg::new("disable_recursion")
             .action(ArgAction::SetTrue)
             .display_order(80)
             .help(
"Disable discovered subdirectory scanning")
             .long("disable-recursion")
             .next_line_help(true)
             .short('r'))
        .arg(Arg::new("max_recursion_depth")
             .action(ArgAction::Set)
             .display_order(80)
             .help(
"Sets the maximum directory depth to recurse to, 0 will disable
recursion")
             .long("max-recursion-depth")
             .next_line_help(true)
                        .value_parser(value_parser!(i32)))
        .arg(Arg::new("scan_listable")
             .action(ArgAction::SetTrue)
             .display_order(80)
             .help(
"Scan listable directories")
             .long("scan-listable")
             .next_line_help(true)
             .short('l')
             )
        .arg(Arg::new("scrape_listable")
             .action(ArgAction::SetTrue)
             .display_order(80)
             .help(
"Enable scraping of listable directories for urls, often produces large
amounts of output")
             .long("scrape-listable")
             .next_line_help(true)
             )
        .arg(Arg::new("cookie")
             .action(ArgAction::Append)
             .display_order(90)
             .help(
"Provide a cookie in the form \"name=value\", can be used multiple times")
             .long("cookie")
             .next_line_help(true)
             .short('c')
             )
        .arg(Arg::new("header")
             .action(ArgAction::Append)
             .display_order(90)
             .help(
"Provide an arbitrary header in the form \"header:value\" - headers with
no value must end in a semicolon")
             .long("header")
             .next_line_help(true)
             .short('H')
             )
        .arg(Arg::new("user_agent")
             .action(ArgAction::Set)
             .display_order(90)
             .help(
"Set the user-agent provided with requests, by default it isn't set")
             .long("user-agent")
             .next_line_help(true)
             .short('a')
           )
        .arg(Arg::new("verbose")
             .action(ArgAction::Count)
             .display_order(100)
             .help(
"Increase the verbosity level. Use twice for full verbosity.")
             .long("verbose")
             .next_line_help(true)
             .short('v')
             .conflicts_with("silent"))
        .arg(Arg::new("silent")
             .action(ArgAction::SetTrue)
             .display_order(100)
             .help(
"Don't output information during the scan, only output the report at
the end.")
             .long("silent")
             .next_line_help(true)
             .short('S'))
        .arg(Arg::new("code_whitelist")
             .action(ArgAction::Append)
             .display_order(110)
             .help(
"Provide a comma separated list of response codes to show in output,
also disables detection of not found codes")
             .long("code-whitelist")
             .num_args(1..)
             .next_line_help(true)
             .short('W')
             .value_delimiter(',')
             .value_parser(value_parser!(u32)))
        .arg(Arg::new("code_blacklist")
             .action(ArgAction::Append)
             .conflicts_with("code_whitelist")
             .display_order(110)
             .help(
"Provide a comma separated list of response codes to not show in output")
             .long("code-blacklist")
             .num_args(1..)
             .next_line_help(true)
             .short('B')
             .value_parser(value_parser!(u32))
             .value_delimiter(','))
        .arg(Arg::new("disable_validator")
             .action(ArgAction::SetTrue)
             .display_order(110)
             .help(
"Disable automatic detection of not found codes")
             .long("disable-validator")
             .next_line_help(true))
        .arg(Arg::new("scan_401")
             .action(ArgAction::SetTrue)
             .display_order(120)
             .help(
"Scan folders even if they return 401 - Unauthorized frequently")
             .long("scan-401")
             .next_line_help(true))
        .arg(Arg::new("scan_403")
             .action(ArgAction::SetTrue)
             .display_order(120)
             .help(
"Scan folders if they return 403 - Forbidden frequently")
             .long("scan-403")
             .next_line_help(true))
        .arg(Arg::new("ignore_cert")
             .action(ArgAction::SetTrue)
             .help(
"Ignore the certificate validity for HTTPS")
             .long("ignore-cert")
             .short('k'))
        .arg(Arg::new("show_htaccess")
             .action(ArgAction::SetTrue)
             .help(
"Enable display of items containing .ht when they return 403 responses")
             .long("show-htaccess")
             .next_line_help(true))
        .arg(Arg::new("timeout")
             .default_value("5")
             .help(
"Maximum time to wait for a response before giving up, given in seconds\n")
             .long("timeout")
             .next_line_help(true)
             .value_parser(value_parser!(u32)))
        .arg(Arg::new("max_errors")
             .default_value("5")
             .help(
"The number of consecutive errors a thread can have before it exits,
set to 0 to disable")
             .long("max-errors")
             .next_line_help(true)
             .value_parser(value_parser!(u32)))
        .arg(Arg::new("no_color")
             .action(ArgAction::SetTrue)
             .alias("no-colour")
             .help("Disable coloring of terminal output")
             .long("no-color")
             .next_line_help(true))
        .arg(Arg::new("length_blacklist")
             .action(ArgAction::Append)
             .help(
"Specify length ranges to hide, e.g. --hide-lengths 348,500-700")
             .long("hide-lengths")
             .num_args(1..)
             .next_line_help(true)
                          .value_delimiter(','))
}

/// filetype is one of "txt", "json", and "xml". Returns a filename that is
/// either the filename supplied by the user if the corresponding argument has
/// been given, or if the "output_all" argument is provided then build a
/// filename from the prefix given
#[inline]
fn filename_from_args(
    args: &clap::ArgMatches,
    filetype: FileTypes,
) -> Option<String> {
    match filetype {
        FileTypes::Txt => {
            if let Some(output_file) = args.get_one::<String>("output_file") {
                return Some(output_file.to_string());
            }
        }
        FileTypes::Json => {
            if let Some(json_file) = args.get_one::<String>("json_file") {
                return Some(json_file.to_string());
            }
        }
        FileTypes::Xml => {
            if let Some(xml_file) = args.get_one::<String>("xml_file") {
                return Some(xml_file.to_string());
            }
        }
    }

    // This function is called once for each filetype, so we don't remove
    // the arg from the matcher
    args.get_one::<String>("output_all")
        .map(|output_all_prefix| {
            format!("{}.{}", output_all_prefix, <&'static str>::from(filetype))
        })
}

fn load_modifiers(args: &mut clap::ArgMatches, mod_type: &str) -> Vec<String> {
    let singular_arg;
    let file_arg;
    match mod_type {
        "prefixes" => {
            singular_arg = "prefixes";
            file_arg = "prefix_file";
        }
        "extensions" => {
            singular_arg = "extensions";
            file_arg = "extension_file";
        }
        _ => panic!(),
    }
    let file_arg = String::from(file_arg);

    let mut modifiers = vec![];

    if !args.get_flag("force_extension") || mod_type == "prefixes" {
        modifiers.push(String::from(""));
    }

    if let Some(singular_args) = args.remove_many(singular_arg) {
        modifiers.extend(singular_args);
    }
    if let Some(file_args) = args.remove_many::<PathBuf>(&file_arg) {
        for filename in file_args {
            modifiers.extend(lines_from_file(&filename));
        }
    }

    modifiers.sort();
    modifiers.dedup();

    if modifiers.is_empty() {
        panic!(
            "The length of the {} list is zero! Did you use -f with an empty {} file?",
            mod_type, mod_type
        );
    }

    modifiers
}

#[inline]
pub fn get_version_string() -> &'static str {
    if cfg!(feature = "release_version_string")
        || env!("VERGEN_GIT_SHA") == "UNKNOWN"
    {
        crate_version!()
    } else {
        concat!(
            crate_version!(),
            " (commit ",
            env!("VERGEN_GIT_SHA"),
            ", build ",
            env!("VERGEN_BUILD_DATE"),
            ")"
        )
    }
}

fn url_is_valid(hostname: &str) -> Result<(), String> {
    let url = Url::parse(hostname);
    if let Ok(u) = url {
        if u.scheme() == "http" || u.scheme() == "https" {
            Ok(())
        } else {
            Err(format!("The URI \"{}\" is not HTTP or HTTPS", hostname))
        }
    } else {
        Err(format!("The URI \"{}\" is invalid", hostname))
    }
}

fn length_blacklist_parse(
    blacklist_inputs: clap::parser::ValuesRef<String>,
) -> LengthRanges {
    let mut length_vector: Vec<LengthRange> =
        Vec::with_capacity(blacklist_inputs.len());

    for length in blacklist_inputs {
        let start;
        let end;

        if length.contains('-') {
            let components: Vec<&str> = length.split('-').collect();
            assert!(
                components.len() == 2,
                "Ranges must be in the form `150-300`"
            );
            start = components[0]
                .parse::<usize>()
                .expect("Length must be integers");
            end = Some(
                components[1]
                    .parse::<usize>()
                    .expect("Ranges must be in the form `150-300`"),
            );
            assert!(
                // This unwrap is permitted because it is always a Some
                start < end.unwrap(),
                "The start of a range must be smaller than the end"
            );
        } else {
            // Length is just one number
            start = length
                .parse::<usize>()
                .expect("Length range must be integers");
            end = None;
        }
        length_vector.push(LengthRange { start, end });
    }
    LengthRanges {
        ranges: length_vector,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use log::LevelFilter::Info;
    use pretty_assertions::assert_eq;
    use std::io::Write;
    use tempfile::NamedTempFile;

    impl Default for GlobalOpts {
        fn default() -> Self {
            GlobalOpts {
                hostnames: Default::default(),
                wordlist_files: Default::default(),
                prefixes: vec!["".into()],
                extensions: vec!["".into()],
                extension_substitution: false,
                max_threads: 10,
                proxy_enabled: Default::default(),
                proxy_address: Default::default(),
                proxy_auth_enabled: Default::default(),
                ignore_cert: Default::default(),
                show_htaccess: Default::default(),
                throttle: Default::default(),
                max_recursion_depth: Default::default(),
                user_agent: Default::default(),
                username: Default::default(),
                password: Default::default(),
                output_file: Default::default(),
                json_file: Default::default(),
                xml_file: Default::default(),
                timeout: 5,
                max_errors: 5,
                wordlist_split: 3,
                scan_listable: Default::default(),
                cookies: Default::default(),
                headers: Default::default(),
                scrape_listable: Default::default(),
                whitelist: Default::default(),
                code_list: Default::default(),
                is_terminal: Default::default(),
                no_color: Default::default(),
                disable_validator: Default::default(),
                http_verb: Default::default(),
                scan_opts: Default::default(),
                log_level: Info,
                length_blacklist: Default::default(),
            }
        }
    }

    #[test]
    fn argparse_length_range_contains() {
        // Range with start and end values
        let range = LengthRange {
            start: 3,
            end: Some(6),
        };
        // Number too small
        assert!(!range.contains(2));
        // Number matching the lower bound (inclusive contains)
        assert!(range.contains(3));
        // Number in the middle
        assert!(range.contains(4));
        // Upper bound (inclusive)
        assert!(range.contains(6));
        // Number too big
        assert!(!range.contains(7));

        // Range with just a start value
        let range = LengthRange {
            start: 5,
            end: None,
        };
        // Number too small
        assert!(!range.contains(4));
        // Number equal
        assert!(range.contains(5));
        // Number too big
        assert!(!range.contains(6));
    }

    #[test]
    fn argparse_length_ranges_contain() {
        // Empty range
        let ranges: LengthRanges = Default::default();
        assert!(!ranges.contains(4));

        // Non-overlapping ranges
        let ranges = LengthRanges {
            ranges: vec![
                LengthRange {
                    start: 4,
                    end: Some(10),
                },
                LengthRange {
                    start: 15,
                    end: Some(18),
                },
            ],
        };
        // too small
        assert!(!ranges.contains(3));
        // in first range
        assert!(ranges.contains(4));
        // in between
        assert!(!ranges.contains(11));
        //in second range
        assert!(ranges.contains(18));
        // too large
        assert!(!ranges.contains(19));
    }

    #[test]
    fn verify_app() {
        app().debug_assert();
    }

    #[track_caller]
    fn assert_args<ArgsIter>(args: ArgsIter, expected: GlobalOpts)
    where
        ArgsIter: IntoIterator,
        ArgsIter::Item: Into<OsString> + Clone,
    {
        let mut opts = get_args(args);
        // normalise the is-terminal value as it's not useful to test
        opts.is_terminal = expected.is_terminal;
        let info = std::panic::Location::caller();
        assert_eq!(opts, expected, "Caller: {}:{}", info.file(), info.line());
    }

    fn make_uri_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            &mut file,
            "http://some-host
            https://other-host"
        )
        .unwrap();
        file
    }

    fn make_extensions_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(&mut file, "txt").unwrap();
        writeln!(&mut file, "jpg").unwrap();
        writeln!(&mut file, "png").unwrap();
        file
    }

    #[test]
    fn required_args() {
        assert_args(
            ["test", "http://some-host"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                ..Default::default()
            },
        );
        assert_args(
            ["test", "-u", "http://some-host", "-u", "https://other-host"],
            GlobalOpts {
                hostnames: vec![
                    "http://some-host".parse().unwrap(),
                    "https://other-host".parse().unwrap(),
                ],
                ..Default::default()
            },
        );
        let uri_file = make_uri_file();
        assert_args(
            ["test", "-U", &uri_file.path().display().to_string()],
            GlobalOpts {
                hostnames: vec![
                    "http://some-host".parse().unwrap(),
                    "https://other-host".parse().unwrap(),
                ],
                ..Default::default()
            },
        );
        assert_args(
            [
                "test",
                "-U",
                &uri_file.path().display().to_string(),
                "-u",
                "http://third-host",
            ],
            GlobalOpts {
                hostnames: vec![
                    "http://some-host".parse().unwrap(),
                    "http://third-host".parse().unwrap(),
                    "https://other-host".parse().unwrap(),
                ],
                ..Default::default()
            },
        );
    }

    #[test]
    fn http_verb() {
        assert_args(
            ["test", "--verb", "Get", "http://some-host"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                ..Default::default()
            },
        );
        assert_args(
            ["test", "--verb", "Post", "http://some-host"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                http_verb: HttpVerb::Post,
                ..Default::default()
            },
        );
        assert_args(
            ["test", "--verb", "Head", "http://some-host"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                http_verb: HttpVerb::Head,
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--verb", "Head"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                http_verb: HttpVerb::Head,
                ..Default::default()
            },
        );
    }

    #[test]
    fn wordlist() {
        assert_args(
            ["test", "http://some-host", "--wordlist", "file-one"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                wordlist_files: Some(vec!["file-one".into()]),
                ..Default::default()
            },
        );
        assert_args(
            [
                "test",
                "http://some-host",
                "--wordlist",
                "file-one",
                "-w",
                "file-two",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                wordlist_files: Some(vec![
                    "file-one".into(),
                    "file-two".into(),
                ]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn extensions() {
        assert_args(
            ["test", "http://some-host", "--extensions", "txt"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                extensions: vec![String::new(), "txt".into()],
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--extensions", "txt,jpg"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                extensions: vec![String::new(), "jpg".into(), "txt".into()],
                ..Default::default()
            },
        );
        assert_args(
            [
                "test",
                "http://some-host",
                "--extensions",
                "txt,jpg",
                "-x",
                "png",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                extensions: ["", "jpg", "png", "txt"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                ..Default::default()
            },
        );
    }

    #[test]
    fn extension_file() {
        let exts_file = make_extensions_file();
        assert_args(
            [
                "test",
                "http://some-host",
                "--extension-file",
                &exts_file.path().display().to_string(),
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                extensions: ["", "jpg", "png", "txt"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                ..Default::default()
            },
        );
        assert_args(
            [
                "test",
                "http://some-host",
                "-X",
                &exts_file.path().display().to_string(),
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                extensions: ["", "jpg", "png", "txt"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                ..Default::default()
            },
        );
        assert_args(
            [
                "test",
                "http://some-host",
                "-X",
                &exts_file.path().display().to_string(),
                "-x",
                "rs",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                extensions: ["", "jpg", "png", "rs", "txt"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                ..Default::default()
            },
        );
    }

    #[test]
    fn ext_sub() {
        assert_args(
            [
                "test",
                "http://some-host",
                "--ext-sub",
                "-x",
                "txt,png,rs,jpg",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                extensions: ["", "jpg", "png", "rs", "txt"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                extension_substitution: true,
                ..Default::default()
            },
        );
        assert_args(
            [
                "test",
                "http://some-host",
                "--ext-sub",
                "-x",
                "txt,png,rs,jpg",
                "--force-extension",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                extensions: ["jpg", "png", "rs", "txt"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                extension_substitution: true,
                ..Default::default()
            },
        );
        assert_args(
            [
                "test",
                "http://some-host",
                "--ext-sub",
                "-x",
                "txt,png,rs,jpg",
                "-f",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                extensions: ["jpg", "png", "rs", "txt"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                extension_substitution: true,
                ..Default::default()
            },
        );
    }

    #[test]
    fn prefixes() {
        assert_args(
            ["test", "http://some-host", "--prefixes", "txt"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                prefixes: vec![String::new(), "txt".into()],
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--prefixes", "txt,jpg"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                prefixes: vec![String::new(), "jpg".into(), "txt".into()],
                ..Default::default()
            },
        );
        assert_args(
            [
                "test",
                "http://some-host",
                "--prefixes",
                "txt,jpg",
                "-p",
                "png",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                prefixes: ["", "jpg", "png", "txt"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                ..Default::default()
            },
        );
    }

    #[test]
    fn prefix_file() {
        let exts_file = make_extensions_file();
        assert_args(
            [
                "test",
                "http://some-host",
                "--prefix-file",
                &exts_file.path().display().to_string(),
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                prefixes: ["", "jpg", "png", "txt"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                ..Default::default()
            },
        );
        assert_args(
            [
                "test",
                "http://some-host",
                "-P",
                &exts_file.path().display().to_string(),
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                prefixes: ["", "jpg", "png", "txt"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                ..Default::default()
            },
        );
        assert_args(
            [
                "test",
                "http://some-host",
                "-P",
                &exts_file.path().display().to_string(),
                "-p",
                "rs",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                prefixes: ["", "jpg", "png", "rs", "txt"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                ..Default::default()
            },
        );
    }

    #[test]
    fn output_file() {
        assert_args(
            ["test", "http://some-host", "-o", "some-file"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                output_file: Some("some-file".into()),
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--oN", "some-file"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                output_file: Some("some-file".into()),
                ..Default::default()
            },
        );
    }

    #[test]
    fn json_file() {
        assert_args(
            ["test", "http://some-host", "--json-file", "some-file"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                json_file: Some("some-file".into()),
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--oJ", "some-file"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                json_file: Some("some-file".into()),
                ..Default::default()
            },
        );
    }

    #[test]
    fn xml_file() {
        assert_args(
            ["test", "http://some-host", "--xml-file", "some-file"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                xml_file: Some("some-file".into()),
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--oX", "some-file"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                xml_file: Some("some-file".into()),
                ..Default::default()
            },
        );
    }

    #[test]
    fn output_all() {
        assert_args(
            ["test", "http://some-host", "--output-all", "some-file"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                output_file: Some("some-file.txt".into()),
                json_file: Some("some-file.json".into()),
                xml_file: Some("some-file.xml".into()),
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--oA", "some-file"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                output_file: Some("some-file.txt".into()),
                json_file: Some("some-file.json".into()),
                xml_file: Some("some-file.xml".into()),
                ..Default::default()
            },
        );
    }

    #[test]
    fn proxy() {
        assert_args(
            ["test", "http://some-host", "--proxy", "http://proxy:8000"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                proxy_enabled: true,
                proxy_address: "http://proxy:8000".into(),
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--burp"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                proxy_enabled: true,
                proxy_address: "http://localhost:8080".into(),
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--no-proxy"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                //TODO should this set proxy_enabled to false?
                proxy_enabled: true,
                proxy_address: String::new(),
                ..Default::default()
            },
        );
    }

    #[test]
    fn max_threads() {
        assert_args(
            ["test", "http://some-host", "--max-threads", "4"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                max_threads: 4,
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "-t", "4"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                max_threads: 4,
                ..Default::default()
            },
        );
    }

    #[test]
    fn wordlist_split() {
        assert_args(
            ["test", "http://some-host", "--wordlist-split", "4"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                wordlist_split: 4,
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "-T", "4"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                wordlist_split: 4,
                ..Default::default()
            },
        );
    }

    #[test]
    fn throttle() {
        assert_args(
            ["test", "http://some-host", "--throttle", "4"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                throttle: 4,
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "-z", "4"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                throttle: 4,
                ..Default::default()
            },
        );
    }

    #[test]
    fn username_password() {
        assert_args(
            [
                "test",
                "http://some-host",
                "--username",
                "user",
                "--password",
                "pass",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                username: Some("user".into()),
                password: Some("pass".into()),
                ..Default::default()
            },
        );
    }

    #[test]
    fn recursion() {
        assert_args(
            ["test", "http://some-host", "--disable-recursion"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                max_recursion_depth: Some(0),
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--max-recursion-depth", "4"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                max_recursion_depth: Some(4),
                ..Default::default()
            },
        );
        assert_args(
            [
                "test",
                "http://some-host",
                "--max-recursion-depth",
                "4",
                "--disable-recursion",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                max_recursion_depth: Some(0),
                ..Default::default()
            },
        );
    }

    #[test]
    fn listable() {
        assert_args(
            ["test", "http://some-host", "--scan-listable"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                scan_listable: true,
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--scrape-listable"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                scrape_listable: true,
                ..Default::default()
            },
        );
    }

    #[test]
    fn cookie() {
        assert_args(
            [
                "test",
                "http://some-host",
                "--cookie",
                "name1=value1",
                "-c",
                "name2=value2",
                "-c",
                "name3=value3",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                cookies: Some(
                    "name1=value1; name2=value2; name3=value3".into(),
                ),
                ..Default::default()
            },
        );
    }

    #[test]
    fn header() {
        assert_args(
            [
                "test",
                "http://some-host",
                "--header",
                "header1:value1",
                "-H",
                "header2: value2",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                headers: Some(
                    ["header1:value1", "header2: value2"]
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                ),
                ..Default::default()
            },
        );
    }

    #[test]
    fn user_agent() {
        assert_args(
            [
                "test",
                "http://some-host",
                "--user-agent",
                "some user agent",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                user_agent: Some("some user agent".into()),
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "-a", "some user agent"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                user_agent: Some("some user agent".into()),
                ..Default::default()
            },
        );
    }

    #[test]
    fn verbosity() {
        assert_args(
            ["test", "http://some-host"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                log_level: LevelFilter::Info,
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--verbose"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                log_level: LevelFilter::Debug,
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--verbose", "--verbose"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                log_level: LevelFilter::Trace,
                ..Default::default()
            },
        );
        assert_args(
            [
                "test",
                "http://some-host",
                "--verbose",
                "--verbose",
                "--verbose",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                log_level: LevelFilter::Trace,
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "-v"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                log_level: LevelFilter::Debug,
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "-vv"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                log_level: LevelFilter::Trace,
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "-vvv"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                log_level: LevelFilter::Trace,
                ..Default::default()
            },
        );
    }

    #[test]
    fn silent() {
        assert_args(
            ["test", "http://some-host", "--silent"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                log_level: LevelFilter::Warn,
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "-S"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                log_level: LevelFilter::Warn,
                ..Default::default()
            },
        );
    }

    #[test]
    fn code_whitelist() {
        assert_args(
            [
                "test",
                "http://some-host",
                "--code-whitelist",
                "200,201,204",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                whitelist: true,
                code_list: vec![200, 201, 204],
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "-W", "200,201,204"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                whitelist: true,
                code_list: vec![200, 201, 204],
                ..Default::default()
            },
        );
    }

    #[test]
    fn code_blacklist() {
        assert_args(
            [
                "test",
                "http://some-host",
                "--code-blacklist",
                "200,201,204",
            ],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                whitelist: false,
                code_list: vec![200, 201, 204],
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "-B", "200,201,204"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                whitelist: false,
                code_list: vec![200, 201, 204],
                ..Default::default()
            },
        );
    }

    #[test]
    fn disable_validator() {
        assert_args(
            ["test", "http://some-host", "--disable-validator"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                disable_validator: true,
                ..Default::default()
            },
        );
    }

    #[test]
    fn scan_opts() {
        assert_args(
            ["test", "http://some-host", "--scan-401"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                scan_opts: ScanOpts {
                    scan_401: true,
                    scan_403: false,
                },
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--scan-403"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                scan_opts: ScanOpts {
                    scan_401: false,
                    scan_403: true,
                },
                ..Default::default()
            },
        );
    }

    #[test]
    fn ignore_cert() {
        assert_args(
            ["test", "http://some-host", "--ignore-cert"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                ignore_cert: true,
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "-k"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                ignore_cert: true,
                ..Default::default()
            },
        );
    }

    #[test]
    fn show_htaccess() {
        assert_args(
            ["test", "http://some-host", "--show-htaccess"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                show_htaccess: true,
                ..Default::default()
            },
        );
    }

    #[test]
    fn timeout() {
        assert_args(
            ["test", "http://some-host", "--timeout", "2"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                timeout: 2,
                ..Default::default()
            },
        );
    }

    #[test]
    fn max_errors() {
        assert_args(
            ["test", "http://some-host", "--max-errors", "2"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                max_errors: 2,
                ..Default::default()
            },
        );
    }

    #[test]
    fn no_colour() {
        assert_args(
            ["test", "http://some-host", "--no-colour"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                no_color: true,
                ..Default::default()
            },
        );
        assert_args(
            ["test", "http://some-host", "--no-color"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                no_color: true,
                ..Default::default()
            },
        );
    }

    #[test]
    fn hide_lengths() {
        assert_args(
            ["test", "http://some-host", "--hide-lengths", "400,600-700"],
            GlobalOpts {
                hostnames: vec!["http://some-host".parse().unwrap()],
                length_blacklist: LengthRanges {
                    ranges: vec![
                        LengthRange {
                            start: 400,
                            end: None,
                        },
                        LengthRange {
                            start: 600,
                            end: Some(700),
                        },
                    ],
                },
                ..Default::default()
            },
        );
    }
}
