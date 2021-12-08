#!/usr/bin/env bash

DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

cd $DIR

set -xe

# https://dot-rocksdb.polkashots.io
# https://ksm-rocksdb.polkashots.io

DB_PATH="polkadot/chains/ksmcc3"
SNAPSHOT_PATH="db/full"
VOLUME="chains"

if [ "$1" == "--polkadot" ]; then
  DB_PATH="polkadot/chains/polkadot"
elif [ "$1" == "--westend" ]; then
  DB_PATH="polkadot/chains/westend2"
fi

# docker volume rm $VOLUME || true
docker volume create $VOLUME || true

mountpoint=$(docker volume inspect $VOLUME | jq '.[].Mountpoint' | tr -d '"')
sudo mkdir -p $mountpoint/$DB_PATH/db || true
sudo rm -fr $mountpoint/$DB_PATH/db/full || true
sudo mv $SNAPSHOT_PATH $mountpoint/$DB_PATH/db

sudo chown -R $(id -un):$(id -gn) $mountpoint
