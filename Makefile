# this exists only to test, but it can be used anyways

all:
	cargo build

test:
	cargo test

run:
	cargo run

install:
	cargo install --path .

.PHONY: all test run install
