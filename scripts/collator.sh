#!/usr/bin/env bash

DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

cd $DIR

set -xe

RELAY_WS_PORT=9945
RELAY_RPC_PORT=9934
RELAY_P2P_PORT=30334

PARA_WS_PORT=9944
PARA_RPC_PORT=9933
PARA_P2P_PORT=30333

PARA_CHAIN="${4:-heiko}"
RELAY_CHAIN="${5:-kusama}"
VOLUME="chains"
NODE_KEY="$1"
KEYSTORE_PATH="$2"
NODE_NAME="$3"
DOCKER_IMAGE="parallelfinance/parallel:v1.8.0"
BASE_PATH="/data"

if [ $# -lt 3 ]; then
  echo "help: ./collator.sh <NODE_KEY> <KEYSTORE_PATH> <NODE_NAME>" && exit 1
fi

docker container stop $PARA_CHAIN-collator || true
docker container rm $PARA_CHAIN-collator || true

# docker volume rm $VOLUME || true

docker volume create $VOLUME || true

docker run --restart=always --name $PARA_CHAIN-collator \
  -d \
  -p $PARA_WS_PORT:$PARA_WS_PORT \
  -p $PARA_RPC_PORT:$PARA_RPC_PORT \
  -p $PARA_P2P_PORT:$PARA_P2P_PORT \
  -p $RELAY_WS_PORT:$RELAY_WS_PORT \
  -p $RELAY_RPC_PORT:$RELAY_RPC_PORT \
  -p $RELAY_P2P_PORT:$RELAY_P2P_PORT \
  -v "$VOLUME:$BASE_PATH" \
  -v "$(realpath $KEYSTORE_PATH):/app/keystore" \
  $DOCKER_IMAGE \
    -d $BASE_PATH \
    --chain=$PARA_CHAIN \
    --collator \
    --ws-port=$PARA_WS_PORT \
    --rpc-port=$PARA_RPC_PORT \
    --keystore-path=/app/keystore \
    --node-key=$NODE_KEY \
    --pruning archive \
    --wasm-execution=compiled \
    --execution=wasm \
    --ws-external \
    --rpc-external \
    --rpc-cors all \
    --rpc-methods Unsafe \
    --state-cache-size 0 \
    --listen-addr=/ip4/0.0.0.0/tcp/$PARA_P2P_PORT \
    --name=$NODE_NAME \
    --prometheus-external \
  -- \
    --chain=$RELAY_CHAIN \
    --ws-port=$RELAY_WS_PORT \
    --rpc-port=$RELAY_RPC_PORT \
    --wasm-execution=compiled \
    --execution=wasm \
    --database=RocksDb \
    --state-cache-size 0 \
    --unsafe-pruning \
    --pruning=1000 \
    --listen-addr=/ip4/0.0.0.0/tcp/$RELAY_P2P_PORT \
    --name="${NODE_NAME}_Embedded_Relay"

# docker logs -f $PARA_CHAIN-collator
