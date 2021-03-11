#!/usr/bin/env bash

set -e

echo "*** Start build parallelfinance/parallel:latest ***"

cd $(dirname ${BASH_SOURCE[0]})/..

#sh ./docker/build.sh

mkdir -p tmp

cp ./docker/target/release/parallel tmp

docker build -t parallelfinance/parallel:latest .

rm -rf tmp
