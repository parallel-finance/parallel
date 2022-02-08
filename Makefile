PARA_ID        							:= 2085
CHAIN          							:= vanilla-dev
RUNTIME        							:= vanilla-runtime
BLOCK_AT       							:= 0x0000000000000000000000000000000000000000000000000000000000000000
URL            							:= ws://localhost:9947
KEYSTORE_PATH  							:= keystore
SURI           							:= //Alice
LAUNCH_CONFIG  							:= config.yml
DOCKER_TAG     							:= latest
RELAY_DOCKER_TAG						:= v0.9.16

.PHONY: init
init: submodules
	git config advice.ignoredHook false
	git config core.hooksPath .githooks
	rustup target add wasm32-unknown-unknown
	cd launch && yarn

.PHONY: submodules
submodules:
	git submodule update --init --recursive
	git submodule foreach git pull origin master

.PHONY: build
build:
	cargo build --bin parallel

.PHONY: ci
ci: check lint check-wasm test

.PHONY: check
check:
	SKIP_WASM_BUILD= cargo check --all-targets --features runtime-benchmarks --features try-runtime

.PHONY: check-wasm
check-wasm:
	cargo check -p vanilla-runtime -p parallel-runtime -p heiko-runtime --features runtime-benchmarks

.PHONY: test
test:
	SKIP_WASM_BUILD= cargo test --workspace --features runtime-benchmarks --exclude parallel --exclude parallel-runtime --exclude vanilla-runtime --exclude heiko-runtime --exclude pallet-loans-rpc --exclude pallet-loans-rpc-runtime-api --exclude parallel-primitives -- --nocapture

.PHONY: bench
bench: bench-loans bench-liquid-staking bench-amm bench-amm-router bench-crowdloans bench-bridge bench-xcm-helper
	./scripts/benchmark.sh

.PHONY: bench-loans
bench-loans:
	cargo run --release --features runtime-benchmarks -- benchmark --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-loans --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/loans/src/weights.rs

.PHONY: bench-crowdloans
bench-crowdloans:
	cargo run --release --features runtime-benchmarks -- benchmark --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-crowdloans --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/crowdloans/src/weights.rs

.PHONY: bench-bridge
bench-bridge:
	cargo run --release --features runtime-benchmarks -- benchmark --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-bridge --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/bridge/src/weights.rs

.PHONY: bench-xcm-helper
bench-xcm-helper:
	cargo run --release --features runtime-benchmarks -- benchmark --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-xcm-helper --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/xcm-helper/src/weights.rs

.PHONY: bench-amm
bench-amm:
	cargo run --release --features runtime-benchmarks -- benchmark --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-amm --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/amm/src/weights.rs

.PHONY: bench-liquid-staking
bench-liquid-staking:
	cargo run --release --features runtime-benchmarks -- benchmark --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-liquid-staking --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/liquid-staking/src/weights.rs

.PHONY: bench-amm-router
bench-amm-router:
	cargo run --release --features runtime-benchmarks -- benchmark --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-router --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/router/src/weights.rs

.PHONY: lint
lint:
	SKIP_WASM_BUILD= cargo fmt --all -- --check
	SKIP_WASM_BUILD= cargo clippy --workspace --features runtime-benchmarks --exclude parallel -- -D dead_code -A clippy::derivable_impls -A clippy::explicit_counter_loop -A clippy::unnecessary_cast -A clippy::unnecessary_mut_passed -A clippy::too_many_arguments -A clippy::type_complexity -A clippy::identity_op -D warnings

.PHONY: fix
fix:
	SKIP_WASM_BUILD= cargo fix --all-targets --allow-dirty --allow-staged

.PHONY: fmt
fmt:
	SKIP_WASM_BUILD= cargo fmt --all

.PHONY: resources
resources:
	docker run --rm parallelfinance/parallel:$(DOCKER_TAG) export-genesis-state --chain $(CHAIN) > ./resources/para-$(PARA_ID)-genesis
	docker run --rm parallelfinance/parallel:$(DOCKER_TAG) export-genesis-wasm --chain $(CHAIN) > ./resources/para-$(PARA_ID).wasm

.PHONY: shutdown
shutdown:
	docker-compose -f output/docker-compose.yml -f output/docker-compose.override.yml down --remove-orphans > /dev/null 2>&1 || true
	rm -fr output || true
	docker volume prune -f

.PHONY: launch
launch: shutdown
	docker image pull parallelfinance/polkadot:$(RELAY_DOCKER_TAG)
	docker image pull parallelfinance/parallel:$(DOCKER_TAG)
	docker image pull parallelfinance/stake-client:latest
	docker image pull parallelfinance/liquidation-client:latest
	docker image pull parallelfinance/nominate-client:latest
	docker image pull parallelfinance/oracle-client:latest
	docker image pull parallelfinance/parallel-dapp:latest
	DOCKER_CLIENT_TIMEOUT=120 COMPOSE_HTTP_TIMEOUT=120 parachain-launch generate $(LAUNCH_CONFIG) && (cp -r keystore* output || true) && cp docker-compose.override.yml output && cd output && docker-compose up -d --build
	cd launch && yarn start

.PHONY: logs
logs:
	docker-compose -f output/docker-compose.yml logs -f

.PHONY: wasm
wasm:
	PACKAGE=$(RUNTIME) ./scripts/srtool-build.sh

.PHONY: spec
spec:
	docker run --rm parallelfinance/parallel:$(DOCKER_TAG) build-spec --chain $(CHAIN) --disable-default-bootnode --raw > ./resources/$(CHAIN)-raw.json

.PHONY: image
image:
	docker build --build-arg BIN=parallel \
		-c 512 \
		-t parallelfinance/parallel:$(DOCKER_TAG) \
		-f Dockerfile.release \
		. --network=host

.PHONY: key
key:
	docker run --rm parallelfinance/parallel:$(DOCKER_TAG) key generate-node-key

.PHONY: keystore
keystore:
	cargo run --bin parallel key insert -d . --keystore-path $(KEYSTORE_PATH) --suri "$(SURI)" --key-type aura
	cargo run --bin parallel key insert -d . --keystore-path $(KEYSTORE_PATH) --suri "$(SURI)" --key-type gran

.PHONY: snapshot
snapshot:
	cargo run --bin parallel --features try-runtime -- try-runtime --chain $(CHAIN) --wasm-execution=compiled on-runtime-upgrade live -a=$(BLOCK_AT) -u=$(URL) -s=snapshot.bin

.PHONY: try-runtime-upgrade
try-runtime-upgrade:
	RUST_LOG=debug cargo run --bin parallel --features try-runtime -- try-runtime --chain $(CHAIN) --wasm-execution=compiled on-runtime-upgrade snap -s snapshot.bin

.PHONY: local-spec
local-spec:
	../polkadot/target/release/polkadot build-spec \
		 --chain rococo-local \
		 --raw \
		 --disable-default-bootnode > ../polkadot/rococo_local.json

.PHONY: local-relay-alice
local-relay-alice:
	../polkadot/target/release/polkadot \
		--chain ../polkadot/rococo_local.json \
		-d ./resources/cumulus_relay0 \
		--validator \
		--alice \
		--port 50555 \
		--node-key 0000000000000000000000000000000000000000000000000000000000000001

.PHONY: local-relay-bob
local-relay-bob:
	../polkadot/target/release/polkadot \
		--chain ../polkadot/rococo_local.json \
		-d ./resources/cumulus_relay1 \
		--validator \
		--bob \
		--port 50556 \
		--bootnodes /ip4/127.0.0.1/tcp/50555/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp

.PHONY: dev-genesis-and-wasm
dev-genesis-and-wasm:
	./target/release/parallel export-genesis-state \
		--chain heiko-dev > ./resources/heiko-dev-para-2085-genesis
	./target/release/parallel export-genesis-wasm \
		--chain heiko-dev > ./resources/heiko-dev-para-2085.wasm

.PHONY: start-local
start-local:
	./target/release/parallel \
		-d local-test \
		--alice \
		--collator \
		--force-authoring \
		--chain heiko-dev \
		--ws-port 9915 \
		-- \
		--execution wasm \
		--chain ../polkadot/rococo_local.json \
		--bootnodes /ip4/127.0.0.1/tcp/50555/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp

.PHONY: clear-local-relays
clear-local-relays:
	rm -rf ./resources/cumulus_relay*

.PHONY: clear-local-parachain
clear-local-parachain:
	rm -rf ./local-test

.PHONY: local-launch
local-launch:
	cd launch && yarn local

help:
	@grep -E '^[a-zA-Z_-]+:.*?' Makefile | cut -d: -f1 | sort
