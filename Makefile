PARA_ID  			:= 2085
CHAIN    			:= heiko-dev
KEYSTORE_PATH := keystore
SURI          := //Alice
LAUNCH_CONFIG := config.yml

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
	SKIP_WASM_BUILD= cargo fmt --all

.PHONY: purge
purge:
	target/debug/parallel-dev purge-chain --dev -y

.PHONY: restart
restart: purge run

.PHONY: resources
resources:
	docker run --rm parallelfinance/parallel:latest export-genesis-state --chain $(CHAIN) --parachain-id $(PARA_ID) > ./resources/para-$(PARA_ID)-genesis
	docker run --rm parallelfinance/parallel:latest export-genesis-wasm --chain $(CHAIN) > ./resources/para-$(PARA_ID).wasm

.PHONY: shutdown
shutdown:
	docker-compose -f output/docker-compose.yml -f output/docker-compose.override.yml down --remove-orphans > /dev/null 2>&1 || true
	rm -fr output || true
	docker volume prune -f

.PHONY: launch
launch: shutdown
	docker image pull parallelfinance/polkadot:v0.9.9-1
	docker image pull parallelfinance/parallel-dapp:latest
	parachain-launch generate $(LAUNCH_CONFIG) && (cp -r keystore* output || true) && cp docker-compose.override.yml output && docker-compose -f output/docker-compose.yml -f output/docker-compose.override.yml up -d --build

.PHONY: logs
logs:
	docker-compose -f output/docker-compose.yml logs -f

.PHONY: wasm
wasm:
	./scripts/srtool-build.sh

.PHONY: spec
spec:
	docker run --rm parallelfinance/parallel:latest build-spec --chain $(CHAIN) --disable-default-bootnode --raw > ./resources/$(CHAIN)-raw.json

.PHONY: image
image:
	docker build --build-arg BIN=parallel \
		-c 512 \
		-t parallelfinance/parallel:latest \
		-f Dockerfile.release \
		. --network=host

.PHONY: keystore
keystore:
	./target/debug/parallel key insert -d . --keystore-path $(KEYSTORE_PATH) --suri "$(SURI)" --key-type aura

help:
	@grep -E '^[a-zA-Z_-]+:.*?' Makefile | cut -d: -f1 | sort
