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

PARA_CHAIN="${3:-vanilla-dev.json}"
RELAY_CHAIN="${4:-kusama-local.json}"
VOLUME="chains"
NODE_NAME="$1"
COLLATOR_NAME="${2:-alice}"
DOCKER_IMAGE="parallelfinance/parallel:v1.8.0"
BASE_PATH="/data"
RELAY_BOOTNODES="/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWDEyCAUKviazJuXdWcAAVEf7nSm9BvPXyK6odp5PetCfV"

if [ $# -lt 1 ]; then
  echo "help: ./collator-dev.sh <NODE_NAME>" && exit 1
fi

docker container stop $NODE_NAME || true
docker container rm $NODE_NAME || true

# docker volume rm $VOLUME || true

docker volume create $VOLUME || true

docker run --restart=always --name $NODE_NAME \
  -d \
  -p $PARA_WS_PORT:$PARA_WS_PORT \
  -p $PARA_RPC_PORT:$PARA_RPC_PORT \
  -p $PARA_P2P_PORT:$PARA_P2P_PORT \
  -p $RELAY_WS_PORT:$RELAY_WS_PORT \
  -p $RELAY_RPC_PORT:$RELAY_RPC_PORT \
  -p $RELAY_P2P_PORT:$RELAY_P2P_PORT \
  -v "$VOLUME:$BASE_PATH" \
  -v "$(pwd):/app" \
  $DOCKER_IMAGE \
    -d $BASE_PATH \
    --chain=/app/$PARA_CHAIN \
    --collator \
    --$COLLATOR_NAME \
    --ws-port=$PARA_WS_PORT \
    --rpc-port=$PARA_RPC_PORT \
    --pruning archive \
    --wasm-execution=compiled \
    --force-authoring \
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
    --chain=/app/$RELAY_CHAIN \
    --ws-port=$RELAY_WS_PORT \
    --rpc-port=$RELAY_RPC_PORT \
    --wasm-execution=compiled \
    --execution=wasm \
    --database=RocksDb \
    --state-cache-size 0 \
    --unsafe-pruning \
    --pruning=1000 \
    --listen-addr=/ip4/0.0.0.0/tcp/$RELAY_P2P_PORT \
    --name="${NODE_NAME}_Embedded_Relay" \
    --bootnodes $RELAY_BOOTNODES

docker logs -f $NODE_NAME
