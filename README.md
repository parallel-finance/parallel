# <div style="display:flex;align-items:center;"><img src="https://user-images.githubusercontent.com/40745291/113244934-d143c400-926a-11eb-9f2b-dc8350e11a58.png" width="40px" style="border-radius:50%;margin-right:10px;"><span style="transform: translateY(5px);">Parallel Finance</span></div>

[![GitHub last commit](https://img.shields.io/github/last-commit/parallel-finance/parallel)](https://github.com/parallel-finance/parallel/commits/master)
[![CI](https://github.com/parallel-finance/parallel/workflows/CI/badge.svg)](https://github.com/parallel-finance/parallel/actions)
[![Discord chat][discord-badge]][discord-url]

A new Cumulus-based Substrate node, ready for hacking :rocket:

[discord-badge]: https://img.shields.io/discord/830972820846018600.svg?logo=discord&style=flat-square
[discord-url]: https://discord.gg/3WGNW3j2

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

## Run

### Local Testnet

Polkadot (rococo-v1 branch, better use commit hash: `f3e2cbf49f179104d20b9f1b54830710ddac8be3`):

```
cargo build --release --features real-overseer

./target/release/polkadot build-spec --chain rococo-local --raw --disable-default-bootnode > rococo_local.json

./target/release/polkadot --chain ./rococo_local.json -d cumulus_relay0 --validator --alice --port 50555

./target/release/polkadot --chain ./rococo_local.json -d cumulus_relay1 --validator --bob --port 50556 \
        --bootnodes /ip4/127.0.0.1/tcp/50555/p2p/<ALICE's peer id>
```

Substrate Parachain Template:

```
# this command assumes the chain spec is in a directory named polkadot that is a sibling of the working directory
./target/release/parallel -d local-test --collator --alice --ws-port 9915 --parachain-id 200 -- --chain ../polkadot/rococo_local.json \
        --bootnodes /ip4/127.0.0.1/tcp/50555/p2p/<ALICE's peer id>
```

### Registering on Local Relay Chain

In order to produce blocks you will need to register the parachain as detailed in the [Substrate Cumulus Workshop](https://substrate.dev/cumulus-workshop/#/en/3-parachains/2-register) by going to

Developer -> sudo -> paraSudoWrapper -> sudoScheduleParaInitialize(id, genesis)

Ensure you set the `ParaId` to `200` and the `parachain: Bool` to `Yes`.

The files you will need are in the `./resources` folder, if you need to build them because you modified the code you can use the following commands

```
cargo build --release
# Build the Chain spec
./target/release/parallel build-spec --disable-default-bootnode > ./resources/template-local-plain.json
# Build the raw file
./target/release/parallel build-spec --chain=./resources/template-local-plain.json --raw --disable-default-bootnode > ./resources/template-local.json


# export genesis state and wasm
./target/release/parallel export-genesis-state --parachain-id 200 > ./resources/para-200-genesis
./target/release/parallel export-genesis-wasm > ./resources/para-200.wasm
```

### Embedded Docs

Once the project has been built, the following command can be used to explore all parameters and
subcommands:

```sh
./target/release/parallel -h
```

## Learn More

Refer to the upstream
[Substrate Developer Hub Node Template](https://github.com/substrate-developer-hub/substrate-node-template)
to learn more about the structure of this project, the capabilities it encapsulates and the way in
which those capabilities are implemented. You can learn more about
[The Path of Parachain Block](https://polkadot.network/the-path-of-a-parachain-block/) on the
official Polkadot Blog.
