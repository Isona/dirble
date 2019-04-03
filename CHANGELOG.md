# Changelog

## 1.0.1 - 2019-04-03

## Added
* Extra information at the end of the help text

## Changed
* Optimised calculation of response sizes

## Fixed
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