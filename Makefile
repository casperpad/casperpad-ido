ALL_CONTRACTS = casper-ido-contract
CONTRACT_TARGET_DIR = target/wasm32-unknown-unknown/release

prepare:
	rustup target add wasm32-unknown-unknown

build-contracts:
	cargo build --release --target wasm32-unknown-unknown $(patsubst %, -p %, $(ALL_CONTRACTS))
	$(foreach WASM, $(ALL_CONTRACTS), wasm-strip $(CONTRACT_TARGET_DIR)/$(subst -,_,$(WASM)).wasm 2>/dev/null | true;)

test: build-contracts
	mkdir -p tests/wasm
	cp target/wasm32-unknown-unknown/release/casper_ido_contract.wasm tests/wasm
	cd tests && cargo test
	# cd casper-ido-tests && cargo test
	# cargo test

clippy:
	cargo clippy --all-targets -- -D warnings
	cargo clippy --all-targets -p erc20-token --target wasm32-unknown-unknown -- -D warnings

check-lint: clippy
	cargo fmt --all -- --check

lint: clippy
	cargo fmt --all

clean:
	cargo clean