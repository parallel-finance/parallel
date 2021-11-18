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

PARA_CHAIN="${2:-heiko}"
RELAY_CHAIN="${3:-kusama}"
VOLUME="chains"
NODE_NAME="$1"
RESERVED_PARAM=""

if [ $# -lt 1 ]; then
  echo "help: ./fullnode.sh <NODE_NAME>" && exit 1
fi

if [[ "$PARA_CHAIN" == "parallel" ]]; then
  PARA_ID=2012
  RESERVED_PARAM="--reserved-only \
    --reserved-nodes /dns/bootnode-0.parallel.fi/tcp/30333/p2p/12D3KooWNngQxhrT19QqK2dCPtCQb5kB92RscWMnPfNxCC1sgr3N \
    --reserved-nodes /dns/bootnode-1.parallel.fi/tcp/30333/p2p/12D3KooWMzctxpmtti9dWsPaosh2cPCBZFUGeQhmT6W1ynErwKKB \
    --reserved-nodes /dns/bootnode-2.parallel.fi/tcp/30333/p2p/12D3KooWAWRTCjiVo3VoSZYMCwKk6CQCSLTqVVjBnbWhvp71Ey6Y \
    --reserved-nodes /dns/bootnode-3.parallel.fi/tcp/30333/p2p/12D3KooWSMKQCs6JXjVdaqBSyoMZLBNWrLjJ3QzTET7Zd7kWoB8G \
    --reserved-nodes /dns/bootnode-4.parallel.fi/tcp/30333/p2p/12D3KooWCAhW29HjprkLmQ39gCTJmHsEWSqLXPkCz27qVbsGjpLk"
elif [[ "$PARA_CHAIN" == "heiko" ]]; then
  RESERVED_PARAM="--reserved-only \
    --reserved-nodes /dns/heiko-bootnode-0.parallel.fi/tcp/30333/p2p/12D3KooWLUTzbrJJDowUKMPfEZrDY6eH8HXvm8hrG6YrdUmdrKPz \
    --reserved-nodes /dns/heiko-bootnode-1.parallel.fi/tcp/30333/p2p/12D3KooWEckTASdnkQC8MfBNnzKGfQJmdmzCBWrwra26nTqY4Hmu \
    --reserved-nodes /dns/heiko-bootnode-2.parallel.fi/tcp/30333/p2p/12D3KooWFJe4LfS15nTBUduq3cMKmHEWwKYrJFmMnAa7wT5W1eZE \
    --reserved-nodes /dns/heiko-bootnode-3.parallel.fi/tcp/30333/p2p/12D3KooWA8jSwEbscptbwv1KqY7d7n2qURbd6zUaaPvzTVBMMgSd \
    --reserved-nodes /dns/heiko-bootnode-4.parallel.fi/tcp/30333/p2p/12D3KooWPmc7C5qkcxLzw5qWuxHM4SQs16w9Ecdy6b6zpPzpuPhX \
    --reserved-nodes /dns/heiko-bootnode-5.parallel.fi/tcp/30333/p2p/12D3KooWBPS34UM3bbv82hfL3LJq7eioRjFSJp6JArGnEj4ZhukN \
    --reserved-nodes /dns/heiko-bootnode-6.parallel.fi/tcp/30333/p2p/12D3KooWNQD9ejZBon81yJuLeV6PWekVqVPX6B72UPepQzWTh8mX \
    --reserved-nodes /dns/heiko-bootnode-7.parallel.fi/tcp/30333/p2p/12D3KooWL63x8ZPkY2ZekUqyvyNwsakwbuy8Rq3Dt9tJcxw5NFTt"
fi

docker container stop $PARA_CHAIN-fullnode || true
docker container rm $PARA_CHAIN-fullnode || true

# docker volume rm $VOLUME || true

docker volume create $VOLUME || true

docker run --restart=always --name $PARA_CHAIN-fullnode \
  -d \
  -p $PARA_WS_PORT:$PARA_WS_PORT \
  -p $PARA_RPC_PORT:$PARA_RPC_PORT \
  -p $PARA_P2P_PORT:$PARA_P2P_PORT \
  -p $RELAY_WS_PORT:$RELAY_WS_PORT \
  -p $RELAY_RPC_PORT:$RELAY_RPC_PORT \
  -p $RELAY_P2P_PORT:$RELAY_P2P_PORT \
  -v "$VOLUME:/data" \
  parallelfinance/parallel:v1.7.2 \
    -d /data \
    --chain=$PARA_CHAIN \
    --parachain-id=$PARA_ID \
    --ws-port=$PARA_WS_PORT \
    --rpc-port=$PARA_RPC_PORT \
    --ws-external \
    --rpc-external \
    --rpc-cors all \
    --ws-max-connections 4096 \
    --pruning archive \
    --wasm-execution=compiled \
    --execution=wasm \
    --state-cache-size 1 \
    --listen-addr=/ip4/0.0.0.0/tcp/$PARA_P2P_PORT \
    --name=$NODE_NAME \
    --prometheus-external \
    ${RESERVED_PARAM} \
  -- \
    --chain=$RELAY_CHAIN \
    --ws-port=$RELAY_WS_PORT \
    --rpc-port=$RELAY_RPC_PORT \
    --ws-external \
    --rpc-external \
    --rpc-cors all \
    --ws-max-connections 4096 \
    --wasm-execution=compiled \
    --execution=wasm \
    --database=RocksDb \
    --unsafe-pruning \
    --pruning=1000 \
    --listen-addr=/ip4/0.0.0.0/tcp/$RELAY_P2P_PORT \
    --name="${NODE_NAME}_Embedded_Relay"

# docker logs -f $PARA_CHAIN-fullnode
