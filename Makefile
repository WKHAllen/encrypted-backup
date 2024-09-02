all: build

build:
	cargo build

run:
	cargo run -p ui

test:
	cargo test -- --nocapture

lint:
	cargo clippy -- -D warnings

clean:
	cargo clean
