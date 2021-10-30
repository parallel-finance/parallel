#!/usr/bin/env bash

DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

cd $DIR

set -xe

DB_PATH="polkadot/chains/ksmcc3"
SNAPSHOT_PATH="full"
VOLUME="chains"

# docker volume rm $VOLUME || true
docker volume create $VOLUME || true

mountpoint=$(docker volume inspect $VOLUME | jq '.[].Mountpoint' | tr -d '"')
sudo mkdir -p $mountpoint/$DB_PATH/db
sudo rm -fr $mountpoint/$DB_PATH/db/full
sudo mv $SNAPSHOT_PATH $mountpoint/$DB_PATH/db

sudo chown -R $(id -un):$(id -gn) $mountpoint
