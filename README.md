# Building from source

To build on your current platform, run `cargo build --release`

To cross-compile for 32- and 64-bit Linux and Windows targets, there is a handy makefile. `make release` will build for all four targets using `cross`. This depends on having cross and docker installed (`cargo install cross`).
