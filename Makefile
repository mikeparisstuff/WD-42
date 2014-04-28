docs:
	rustdoc -o ./public/doc rust-http-master/src/http/lib.rs
	rustdoc -L rust-http-master/build -o ./public/doc application.rs

all: docs
	cd rust-http-master && make all
	rustc -L rust-http-master/build server.rs

run: all
	./server

