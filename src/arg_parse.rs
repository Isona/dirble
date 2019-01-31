extern crate clap;
use clap::{App, Arg, ArgMatches, AppSettings};

pub fn get_args() -> ArgMatches<'static>
{
    let matches = App::new("dirble")
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
                        .get_matches();
    matches


}


fn starts_with_http(hostname: String) -> Result<(), String> {
    if hostname.starts_with("https://") || hostname.starts_with("http://") {
        Ok(())
    }
    else {
        Err(String::from("The provided target URI must start with http:// or https://"))
    }
}