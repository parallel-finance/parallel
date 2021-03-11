#!/usr/bin/env bash

set -e

echo "*** Start building parallel ***"

cd $(dirname ${BASH_SOURCE[0]})/..

docker-compose down --remove-orphans
docker-compose run --rm build $@