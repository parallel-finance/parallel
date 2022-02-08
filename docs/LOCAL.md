# Running Locally


### Resources
Clone Polkadot in a directory neighboring the Parallel repo

```bash
git clone git@github.com:paritytech/polkadot.git
```

Your directory should look similar too

```
.
├── parallel
└── polkadot
```

### Build

Build our relay chain with
```bash
cd polkadot
git checkout release-v0.9.16

# building will take a long time
cargo build --release 
```

Build our parachain/collator repo with
```bash
cd ../parallel

# building will take a long time
cargo build --release 
```

### Setup relay

We need to create a spec file for the relay chain. We can create the `rococo_local.json` inside of the polkadot directory with.
```bash
make local-spec
```

### Start Relay

Now we can start the relay chain using our `Makefile`. We'll need to open two new terminals.

In the first terminal
```bash
make local-relay-alice
```

In the second terminal
```bash
make local-relay-bob
```

You should be able to see the running Relay chain at https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer

### Prepare parachain

Build the resources for the chain to connect it to the relay chain
```bash
make dev-genesis-and-wasm
```

### Start parachain

We can start the parachain collator with
```bash
make start-local
```

You should be able to see the running Parachain at https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9915#/explorer

### Connect to relay and product blocks

Although we've started the relay and parachain, we need to connect the parachain to the relay to start producing blocks.

We can run a script that will send a sudo command to connect the parachain to the relay using the `genesis` and `wasm` files above with
```bash
make local-launch
```


# Cleanup

We can clear the relay chain data with

```bash
make clear-local-relays
```

We can clear the parachain data with

```bash
make clear-local-parachain
```
