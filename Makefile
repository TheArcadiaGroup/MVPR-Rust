prepare:
	rustup target add wasm32-unknown-unknown

build-proposal:
	cargo build --release -p proposal --target wasm32-unknown-unknown
	
build-reputation:
	cargo build --release -p reputation --target wasm32-unknown-unknown

test-only:
	cargo test -p tests

lint:
	cargo fmt
	cargo clippy --all-targets --all -- -D warnings -A renamed_and_removed_lints

clean:
	cargo clean

copy-wasm-file-to-test:
	cp target/wasm32-unknown-unknown/release/contract.wasm tests/Contract.wasm

test: build-contract copy-wasm-file-to-test test-only

build: build-proposal build-reputation