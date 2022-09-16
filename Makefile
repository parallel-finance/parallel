PARA_ID        											:= 2012
CHAIN          											:= kerria-dev
RELAY_CHAIN                         := polkadot-local
CUMULUS_CHAIN     									:= statemint-dev
RUNTIME        											:= kerria-runtime
BLOCK_AT       											:= 0x0000000000000000000000000000000000000000000000000000000000000000
URL            											:= ws://localhost:9948
KEYSTORE_PATH  											:= keystore
SURI           											:= //Alice
LAUNCH_CONFIG_YAML	  							:= config.yml
LAUNCH_CONFIG_JSON	  							:= config.json
DOCKER_OVERRIDE_YAML                := docker-compose.override.yml
DOCKER_TAG     											:= latest_evm
RELAY_DOCKER_TAG										:= v0.9.24
CUMULUS_DOCKER_TAG									:= v0.9.24

.PHONY: init
init: submodules
	git config advice.ignoredHook false
	git config core.hooksPath .githooks
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2022-05-11 --component rust-src --component rustfmt --component clippy --target wasm32-unknown-unknown
	cargo install cargo-udeps --locked
	cd scripts/helper && yarn
	cd scripts/polkadot-launch && yarn
	cd scripts/evm && npm install

.PHONY: submodules
submodules:
	git submodule update --init --recursive
	git submodule foreach git pull origin master

.PHONY: build
build:
	cargo build --bin parallel

.PHONY: build-release
build-release:
	cargo build --locked --workspace --exclude runtime-integration-tests --bin parallel --release --features runtime-benchmarks --features try-runtime

.PHONY: build-release-with-evm
build-release-with-evm:
	cargo build --locked --workspace --exclude runtime-integration-tests --bin parallel --release --features with-evm-runtime --features runtime-benchmarks --features try-runtime

.PHONY: build-compact-release
build-compact-release:
	cargo build --locked --workspace --exclude runtime-integration-tests --bin parallel --release

.PHONY: build-production
build-production:
	cargo build --locked --workspace --exclude runtime-integration-tests --bin parallel --profile production

.PHONY: build-release-if-not-exists
build-release-if-not-exists:
	if [ ! -f ./target/release/parallel ]; then \
		make build-release; \
	fi

.PHONY: build-evm-release-if-not-exists
build-evm-release-if-not-exists:
	if [ ! -f ./target/release/parallel ]; then \
		make build-release-with-evm; \
	fi

.PHONY: clean
clean:
	cargo clean -p parallel -p vanilla-runtime -p kerria-runtime -p heiko-runtime -p parallel-runtime

.PHONY: ci
ci: check check-with-evm lint check-helper check-wasm test integration-test

.PHONY: check
check:
	SKIP_WASM_BUILD= cargo check --all-targets --features runtime-benchmarks --features try-runtime

.PHONY: check-with-evm
check-with-evm:
	SKIP_WASM_BUILD= cargo check --all-targets --features with-evm-runtime --features runtime-benchmarks --features try-runtime --features testing

.PHONY: check-wasm
check-wasm:
	cargo check -p parallel-runtime -p heiko-runtime --features runtime-benchmarks

.PHONY: check-helper
check-helper:
	cd scripts/helper && yarn && yarn build

.PHONY: test
test:
	SKIP_WASM_BUILD= cargo test --workspace --features runtime-benchmarks --exclude runtime-integration-tests --exclude parallel --exclude parallel-runtime --exclude vanilla-runtime --exclude kerria-runtime --exclude heiko-runtime --exclude pallet-loans-rpc --exclude pallet-loans-rpc-runtime-api --exclude parallel-primitives -- --nocapture

.PHONY: test-with-evm
test-with-evm:
	SKIP_WASM_BUILD= cargo test --workspace --features with-evm-runtime --features runtime-benchmarks --exclude runtime-integration-tests --exclude pallet-loans-rpc --exclude pallet-loans-rpc-runtime-api --exclude parallel-primitives -- --nocapture

.PHONY: test-loans
test-loans:
	SKIP_WASM_BUILD= cargo test -p pallet-loans --lib --no-fail-fast -- --nocapture

.PHONY: test-prices
test-prices:
	SKIP_WASM_BUILD= cargo test -p pallet-prices --lib --no-fail-fast -- --nocapture

.PHONY: test-crowdloans
test-crowdloans:
	SKIP_WASM_BUILD= cargo test -p pallet-crowdloans --lib --no-fail-fast -- --nocapture

.PHONY: integration-test
integration-test:
	RUST_LOG="xcm=trace,xcm-executor=trace,liquidStaking=trace" SKIP_WASM_BUILD= cargo test -p runtime-integration-tests -- --nocapture

.PHONY: integration-test-statemine
integration-test-statemine:
	RUST_LOG="xcm=trace,xcm-executor=trace" SKIP_WASM_BUILD= cargo test -p runtime-integration-tests -- statemine --nocapture

.PHONY: integration-test-kusama-call
integration-test-kusama-call:
	RUST_LOG="xcm=trace,xcm-executor=trace,liquidStaking=trace" SKIP_WASM_BUILD= cargo test -p runtime-integration-tests -- kusama_call --nocapture

.PHONY: integration-test-sibling-transfer
integration-test-sibling-transfer:
	RUST_LOG="xcm=trace,xcm-executor=trace" SKIP_WASM_BUILD= cargo test -p runtime-integration-tests -- polkadot_transfer --nocapture

.PHONY: test-pallet-evm-precompile-assets-erc20
test-pallet-evm-precompile-assets-erc20:
	SKIP_WASM_BUILD= cargo test -p pallet-evm-precompile-assets-erc20 --lib --no-fail-fast -- --nocapture

.PHONY: test-pallet-evm-precompile-balances-erc20
test-pallet-evm-precompile-balances-erc20:
	SKIP_WASM_BUILD= cargo test -p pallet-evm-precompile-balances-erc20 --lib --no-fail-fast -- --nocapture

.PHONY: test-pallet-evm-signatures
test-pallet-evm-signatures:
	SKIP_WASM_BUILD= cargo test -p pallet-evm-signatures --lib --no-fail-fast -- --nocapture

.PHONY: bench
bench:build-release-if-not-exists
	./scripts/benchmark.sh

.PHONY: bench-farming
bench-farming: build-evm-release-if-not-exists
	./target/release/parallel benchmark pallet --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-farming --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/farming/src/weights.rs

.PHONY: bench-loans
bench-loans: build-evm-release-if-not-exists
	./target/release/parallel benchmark pallet --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-loans --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/loans/src/weights.rs

.PHONY: bench-crowdloans
bench-crowdloans: build-evm-release-if-not-exists
	./target/release/parallel benchmark pallet --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-crowdloans --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/crowdloans/src/weights.rs

.PHONY: bench-bridge
bench-bridge: build-evm-release-if-not-exists
	./target/release/parallel benchmark pallet --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-bridge --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/bridge/src/weights.rs

.PHONY: bench-xcm-helper
bench-xcm-helper: build-evm-release-if-not-exists
	./target/release/parallel benchmark pallet --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-xcm-helper --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/xcm-helper/src/weights.rs

.PHONY: bench-amm
bench-amm: build-evm-release-if-not-exists
	./target/release/parallel benchmark pallet --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-amm --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/amm/src/weights.rs

.PHONY: bench-liquid-staking
bench-liquid-staking: build-evm-release-if-not-exists
	./target/release/parallel benchmark pallet --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-liquid-staking --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/liquid-staking/src/weights.rs

.PHONY: bench-amm-router
bench-amm-router: build-evm-release-if-not-exists
	./target/release/parallel benchmark pallet --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-router --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/router/src/weights.rs

.PHONY: bench-streaming
bench-streaming: build-evm-release-if-not-exists
	./target/release/parallel benchmark pallet --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-streaming --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/streaming/src/weights.rs

.PHONY: bench-asset-registry
bench-asset-registry: build-evm-release-if-not-exists
	./target/release/parallel benchmark pallet --chain=$(CHAIN) --execution=wasm --wasm-execution=compiled --pallet=pallet-asset-registry --extrinsic='*' --steps=50 --repeat=20 --heap-pages=4096 --template=./.maintain/frame-weight-template.hbs --output=./pallets/asset-registry/src/weights.rs

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

.PHONY: udeps
udeps:
	SKIP_WASM_BUILD= cargo udeps -q --all-targets

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
	sudo rm -fr output || true
	rm -fr data || true
	docker volume prune -f

.PHONY: launch
launch: shutdown
	yq -i eval '.relaychain.image = "parallelfinance/polkadot:$(RELAY_DOCKER_TAG)"' $(LAUNCH_CONFIG_YAML)
	yq -i eval '.relaychain.chain = "$(RELAY_CHAIN)"' $(LAUNCH_CONFIG_YAML)
	yq -i eval '.relaychain.runtimeGenesisConfig.hrmp.preopenHrmpChannels[0].sender = $(PARA_ID)' $(LAUNCH_CONFIG_YAML)
	yq -i eval '.relaychain.runtimeGenesisConfig.hrmp.preopenHrmpChannels[1].recipient = $(PARA_ID)' $(LAUNCH_CONFIG_YAML)
	yq -i eval '.parachains[0].image = "parallelfinance/parallel:$(DOCKER_TAG)"' $(LAUNCH_CONFIG_YAML)
	yq -i eval '.parachains[0].id = $(PARA_ID)' $(LAUNCH_CONFIG_YAML)
	yq -i eval '.parachains[0].chain.base = "$(CHAIN)"' $(LAUNCH_CONFIG_YAML)
	yq -i eval '.parachains[1].image = "parallelfinance/cumulus:$(CUMULUS_DOCKER_TAG)"' $(LAUNCH_CONFIG_YAML)
	yq -i eval '.parachains[1].chain.base = "$(CUMULUS_CHAIN)"' $(LAUNCH_CONFIG_YAML)
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
	make PARA_ID=2085 CHAIN=vanilla-dev RELAY_CHAIN=kusama-local CUMULUS_CHAIN=statemine-dev launch

.PHONY: dev-launch
dev-launch: shutdown
	yq -i eval '.relaychain.chain = "$(RELAY_CHAIN)"'  $(LAUNCH_CONFIG_JSON) -j
	yq -i eval '.parachains[0].id = $(PARA_ID)' $(LAUNCH_CONFIG_JSON) -j
	yq -i eval '.parachains[0].chain = "$(CHAIN)"' $(LAUNCH_CONFIG_JSON) -j
	ts-node scripts/polkadot-launch/src/cli.ts config.json

.PHONY: dev-launch-vanilla
dev-launch-vanilla:
	make PARA_ID=2085 CHAIN=vanilla-dev RELAY_CHAIN=kusama-local dev-launch

.PHONY: launch-evm
launch-evm:
	cargo run --locked --bin parallel --features with-evm-runtime --features runtime-benchmarks --features try-runtime -- --tmp --alice --dev --rpc-cors all --unsafe-ws-external --unsafe-rpc-external --ws-port 19944 --rpc-port 29933

.PHONY: provisioning-evm
provisioning-evm:
	cd scripts/evm && npm run build && npm run init-chain && npm run provisioning-chain

.PHONY: test-evm
test-evm:
	cd scripts/evm && npm run test-chain

.PHONY: logs
logs:
	docker-compose -f output/docker-compose.yml logs -f

.PHONY: wasm
wasm:
	PACKAGE=$(RUNTIME) ./scripts/srtool-build.sh

.PHONY: spec
spec:
	docker run --rm parallelfinance/parallel:$(DOCKER_TAG) build-spec --chain $(CHAIN) --disable-default-bootnode --raw > ./resources/$(CHAIN)-raw.json

.PHONY: production-image
production-image:
	DOCKER_BUILDKIT=1 docker build --build-arg BIN=parallel \
		-c 512 \
		-t parallelfinance/parallel:latest \
		-f Dockerfile.release .

.PHONY: integration-image-with-evm
integration-image-with-evm:
	DOCKER_BUILDKIT=1 docker build --build-arg BIN=parallel \
		-c 512 \
		-t parallelfinance/parallel:latest_evm \
		-f Dockerfile.evm .

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
	cargo run --bin parallel --release --features try-runtime --features runtime-benchmarks -- try-runtime --chain $(CHAIN) --wasm-execution=compiled on-runtime-upgrade live -a=$(BLOCK_AT) -u=$(URL) -s=snapshot.bin

.PHONY: try-snapshot-upgrade
try-snapshot-upgrade:
	cargo run --bin parallel --release --features try-runtime --features runtime-benchmarks -- try-runtime --chain $(CHAIN) --wasm-execution=compiled on-runtime-upgrade snap -s snapshot.bin

.PHONY: try-live-upgrade
try-live-upgrade:
	cargo run --bin parallel --release --features try-runtime --features runtime-benchmarks -- try-runtime --chain $(CHAIN) --wasm-execution=compiled on-runtime-upgrade live --uri=$(URL)

.PHONY: try-heiko-live-upgrade
try-heiko-live-upgrade:
	make CHAIN=heiko-dev URL=wss://heiko-rpc.parallel.fi:443 try-live-upgrade

.PHONY: try-parallel-live-upgrade
try-parallel-live-upgrade:
	make CHAIN=parallel-dev URL=wss://rpc.parallel.fi:443 try-live-upgrade

help:
	@grep -E '^[a-zA-Z_-]+:.*?' Makefile | cut -d: -f1 | sort
