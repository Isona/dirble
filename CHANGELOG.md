# Changelog

## 1.4.0 - 2019-07-05

### Added
* Add ability to do scans using HEAD and POST requests
* Directories which return 401 and 403 codes are no longer scanned by default
* Not found detection now can detect response lengths that vary by the URL length
* Added dockerfile to git repository
* Commit hashes are now displayed with the version number in local builds

### Changed
* Wordlist items now have a leading and trailing slash removed
* Default wordlist location checks have been improved
* SimpleLog crate now used to print additional scanning information
* Silent and verbose flags now affect logging level
* Output for listable directories now has a bold L
* Wordlist splitting of initial URLs is increased

### Fixed
* Disable recursion flag now works as intended
* Validator always defaulting to 404

## 1.3.1 - 2019-05-01

### Changed
* The --host argument has been changed to --uri and --url
* The --host-file argument has been changed to --uri-file and --url-file
* Version number added to startup text
* Startup text now uses "Targets" instead of "Hosts"
* Version number is now pulled from cargo.toml

### Fixed
* Panic when there were errors during target validation

## 1.3.0 - 2019-04-22

### Added
* Option to save output as XML
* Option to save output as JSON
* XML schema
* Flag to output all file formats
* Target validator thread
* Per directory not found response tuning using status code/size
* Option to disable not found response tuning

### Changed
* Output is now handled by an output thread rather than the main thread
* Better error output when libcurl returns an error from a request
* Look for default wordlist in executable directory instead of working directory

### Fixed
* Bug involving redirect URLs being output being incorrect
* Panic when writing to json file when there were no results
* Threads reporting they're done twice when they end from errors
* Directories being output as code:0|size:0


## 1.2.0 - 2019-04-11

### Added
* Coloured status codes in terminal output
* No-color option
* Disable end report printing if output is not a terminal
* Support for loading multiple wordlists
* Prefix support
* Whitelisting and blacklisting of response codes to display

### Changed
* Threading options now have short options
* Reordered help text


## 1.1.0 - 2019-04-08

### Added
* Text at start up
* Option to provide multiple hosts to scan via command line
* Support for lists of hosts to scan from a file
* Ability to provide multiple lists of extensions via command line
* Support for multiple lists of extensions from file

### Changed
* Swapped -x and -X flags for consistency
* Help text to have more use cases

### Fixed
* Support for non utf-8 files

## 1.0.1 - 2019-04-03

### Added
* Extra information at the end of the help text

### Changed
* Optimised calculation of response sizes

### Fixed
* Bug with indentation when a folder was provided to scan
* Typo in readme.md


## 1.0.0 - 2019-04-01

### Added
* Initial directory scanning functionality
* Extensions on the command line and from a file
* Saving the output into a text file
* Setting the thread count
* Setting the number of threads working on each wordlist+directory job
* Request throttling
* Various proxy configurations (including Burpsuite's default)
* Passing cookies with each request
* Passing custom headers with each request
* Basic auth support
* Setting custom user agents
* Scraping the urls from listable directories
* Option to force a full scan of listable directories
* Non-recursive mode
* Showing or hiding .ht files that return 403 responses