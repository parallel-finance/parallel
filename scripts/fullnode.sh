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

PARA_ID=2085

PARA_CHAIN="heiko"
RELAY_CHAIN="kusama"
VOLUME="chains"

docker container stop heiko-fullnode || true
docker container rm heiko-fullnode || true

# docker volume rm $VOLUME || true

docker volume create $VOLUME || true

docker run --restart=always --name heiko-fullnode \
  -d \
  -p $PARA_WS_PORT:$PARA_WS_PORT \
  -p $PARA_RPC_PORT:$PARA_RPC_PORT \
  -p $PARA_P2P_PORT:$PARA_P2P_PORT \
  -p $RELAY_WS_PORT:$RELAY_WS_PORT \
  -p $RELAY_RPC_PORT:$RELAY_RPC_PORT \
  -p $RELAY_P2P_PORT:$RELAY_P2P_PORT \
  -v "$VOLUME:/data" \
  parallelfinance/parallel:v1.0.0 \
    -d /data \
    --chain=$PARA_CHAIN \
    --parachain-id=$PARA_ID \
    --ws-port=$PARA_WS_PORT \
    --rpc-port=$PARA_RPC_PORT \
    --ws-external \
    --rpc-external \
    --rpc-cors all \
    --pruning archive \
    --wasm-execution=compiled \
    --ws-max-connections 1024 \
    --execution=wasm \
    --listen-addr=/ip4/0.0.0.0/tcp/$PARA_P2P_PORT \
  -- \
    --chain=$RELAY_CHAIN \
    --ws-port=$RELAY_WS_PORT \
    --rpc-port=$RELAY_RPC_PORT \
    --ws-external \
    --rpc-external \
    --rpc-cors all \
    --wasm-execution=compiled \
    --execution=wasm \
    --pruning archive \
    --listen-addr=/ip4/0.0.0.0/tcp/$RELAY_P2P_PORT

# docker logs -f heiko-fullnode
