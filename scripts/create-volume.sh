#!/usr/bin/env bash

DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

cd $DIR

# curl -fLO https://ksm-rocksdb.polkashots.io/kusama-9217113.RocksDb.7z
# wget https://ksm-rocksdb.polkashots.io/kusama-9217113.RocksDb.7z
# sudo apt install p7zip-full
# 7z x kusama-9217113.RocksDb.7z

set -xe

DB_PATH="polkadot/chains/ksmcc3"
SNAPSHOT_PATH="db"
VOLUME="chains"

# docker volume rm $VOLUME || true
docker volume create $VOLUME || true

mountpoint=$(docker volume inspect $VOLUME | jq '.[].Mountpoint' | tr -d '"')
sudo mkdir -p $mountpoint/$DB_PATH
sudo mv $SNAPSHOT_PATH $mountpoint/$DB_PATH/db

sudo chown -R $(id -un):$(id -gn) $mountpoint
