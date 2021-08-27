#!/usr/bin/env bash

DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

cd $DIR

set -xe

RELAY_WS_PORT=9950
RELAY_RPC_PORT=9939
RELAY_P2P_PORT=30339

PARA_WS_PORT=9951
PARA_RPC_PORT=9940
PARA_P2P_PORT=30340

PARA_ID=2085

PARA_CHAIN="heiko"
RELAY_CHAIN="westend"
VOLUME="chains"
NODE_KEY="$1"
KEYSTORE_PATH="$2"

if [ $# -lt 2 ]; then
  echo "help: ./collator.sh <NODE_KEY> <KEYSTORE_PATH>" && exit 1
fi

docker container stop heiko-collator || true
docker container rm heiko-collator || true

# docker volume rm $VOLUME || true

docker volume create $VOLUME || true

docker run --restart=always --name heiko-collator \
  -d \
  -p $PARA_WS_PORT:$PARA_WS_PORT \
  -p $PARA_RPC_PORT:$PARA_RPC_PORT \
  -p $PARA_P2P_PORT:$PARA_P2P_PORT \
  -p $RELAY_WS_PORT:$RELAY_WS_PORT \
  -p $RELAY_RPC_PORT:$RELAY_RPC_PORT \
  -p $RELAY_P2P_PORT:$RELAY_P2P_PORT \
  -v "$VOLUME:/data" \
  -v "$(realpath $KEYSTORE_PATH):/app/keystore" \
  parallelfinance/parallel:latest \
    -d /data \
    --chain=$PARA_CHAIN \
    --validator \
    --parachain-id=$PARA_ID \
    --ws-port=$PARA_WS_PORT \
    --rpc-port=$PARA_RPC_PORT \
    --keystore-path=/app/keystore \
    --node-key=$NODE_KEY \
    --ws-external \
    --rpc-external \
    --rpc-cors all \
    --rpc-methods=Unsafe \
    --unsafe-rpc-external \
    --unsafe-ws-external \
    --force-authoring \
    --wasm-execution=compiled \
    --execution=wasm \
    --listen-addr=/ip4/0.0.0.0/tcp/$PARA_P2P_PORT \
  -- \
    --chain=$RELAY_CHAIN \
    --ws-port=$RELAY_WS_PORT \
    --rpc-port=$RELAY_RPC_PORT \
    --node-key=$NODE_KEY \
    --ws-external \
    --rpc-external \
    --wasm-execution=compiled \
    --execution=wasm \
    --listen-addr=/ip4/0.0.0.0/tcp/$RELAY_P2P_PORT \
    --no-beefy

# docker logs -f heiko-collator
