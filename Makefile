.PHONY : help release
.SILENT : help

targets = x86_64-unknown-linux-gnu \
		  i686-unknown-linux-gnu \
		  x86_64-pc-windows-gnu \
		  i686-pc-windows-gnu \
#		  wasm32-unknown-emscripten
#		  ^ Potential bug in cross, openssl does not compile for wasm for some
#		  reason.

help :
	echo "Run 'make release' to make all targets"
	echo "To build for just one platform then run 'make <target>'"
	echo "Supported targets are:"
	echo $(targets)

release : $(targets)
	mkdir -p bin
	cp target/x86_64-unknown-linux-gnu/release/dirble bin/dirble
	cp target/i686-unknown-linux-gnu/release/dirble bin/dirble32
	cp target/x86_64-pc-windows-gnu/release/dirble.exe bin/dirble.exe
	cp target/i686-pc-windows-gnu/release/dirble.exe bin/dirble32.exe

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
