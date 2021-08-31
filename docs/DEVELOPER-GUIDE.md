# Developer Guide

## Getting Started

Follow these steps to get started with the Cumulus Template :hammer_and_wrench:

### Setup

First, complete the [basic Rust setup instructions](./doc/rust-setup.md).

If necessary, refer to the setup instructions at the
[Substrate Developer Hub](https://substrate.dev/docs/en/knowledgebase/getting-started/#manual-installation).

### Build

Once the development environment is set up, build the node template. This command will build the
[Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution) and
[native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

```bash
cargo build --release
```

### Available commands

```
make help
```

## Run Heiko Node (manually)

### Local Testnet

Polkadot (v0.9.9-1 branch)

```
cargo build --release

./target/release/polkadot build-spec --chain rococo-local --raw --disable-default-bootnode > rococo_local.json

./target/release/polkadot --chain ./rococo_local.json -d cumulus_relay0 --validator --alice --port 50555 --node-key 0000000000000000000000000000000000000000000000000000000000000001


./target/release/polkadot --chain ./rococo_local.json -d cumulus_relay1 --validator --bob --port 50556 \
        --bootnodes /ip4/127.0.0.1/tcp/50555/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

Substrate Parachain Template:

```
# this command assumes the chain spec is in a directory named polkadot that is a sibling of the working directory
./target/release/parallel -d local-test --collator --alice --chain heiko-dev --ws-port 9915 --parachain-id 2085 -- --chain ../polkadot/rococo_local.json \
        --bootnodes /ip4/127.0.0.1/tcp/50555/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

### Registering on Local Relay Chain

In order to produce blocks you will need to register the parachain as detailed in the [Substrate Cumulus Workshop](https://substrate.dev/cumulus-workshop/#/en/3-parachains/2-register) by going to

Developer -> sudo -> paraSudoWrapper -> sudoScheduleParaInitialize(id, genesis)

Ensure you set the `ParaId` to `2085` and the `parachain: Bool` to `Yes`.

The files you will need are in the `./resources` folder, if you need to build them because you modified the code you can use the following commands

```
cargo build --release
# Build the Chain spec
./target/release/parallel build-spec --chain heiko-dev --disable-default-bootnode > ./resources/template-local-plain.json
# Build the raw file
./target/release/parallel build-spec --chain=./resources/template-local-plain.json --raw --disable-default-bootnode > ./resources/template-local.json


# export genesis state and wasm
./target/release/parallel export-genesis-state --chain heiko-dev --parachain-id 2085 > ./resources/para-2085-genesis
./target/release/parallel export-genesis-wasm --chain heiko-dev > ./resources/para-2085.wasm
```

### Embedded Docs

Once the project has been built, the following command can be used to explore all parameters and
subcommands:

```sh
./target/release/parallel -h
```

### Docker

Run Heiko Dev Network (via parachain-launch 1.0.3)

```
make launch
```

Generate heiko-dev's genesis state & wasm

```
docker run --rm  parity/polkadot:latest build-spec --chain rococo-local --raw --disable-default-bootnode > rococo-local.json

docker run --rm  parallelfinance/parallel:latest export-genesis-state --chain heiko-dev --parachain-id 2085 > ./para-2085-genesis
docker run --rm  parallelfinance/parallel:latest export-genesis-wasm --chain heiko-dev > ./para-2085.wasm
```

### Wasm

```
make wasm
make PACKAGE=parallel-runtime wasm
```

### Image

```
make image
```
