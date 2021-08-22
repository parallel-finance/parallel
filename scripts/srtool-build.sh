#!/usr/bin/env bash

DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

cd $DIR/../

set -xe

RUSTC_VERSION=1.53.0-0.9.16;
PACKAGE=${PACKAGE:-heiko-runtime};
BUILD_OPTS=$BUILD_OPTS;

docker run --rm -it \
  -e PACKAGE=$PACKAGE \
  -e BUILD_OPTS="$BUILD_OPTS" \
  -v $PWD:/build \
  -v $TMPDIR/cargo:/cargo-home \
  --network=host \
  paritytech/srtool:$RUSTC_VERSION
