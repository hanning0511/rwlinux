.DEFAULT_GOAL := all

rw:
	cargo build --release --bin=rw

devmem:
	cargo build --release --bin=devmem

rwapi:
	cargo build --release --bin=rwapi

all: devmem rwapi rw

install:
	cp target/release/rw /usr/bin/
	cp target/release/rwapi /usr/bin/
	cp target/release/devmem /usr/bin/

clean:
	cargo clean