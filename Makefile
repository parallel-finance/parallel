PARA_ID        											:= 2012
CHAIN          											:= kerria-dev
RELAY_CHAIN                         := polkadot-local
RUNTIME        											:= kerria-runtime
BLOCK_AT       											:= 0x0000000000000000000000000000000000000000000000000000000000000000
URL            											:= ws://localhost:9948
RELAY_URL            								:= ws://localhost:9944
KEYSTORE_PATH  											:= keystore
SURI           											:= //Alice
LAUNCH_CONFIG_YAML	  							:= config.yml
LAUNCH_CONFIG_JSON	  							:= config.json
DOCKER_OVERRIDE_YAML                := docker-compose.override.yml
DOCKER_TAG     											:= latest
RELAY_DOCKER_TAG										:= v0.9.17

.PHONY: init
init: submodules
	git config advice.ignoredHook false
	git config core.hooksPath .githooks
	rustup target add wasm32-unknown-unknown
	cd scripts/helper && yarn
	cd scripts/polkadot-launch && yarn

.PHONY: submodules
submodules:
	git submodule update --init --recursive
	git submodule foreach git pull origin master

.PHONY: build
build:
	cargo build --bin parallel

.PHONY: build-release
build-release:
	cargo build --workspace --exclude runtime-integration-tests --bin parallel --release  --features runtime-benchmarks --features try-runtime

.PHONY: clean
clean:
	cargo clean -p parallel -p vanilla-runtime -p kerria-runtime -p heiko-runtime -p parallel-runtime

.PHONY: ci
ci: check lint check-helper check-wasm test integration-test

.PHONY: check
check:
	SKIP_WASM_BUILD= cargo check --all-targets --features runtime-benchmarks --features try-runtime

.PHONY: check-wasm
check-wasm:
	cargo check -p vanilla-runtime -p kerria-runtime -p parallel-runtime -p heiko-runtime --features runtime-benchmarks

.PHONY: check-helper
check-helper:
	cd scripts/helper && yarn && yarn build

.PHONY: test
test:
	SKIP_WASM_BUILD= cargo test --workspace --features runtime-benchmarks --exclude runtime-integration-tests --exclude parallel --exclude parallel-runtime --exclude vanilla-runtime --exclude kerria-runtime --exclude heiko-runtime --exclude pallet-loans-rpc --exclude pallet-loans-rpc-runtime-api --exclude parallel-primitives -- --nocapture

.PHONY: integration-test
integration-test:
	SKIP_WASM_BUILD= cargo test -p runtime-integration-tests -- --nocapture

.PHONY: bench
bench: bench-loans bench-liquid-staking bench-amm bench-amm-router bench-crowdloans bench-bridge bench-xcm-helper bench-farming
	./scripts/benchmark.sh

.PHONY: bench-farming
bench-farming:
	cargo run --release --features runtime-benchmarks -- benchmark --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-farming --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/farming/src/weights.rs

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


.PHONY: bench-streaming
bench-streaming:
	cargo run --release --features runtime-benchmarks -- benchmark --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-streaming --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/stream/src/weights.rs

.PHONY: bench-asset-registry
bench-asset-registry:
	cargo run --release --features runtime-benchmarks -- benchmark --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-asset-registry --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/asset-registry/src/weights.rs

.PHONY: lint
lint:
	SKIP_WASM_BUILD= cargo fmt --all -- --check
	SKIP_WASM_BUILD= cargo clippy --workspace --features runtime-benchmarks --exclude parallel -- -D dead_code -A clippy::derivable_impls -A clippy::explicit_counter_loop -A clippy::unnecessary_cast -A clippy::unnecessary_mut_passed -A clippy::too_many_arguments -A clippy::type_complexity -A clippy::identity_op -D warnings
	cd scripts/helper && yarn format -c && yarn lint

.PHONY: fix
fix:
	SKIP_WASM_BUILD= cargo fix --all-targets --allow-dirty --allow-staged

.PHONY: fmt
fmt:
	SKIP_WASM_BUILD= cargo fmt --all
	cd scripts/helper && yarn format

.PHONY: resources
resources:
	docker run --rm parallelfinance/parallel:$(DOCKER_TAG) export-genesis-state --chain $(CHAIN) > ./resources/para-$(PARA_ID)-genesis
	docker run --rm parallelfinance/parallel:$(DOCKER_TAG) export-genesis-wasm --chain $(CHAIN) > ./resources/para-$(PARA_ID).wasm

.PHONY: shutdown
shutdown:
	pkill parallel || true
	pkill polkadot || true
	docker-compose \
		-f output/docker-compose.yml \
		-f output/docker-compose.override.yml \
		down \
		--remove-orphans > /dev/null 2>&1 || true
	rm -fr output || true
	rm -fr data || true
	docker volume prune -f

.PHONY: launch
launch: shutdown
	yq -i eval '.relaychain.image = "parallelfinance/polkadot:$(RELAY_DOCKER_TAG)"' $(LAUNCH_CONFIG_YAML)
	yq -i eval '.relaychain.chain = "$(RELAY_CHAIN)"' $(LAUNCH_CONFIG_YAML)
	yq -i eval '.parachains[0].image = "parallelfinance/parallel:$(DOCKER_TAG)"' $(LAUNCH_CONFIG_YAML)
	yq -i eval '.parachains[0].id = $(PARA_ID)' $(LAUNCH_CONFIG_YAML)
	yq -i eval '.parachains[0].chain.base = "$(CHAIN)"' $(LAUNCH_CONFIG_YAML)
	docker image pull parallelfinance/polkadot:$(RELAY_DOCKER_TAG)
	docker image pull parallelfinance/parallel:$(DOCKER_TAG)
	docker image pull parallelfinance/stake-client:latest
	docker image pull parallelfinance/liquidation-client:latest
	docker image pull parallelfinance/oracle-client:latest
	docker image pull parallelfinance/heiko-dapp:latest
	docker image pull parallelfinance/parallel-dapp:latest
	parachain-launch generate $(LAUNCH_CONFIG_YAML) \
		&& (cp -r keystore* output || true) \
		&& cp docker-compose.override.yml output \
		&& cd output \
		&& DOCKER_CLIENT_TIMEOUT=1080 COMPOSE_HTTP_TIMEOUT=1080 PARA_ID=$(PARA_ID) docker-compose up -d --build
	cd scripts/helper && yarn start launch --network $(CHAIN)

.PHONY: launch-vanilla
launch-vanilla:
	make PARA_ID=2085 CHAIN=vanilla-dev RELAY_CHAIN=kusama-local launch

.PHONY: dev-launch
dev-launch: shutdown
	yq -i eval '.relaychain.chain = "$(RELAY_CHAIN)"'  $(LAUNCH_CONFIG_JSON) -j
	yq -i eval '.parachains[0].id = $(PARA_ID)' $(LAUNCH_CONFIG_JSON) -j
	yq -i eval '.parachains[0].chain = "$(CHAIN)"' $(LAUNCH_CONFIG_JSON) -j
	ts-node scripts/polkadot-launch/src/cli.ts config.json

.PHONY: dev-launch-vanilla
dev-launch-vanilla:
	make PARA_ID=2085 CHAIN=vanilla-dev RELAY_CHAIN=kusama-local dev-launch

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
	cargo run --bin parallel key insert -d . --keystore-path $(KEYSTORE_PATH) --suri "$(SURI)" --key-type babe

.PHONY: snapshot
snapshot:
	cargo run --bin parallel --features try-runtime -- try-runtime --chain $(CHAIN) --wasm-execution=compiled on-runtime-upgrade live -a=$(BLOCK_AT) -u=$(URL) -s=snapshot.bin

.PHONY: try-runtime-upgrade
try-runtime-upgrade:
	RUST_LOG=debug cargo run --bin parallel --features try-runtime -- try-runtime --chain $(CHAIN) --wasm-execution=compiled on-runtime-upgrade snap -s snapshot.bin

help:
	@grep -E '^[a-zA-Z_-]+:.*?' Makefile | cut -d: -f1 | sort
