#!/usr/bin/env bash

set -ex

DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

cd $DIR/..

echo "*** Start parallel node ***"

docker-compose down --remove-orphans
docker-compose up -d
# docker-compose logs -f
