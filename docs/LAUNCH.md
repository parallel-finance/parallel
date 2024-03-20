# Launch guide

Parallel uses [parachain-launch](https://github.com/open-web3-stack/parachain-launch) to launch our services including:

- `parallel-dapp` : DAPP for money market, staking, crowdloans, cross chain transfer and more
- `stake-client` : Liquid staking pallet's rewards/slashes' feeder
- `oracle-client` : Loans pallet's price feeder
- `liquidation-client` : Loans pallet's liquidation operator
- `polkadot` : Relaychain
- `parallel` : Parachain
- `cumulus` : A dummy parachain

## Getting Started

### Install nodejs, rust, parachain-launch, yq and initialize submodules

1. nodejs, parachain-launch

```
NODE_VERSION=v14.17.0
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash \
    && [ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh" \
    && [ -s "$NVM_DIR/bash_completion" ] && . "$NVM_DIR/bash_completion" \
    && nvm install $NODE_VERSION \
    && nvm use $NODE_VERSION \
    && nvm alias default $NODE_VERSION \
    && npm install -g yarn \
    && yarn global add @open-web3/parachain-launch ts-node
```

2. yq

```
VERSION=v4.2.0
BINARY=yq_linux_amd64
sudo wget https://github.com/mikefarah/yq/releases/download/${VERSION}/${BINARY} -O /usr/bin/yq &&\
    sudo chmod +x /usr/bin/yq
```

3. rust

```
RUST_TOOLCHAIN=nightly-2022-11-15
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain ${RUST_TOOLCHAIN} --component rust-src --target wasm32-unknown-unknown
```

4. Initialize submodules

```
make init
```

### Create .env file with relaychain sudo key

Suppose the relaychain sudo key is `//Alice`, we need to put the following content in `scripts/helper/.env` file

**NOTE**: Please contact a Parallel team member for the relaychain sudo key.

```
RELAY_CHAIN_SUDO_KEY="//Alice"
```

### Launch

Then run the following command to launch all services

```
make launch
```

- [Relaychain](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer)
- [Parachain](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9948#/explorer)
- [DAPP](http://127.0.0.1:8080)

### Port forwarding (optional)

If you are running `make launch` on remote server, you can forward to local. Here is
the bash script to save you time:

```
function forward-port-to-local {
    if [ ! -z "$1" ] && [ ! -z "$2" ]; then
        ssh -N -L ${3:-$2}:localhost:${2} ${1}
    fi
}
```

e.g. forward ubuntu@192.168.1.11's 9944 port to local:

```
forward-port-to-local ubuntu@192.168.1.11 9944
```

Then access everything locally:

- [Relaychain](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer)
- [Parachain](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9948#/explorer)
- [DAPP](http://127.0.0.1:8080)

## Advanced

If you need to adjust relaychain & parachain version or other parameters, you can edit `config.yml` file
If you need to adjust services like `stake-client` etc, you can edit `docker-compose.override.yml`
