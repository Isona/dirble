.PHONY : help release
.SILENT : help

targets = x86_64-unknown-linux-gnu \
		  i686-unknown-linux-gnu \
		  x86_64-pc-windows-gnu \
		  i686-pc-windows-gnu \
#		  wasm32-unknown-emscripten
#		  ^ Potential bug in cross, openssl does not compile for wasm for some
#		  reason.

default :
	cargo build --release
	@echo Release binary: target/release/dirble

help :
	echo "Run 'make release' to make all targets"
	echo "To build for just one platform then run 'make <target>'"
	echo "Supported targets are:"
	echo $(targets)

release : $(targets) dirble_wordlist.txt
	mkdir -p dirble/extensions
	cp dirble_wordlist.txt dirble/
	cp wordlists/* dirble/extensions
	cp target/x86_64-unknown-linux-gnu/release/dirble dirble/dirble
	cp target/i686-unknown-linux-gnu/release/dirble dirble/dirble32
	cp target/x86_64-pc-windows-gnu/release/dirble.exe dirble/dirble.exe
	cp target/i686-pc-windows-gnu/release/dirble.exe dirble/dirble32.exe
	zip dirble/dirble-x86_64-linux.zip \
		dirble/dirble \
		dirble/dirble_wordlist.txt \
		dirble/extensions/*
	zip dirble/dirble-i686-linux.zip \
		dirble/dirble32 \
		dirble/dirble_wordlist.txt \
		dirble/extensions/*
	zip dirble/dirble-x86_64-windows.zip \
		dirble/dirble.exe \
		dirble/dirble_wordlist.txt \
		dirble/extensions/*
	zip dirble/dirble-i686-windows.zip \
		dirble/dirble32.exe \
		dirble/dirble_wordlist.txt \
		dirble/extensions/*
	rm -rfv release
	mv dirble release

x86_64-unknown-linux-gnu : 
	cross build --release --target x86_64-unknown-linux-gnu

i686-unknown-linux-gnu :
	cross build --release --target i686-unknown-linux-gnu

x86_64-pc-windows-gnu :
	cross build --release --target x86_64-pc-windows-gnu

i686-pc-windows-gnu :
	cross build --release --target i686-pc-windows-gnu

#wasm32-unknown-emscripten :
#	cross build --release --target wasm32-unknown-emscripten
