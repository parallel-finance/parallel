#!/usr/bin/env bash

set -ex

PROFILE=release
DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)
CARGO_HOME=".docker/cargo"
TARGET_BIN="$CARGO_HOME/parallel"

cd $DIR/..

echo "*** Start building parallel ***"
docker run --rm \
    -v "$(pwd)":/parallel \
    -t paritytech/ci-linux:production \
    bash -c "cd /parallel && CARGO_HOME=/parallel/$CARGO_HOME cargo build --$PROFILE --target-dir /parallel/$CARGO_HOME/target"

sudo cp $CARGO_HOME/target/$PROFILE/parallel $TARGET_BIN
sudo chown $(id -un):$(id -gn) $TARGET_BIN

echo "*** Start building parallel image ***"
docker build -t \
    alexcj96/parallel:latest \
    . \
&& {
    echo "*** Updating resources ***"
    $TARGET_BIN build-spec --disable-default-bootnode > ./resources/template-local-plain.json
    $TARGET_BIN build-spec --chain=./resources/template-local-plain.json --raw --disable-default-bootnode > ./resources/template-local.json
    $TARGET_BIN export-genesis-state --parachain-id 200 > ./resources/para-200-genesis
    $TARGET_BIN export-genesis-wasm > ./resources/para-200.wasm
}
