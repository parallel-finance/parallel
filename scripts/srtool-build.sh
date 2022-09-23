#!/usr/bin/env bash

DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

cd $DIR/../

set -xe

RUSTC_VERSION=1.62.0;
PACKAGE=${PACKAGE:-vanilla-runtime};
BUILD_OPTS=$BUILD_OPTS;

CMD="docker run \
  -i \
  --rm \
  -e PACKAGE=${PACKAGE} \
  -e BUILD_OPTS="${BUILD_OPTS}" \
  -v ${PWD}:/build \
  -v ${TMPDIR}/cargo:/cargo-home \
  --user root \
  --network=host \
  paritytech/srtool:${RUSTC_VERSION} \
    build --app --json -cM"

stdbuf -oL $CMD | {
  while IFS= read -r line
  do
      echo ║ $line
      JSON="$line"
  done

  echo ::set-output name=json::$JSON

  PROP=`echo $JSON | jq -r .runtimes.compact.prop`
  echo ::set-output name=proposal_hash::$PROP

  WASM=`echo $JSON | jq -r .runtimes.compact.wasm`
  echo ::set-output name=wasm::$WASM

  Z_WASM=`echo $JSON | jq -r .runtimes.compressed.wasm`
  echo ::set-output name=wasm_compressed::$Z_WASM

  IPFS=`echo $JSON | jq -r .runtimes.compact.ipfs`
  echo ::set-output name=ipfs::$IPFS
}