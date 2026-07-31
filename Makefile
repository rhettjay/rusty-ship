.PHONY: all build run release check test clean dev

all: build

build:
	cargo build

run:
	cargo run

release:
	cargo run --release

check:
	cargo check

test:
	cargo test

clean:
	cargo clean

dev: check test
