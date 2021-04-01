prepare:
	rustup target add wasm32-unknown-unknown

build-proposal:
	cargo build --release -p proposal --target wasm32-unknown-unknown
	
build-reputation:
	cargo build --release -p reputation --target wasm32-unknown-unknown

build-voting:
	cargo build --release -p voting --target wasm32-unknown-unknown

build-governance:
	cargo build --release -p governance --target wasm32-unknown-unknown
build-execution:
	cargo build --release -p execution --target wasm32-unknown-unknown

test-only:
	cargo test -p tests

lint:
	cargo fmt
	cargo clippy --all-targets --all -- -D warnings -A renamed_and_removed_lints

clean:
	cargo clean

copy-wasm-file-to-test:
	cp target/wasm32-unknown-unknown/release/execution.wasm tests/execution.wasm && cp target/wasm32-unknown-unknown/release/reputation.wasm tests/reputation.wasm && cp target/wasm32-unknown-unknown/release/governance.wasm tests/governance.wasm && cp target/wasm32-unknown-unknown/release/proposal.wasm tests/proposal.wasm && cp target/wasm32-unknown-unknown/release/voting.wasm tests/voting.wasm

test: build copy-wasm-file-to-test test-only

build: build-proposal build-reputation build-voting build-governance build-execution