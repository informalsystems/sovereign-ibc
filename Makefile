.PHONY: help

help: ## Display this help message
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

install-dev-tools:  ## Installs all necessary cargo helpers
	cargo install cargo-llvm-cov
	cargo install cargo-hack
	cargo install cargo-udeps
	cargo install flaky-finder
	cargo install cargo-nextest --locked
	cargo install cargo-risczero
	cargo risczero install

build: ## Build the the entire project
	@cargo build

build-sov-celestia-cw: ## Build the WASM file for the sov-celestia-cw light client
	@echo "Building the WASM file for the sov-celestia-cw light client"
	@RUSTFLAGS='-C link-arg=-s' cargo build -p sov-celestia-client-cw --target wasm32-unknown-unknown --release --lib --locked
	@mkdir -p contracts
	@cp target/wasm32-unknown-unknown/release/sov_celestia_client_cw.wasm contracts/

optimize-contracts: ## Optimize WASM files in contracts directory
	@echo "Optimizing WASM files..."
	@for wasm_file in contracts/*.wasm; do \
		optimized_file="contracts/$$(basename $$wasm_file .wasm).opt.wasm"; \
		wasm-opt "$$wasm_file" -o "$$optimized_file" -Os; \
	done

clean: ## Cleans compiled
	@cargo clean

lint:  ## cargo check and clippy. Skip clippy on guest code since it's not supported by risc0
	## fmt first, because it's the cheapest
	cargo +nightly fmt --all --check
	cargo check --all-targets --all-features
	CI_SKIP_GUEST_BUILD=1 cargo clippy --all-targets --all-features

lint-fix:  ## cargo fmt, fix and clippy. Skip clippy on guest code since it's not supported by risc0
	cargo +nightly fmt --all
	cargo fix --allow-dirty
	CI_SKIP_GUEST_BUILD=1 cargo clippy --fix --allow-dirty

find-unused-deps: ## Prints unused dependencies for project. Note: requires nightly
	cargo udeps --all-targets --all-features

check-features: ## Checks that project compiles with all combinations of features. default is not needed because we never check `cfg(default)`, we only use it as an alias.
	cargo hack check --workspace --feature-powerset --exclude-features default

test-legacy: ## Runs test suite with output from tests printed
	@cargo test -- --nocapture -Zunstable-options --report-time

test:  ## Runs test suite using next test
	@cargo nextest run --workspace --all-features

docs:  ## Generates documentation locally
	cargo doc --all-features --no-deps --release --open

check-risc0:  ## Checks that the project compiles with risc0
	cd ci/risc0-check && RISC0_DEV_MODE=1 CARGO_NET_GIT_FETCH_WITH_CLI=true cargo run