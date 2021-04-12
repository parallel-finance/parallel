#!/usr/bin/env bash

set -ex

SCRIPT_DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)
ROOT="$(realpath $SCRIPT_DIR/..)"
TMP_DIR="dist"
BIN="target/release/parallel"
SYSTEM="$(uname -s)"
TAR="tar"

if [ $SYSTEM == "Darwin" ]; then
    TAR="gtar"
fi

function get_pkg_metadata() {
    local val=$(sed -ne "s/^\s*$1.*'\(.*\)'/\1/p" node/Cargo.toml |  sed -n "$2p")
    echo $val
}

cd $ROOT

COMMIT_HASH="$(git rev-parse --short HEAD)"
PKG_NAME="$(get_pkg_metadata "name" 1)"
PKG_VER="$(get_pkg_metadata "version" 1)"
PKG_XZ="$PKG_NAME-$PKG_VER-$COMMIT_HASH.tar.xz"

if [ ! -f $BIN ]; then
    echo "No binary found in .cargo, run scripts/build-image.sh to generate it" && exit 1
fi

INCLUDES=( \
    "README.md" \
    "$BIN" \
    "scripts/setup.sh" \
    "resources"
)
FLAGS="-cvJf"

# Coping files
if [ ! -d "$TMP_DIR" ]; then
    mkdir -p $TMP_DIR
fi

for INCLUDE in "${INCLUDES[@]}"
do
    cp -r $ROOT/$INCLUDE $TMP_DIR
done

$BIN export-genesis-state --parachain-id 200 > $TMP_DIR/resources/para-200-genesis
$BIN export-genesis-wasm > $TMP_DIR/resources/para-200.wasm

# Package
CMD="tar $FLAGS $PKG_XZ $TMP_DIR"
CMD="$CMD --transform 's|$TMP_DIR|$PKG_NAME|'"

bash -c "$CMD"

if [ -d "$TMP_DIR" ]; then
    rm -fr $TMP_DIR
fi
