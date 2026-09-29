.PHONY: all build install test clean

all: install

build:
	cargo build --release

install: build
	mkdir -p $(HOME)/.local/bin
	install -m 755 target/release/tetyak $(HOME)/.local/bin/tetyak
	install -m 755 target/release/tetris $(HOME)/.local/bin/tetris
	@echo "Installed tetyak and tetris to $(HOME)/.local/bin"

test:
	cargo test

clean:
	cargo clean
