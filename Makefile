.PHONY: run
run:
	cargo run --bin parallel-dev -- --dev -lruntime=debug

.PHONY: build
build: build-dev build-parallel

.PHONY: build-dev
build-dev:
	cargo build --bin parallel-dev --locked

.PHONY: build-parallel
build-parallel:
	cargo build --bin parallel --locked

.PHONY: check
check:
	SKIP_WASM_BUILD= cargo check --all-targets --all-features

.PHONY: test
test:
	SKIP_WASM_BUILD= cargo test --workspace --exclude parallel --exclude parallel-dev --exclude parallel-runtime --exclude vanilla-runtime --exclude heiko-runtime --exclude pallet-loans-benchmarking -- --nocapture

.PHONY: bench
bench: bench-loans bench-liquid-staking

.PHONY: bench-loans
bench-loans:
	target/release/parallel-dev benchmark --chain=dev --execution=wasm --wasm-execution=compiled --pallet=pallet-loans --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/loans/src/weights.rs

.PHONY: bench-liquid-staking
bench-liquid-staking:
	target/release/parallel-dev benchmark --chain=dev --execution=wasm --wasm-execution=compiled --pallet=pallet-liquid-staking --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/liquid-staking/src/weights.rs

.PHONY: lint
lint:
	SKIP_WASM_BUILD= cargo clippy --workspace --exclude parallel --exclude parallel-dev --exclude pallet-loans-benchmarking -- -A clippy::type_complexity -A clippy::identity_op -D warnings

.PHONY: fmt
fmt:
	SKIP_WASM_BUILD= cargo fmt --all -- --check

.PHONY: purge
purge:
	target/debug/parallel-dev purge-chain --dev -y

.PHONY: restart
restart: purge run

.PHONY: resources
resources:
	docker run --rm parallelfinance/parallel:latest export-genesis-state --chain heiko-dev --parachain-id 2085 > ./resources/para-2085-genesis
	docker run --rm parallelfinance/parallel:latest export-genesis-wasm --chain heiko-dev > ./resources/para-2085.wasm

.PHONY: launch
launch:
	parachain-launch generate && cp docker-compose.override.yml output && cd output && docker-compose up -d --build

.PHONY: wasm
wasm:
	./scripts/srtool-build.sh

help:
	@grep -E '^[a-zA-Z_-]+:.*?' Makefile | cut -d: -f1 | sort
