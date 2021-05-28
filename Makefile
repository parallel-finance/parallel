.PHONY: run
run:
	cargo run --manifest-path node/parallel-dev/Cargo.toml -- --dev -lruntime=debug

.PHONY: build
build: build-dev build-parallel

.PHONY: build-dev
build-dev:
	cargo build --manifest-path node/parallel-dev/Cargo.toml --locked

.PHONY: build-parallel
build-parallel:
	cargo build --manifest-path node/parallel/Cargo.toml --locked

.PHONY: check
check: check-dev check-parallel check-benchmarks
	SKIP_WASM_BUILD= cargo check

.PHONY: check-tests
check-tests:
	SKIP_WASM_BUILD= cargo check --tests --workspace

.PHONY: check-dev
check-dev:
	SKIP_WASM_BUILD= cargo check --manifest-path node/parallel-dev/Cargo.toml --tests --workspace

.PHONY: check-parallel
check-parallel:
	SKIP_WASM_BUILD= cargo check --manifest-path node/parallel/Cargo.toml --tests --workspace

.PHONY: check-benchmarks
check-benchmarks:
	SKIP_WASM_BUILD= cargo check --manifest-path node/parallel/Cargo.toml --tests --workspace --features runtime-benchmarks

.PHONY: check-debug
check-debug:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check

.PHONY: test
test: test-dev test-parallel

.PHONY: test-dev
test-dev:
	SKIP_WASM_BUILD= cargo test --manifest-path node/parallel-dev/Cargo.toml -p pallet-loans -p pallet-liquidation -p pallet-liquid-staking -p pallet-prices

.PHONY: test-parallel
test-parallel:
	SKIP_WASM_BUILD= cargo test --manifest-path node/parallel/Cargo.toml --workspace

.PHONY: bench
bench: bench-loans

.PHONY: bench-loans
bench-loans:
	target/release/parallel-dev benchmark --chain=dev --execution=wasm --wasm-execution=compiled --pallet=pallet-loans --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/loans/src/weights.rs

.PHONY: lint
lint:
	SKIP_WASM_BUILD= cargo clippy -- -D warnings

.PHONY: fmt
fmt:
	SKIP_WASM_BUILD= cargo fmt

.PHONY: purge
purge:
	target/debug/parallel-dev purge-chain --dev -y

.PHONY: restart
restart: purge run

.PHONY: resources
resources:
	target/release/parallel export-genesis-state --parachain-id 200 > ./resources/para-200-genesis
	target/release/parallel export-genesis-wasm > ./resources/para-200.wasm

.PHONY: launch
launch:
	polkadot-launch config.json


help:
	@grep -E '^[a-zA-Z_-]+:.*?' Makefile | cut -d: -f1 | sort
