ALL_CONTRACTS = casper_ido erc20-test-call 
CONTRACT_TARGET_DIR = target/wasm32-unknown-unknown/release

prepare:
	rustup target add wasm32-unknown-unknown

build-contracts:
	cargo build --release --target wasm32-unknown-unknown $(patsubst %, -p %, $(ALL_CONTRACTS))
	$(foreach WASM, $(ALL_CONTRACTS), wasm-strip $(CONTRACT_TARGET_DIR)/$(subst -,_,$(WASM)).wasm 2>/dev/null | true;)

test: build-contracts
	mkdir -p testing/tests/wasm
	cp target/wasm32-unknown-unknown/release/casper_ido.wasm testing/tests/wasm
	cp target/wasm32-unknown-unknown/release/pre_invest.wasm testing/tests/wasm
	cp target/wasm32-unknown-unknown/release/erc20_test.wasm testing/tests/wasm
	cp target/wasm32-unknown-unknown/release/erc20_test_call.wasm testing/tests/wasm
	cd testing/tests && cargo test

clippy:
	cargo clippy --all-targets -- -D warnings
	cargo clippy --all-targets -p erc20-token --target wasm32-unknown-unknown -- -D warnings

check-lint: clippy
	cargo fmt --all -- --check

lint: clippy
	cargo fmt --all

clean:
	cargo clean