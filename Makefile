.PHONY : help release
.SILENT : help

targets = x86_64-unknown-linux-gnu \
		  i686-unknown-linux-gnu \
		  x86_64-pc-windows-gnu \
		  i686-pc-windows-gnu \
#		  wasm32-unknown-emscripten
#		  ^ Potential bug in cross, openssl does not compile for wasm for some
#		  reason.

cargo_flags = --release \
			  --features release_version_string

# Get the version string out of the Cargo.toml by taking the second field
# (delimited by double quotes) of the 'version = "x.y.z"' line
version=$(shell awk -F'"' '/version/ {print $$2}' Cargo.toml)
date=$(shell date +%Y%m%d)
filename="dirble-${version}-${date}"

default :
	cargo build --release
	@echo Release binary: target/release/dirble

help :
	echo "Run 'make release' to make all targets"
	echo "To build for just one platform then run 'make <target>'"
	echo "Supported targets are:"
	echo $(targets)
	echo "There is a combined mac target for 32 and 64-bit systems,"
	echo "currently only supports being run on a mac."

release : $(targets) dirble_wordlist.txt
	mkdir -p dirble/extensions
	cp dirble_wordlist.txt dirble/
	cp wordlists/* dirble/extensions
	cp target/x86_64-unknown-linux-gnu/release/dirble dirble/dirble
	cp target/i686-unknown-linux-gnu/release/dirble dirble/dirble32
	cp target/x86_64-pc-windows-gnu/release/dirble.exe dirble/dirble.exe
	cp target/i686-pc-windows-gnu/release/dirble.exe dirble/dirble32.exe
	zip dirble/${filename}-x86_64-linux.zip \
		dirble/dirble \
		dirble/dirble_wordlist.txt \
		dirble/extensions/*
	zip dirble/${filename}-i686-linux.zip \
		dirble/dirble32 \
		dirble/dirble_wordlist.txt \
		dirble/extensions/*
	zip dirble/${filename}-x86_64-windows.zip \
		dirble/dirble.exe \
		dirble/dirble_wordlist.txt \
		dirble/extensions/*
	zip dirble/${filename}-i686-windows.zip \
		dirble/dirble32.exe \
		dirble/dirble_wordlist.txt \
		dirble/extensions/*
	rm -rfv release
	mv dirble release

mac : x86_64-apple-darwin dirble_wordlist.txt
	mkdir -p dirble/extensions
	cp dirble_wordlist.txt dirble/
	cp wordlists/* dirble/extensions
	cp target/x86_64-apple-darwin/release/dirble dirble/dirble
	zip dirble/${filename}-x86_64-apple-darwin.zip \
		dirble/dirble \
		dirble/dirble_wordlist.txt \
		dirble/extensions/*
	rm -rfv release
	mv dirble release

x86_64-unknown-linux-gnu : 
	cross build $(cargo_flags) --target x86_64-unknown-linux-gnu

i686-unknown-linux-gnu :
	cross build $(cargo_flags) --target i686-unknown-linux-gnu

x86_64-pc-windows-gnu :
	cross build $(cargo_flags) --target x86_64-pc-windows-gnu

i686-pc-windows-gnu :
	cross build $(cargo_flags) --target i686-pc-windows-gnu

x86_64-apple-darwin :
	cargo build $(cargo_flags) --target x86_64-apple-darwin

#wasm32-unknown-emscripten :
#	cross build --release --target wasm32-unknown-emscripten
