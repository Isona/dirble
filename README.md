# Introduction

Dirble is a website directory scanning tool for Windows and Linux.

# How to Use

Download one of the precompiled binaries for your system, then run it from a terminal. By default Dirble looks for a dirble_wordlist.txt in the directory it is run from.

# Example Uses

Run against a website using the default dirble_wordlist.txt:
`dirble [address]`

# Building from source

To build on your current platform, ensure cargo is installed and then run `cargo build --release`.

To cross-compile for 32- and 64-bit Linux and Windows targets, there is a handy makefile. `make release` will build for all four targets using `cross`. This depends on having cross and docker installed (`cargo install cross`).

# Features

|                                  | Dirble | Dirb | Dirsearch | Gobuster |
|----------------------------------|:------:|:----:|:---------:|:--------:|
| .ht* file filtering              |    ✅   |   ❌  |     ❌     |     ❌    |
| Cookies                          |    ✅   |   ✅  |     ✅     |     ❌    |
| Custom HTTP verbs                |    ❌   |   ❌  |     ❌     |     ❌    |
| Custom headers                   |    ✅   |   ✅  |     ✅     |     ❌    |
| Exclude status codes             |    ❌   |   ✅  |     ✅     |     ❌    |
| Extensions                       |    ✅   |   ✅  |     ✅     |     ✅    |
| HTTP basic auth                  |    ✅   |   ✅  |     ❌     |     ✅    |
| Import a base request from file  |    ❌   |   ❌  |     ❌     |     ❌    |
| Listable directory optimisation  |    ✅   |   ✅  |     ❌     |     ❌    |
| Load headers from a file         |    ❌   |   ❌  |     ❌     |     ❌    |
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
| Listable Directory Scraping      |    ✅   |   ❌  |     ❌     |     ❌    |
