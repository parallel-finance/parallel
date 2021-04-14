#!/usr/bin/env bash

set -ex

PROFILE=release
DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

cd $DIR/..

echo "*** Start building parallel ***"
docker run --rm \
    -v "$(pwd)":/parallel \
    -t paritytech/ci-linux:production \
    bash -c "cd /parallel && CARGO_HOME=/parallel/.cargo cargo build --$PROFILE --target-dir /parallel/.cargo/target"

sudo cp .cargo/target/$PROFILE/parallel .cargo/parallel
sudo chown $(id -un):$(id -gn) .cargo/parallel

echo "*** Start building parallel image ***"
docker build -t \
    alexcj96/parallel:latest \
    . \
&& {
    echo "*** Updating resources ***"
    .cargo/parallel build-spec --disable-default-bootnode > ./resources/template-local-plain.json
    .cargo/parallel build-spec --chain=./resources/template-local-plain.json --raw --disable-default-bootnode > ./resources/template-local.json
    .cargo/parallel export-genesis-state --parachain-id 200 > ./resources/para-200-genesis
    .cargo/parallel export-genesis-wasm > ./resources/para-200.wasm
}
