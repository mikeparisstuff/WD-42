

all: rustic application
	cd rust-http-master && make all
	rustc -L rust-http-master/build rustic.rs

