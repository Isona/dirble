# Introduction

Dirble is a website directory scanning tool for Windows and Linux. It's designed to be fast to run and easy to use.

# How to Use

Download one of the precompiled binaries for your system, then run it from a terminal. By default Dirble looks for a dirble_wordlist.txt in the directory it is run from.

# Example Uses

Run against a website using the default dirble_wordlist.txt from the current directory:
`dirble [address]`

Run with a different wordlist and including .php and .html extensions:
`dirble [address] -w example_wordlist.txt -X .php,.html`

With listable directory scraping enabled:
`dirble [address] --scrape-listable`

# Building from source

To build on your current platform, ensure cargo is installed and then run `cargo build --release`.

To cross-compile for 32- and 64-bit Linux and Windows targets, there is a handy makefile. `make release` will build for all four targets using `cross`. This depends on having cross and docker installed (`cargo install cross`).

# Features

|                                  | Dirble | Dirb | Dirsearch | Gobuster |
|----------------------------------|:------:|:----:|:---------:|:--------:|
| .ht* file filtering              |    ✅   |   ❌  |     ❌     |     ❌    |
| Cookies                          |    ✅   |   ✅  |     ✅     |     ❌    |
| Custom headers                   |    ✅   |   ✅  |     ✅     |     ❌    |
| Exclude status codes             |    ❌   |   ✅  |     ✅     |     ❌    |
| Extensions                       |    ✅   |   ✅  |     ✅     |     ✅    |
| HTTP basic auth                  |    ✅   |   ✅  |     ❌     |     ✅    |
| Listable directory optimisation  |    ✅   |   ✅  |     ❌     |     ❌    |
| Listable directory scraping      |    ✅   |   ❌  |     ❌     |     ❌    |
| Output file                      |    ✅   |   ✅  |     ✅     |     ✅    |
| Positive status codes            |    ❌   |   ❌  |     ❌     |     ✅    |
| Proxy                            |    ✅   |   ✅  |     ✅     |     ✅    |
| Recursion                        |    ✅   |   ✅  |     ✅     |     ❌    |
| Speed                            |    ✅   |   ✅  |     ❌     |     ✅    |
| Threading                        |    ✅   |   ❌  |     ✅     |     ✅    |
| Throttle                         |    ✅   |   ✅  |     ✅     |     ❌    |
| Tune 404 based on size/redirection |    ❌   |   ✅  |     ❌     |     ❌    |
| URL list                         |    ❌   |   ❌  |     ✅     |     ❌    |
| User agents                      |    ✅   |   ✅  |     ✅     |     ✅    |

# Performance

The following graph was generated with by running each tool with Hyperfine against a test server with 5ms latency and 1% packet loss. (Gobuster was omitted due to lack of recursion).

![This is a cool graph](images/comparison_graph.png)

Released under GPL v3.0, see LICENSE for more information