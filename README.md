# Building from source

To build on your current platform, run `cargo build --release`

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
| NTLM auth                        |    ❌   |   ❌  |     ❌     |     ❌    |
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
