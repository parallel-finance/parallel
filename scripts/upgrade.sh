#!/usr/bin/env bash
set -e
# usage: ./scripts/upgrade.sh v0.9.17

# The following line ensure we run from the project root
PROJECT_ROOT=`git rev-parse --show-toplevel`
cd ${PROJECT_ROOT}

if [ $# -lt 1 ]; then
  echo "help: ./scripts/upgrade.sh <VERSION>" && exit 1
fi

FROM=`grep "^substrate-build-script-utils" ./node/parallel/Cargo.toml | egrep -o "(v[0-9\.]+)"`
TO=${1}

cargo_toml_list=$(find . -name "Cargo.toml" -not -path "./target/*")
echo "upgrading substrate dependencies from ${FROM} to ${TO}..."
for cargo_toml in $cargo_toml_list
do
    if [ "$(uname)" == "Darwin" ];then  # Mac
        sed -i "" "s/$FROM/$TO/g" ${cargo_toml}
    else
         sed -i "s/$FROM/$TO/g" ${cargo_toml} # Linux
    fi
done
