#!/usr/bin/env bash

DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

cd $DIR/../

set -xe

RUSTC_VERSION=1.62.0;
PACKAGE=${PACKAGE:-vanilla-runtime};
BUILD_OPTS=$BUILD_OPTS;

docker run --rm -i \
  -e PACKAGE=$PACKAGE \
  -e BUILD_OPTS="$BUILD_OPTS" \
  -v $PWD:/build \
  -v $TMPDIR/cargo:/cargo-home \
  --user root \
  --network=host \
  paritytech/srtool:$RUSTC_VERSION
