#!/usr/bin/env bash

set -e

echo "*** Start parallel node ***"

cd $(dirname ${BASH_SOURCE[0]})/..

docker-compose down --remove-orphans
docker-compose run --rm --service-ports dev $@