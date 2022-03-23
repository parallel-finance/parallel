#!/usr/bin/env bash
set -e
# usage: ./scripts/bump.sh 1.7.9

# The following line ensure we run from the project root
PROJECT_ROOT=`git rev-parse --show-toplevel`
cd ${PROJECT_ROOT}

if [ $# -lt 1 ]; then
  echo "help: ./scripts/bump.sh <VERSION>" && exit 1
fi

FROM=`grep "^version" ./node/parallel/Cargo.toml | egrep -o "([0-9\.]+)"`
TO=${1}

cargo_toml_list=$(find . -name "Cargo.toml" -not -path "./target/*")
echo "bump parallel version from ${FROM} to ${TO}..."
for cargo_toml in $cargo_toml_list
do
    if [ "$(uname)" == "Darwin" ];then  # Mac
        sed -i "" "s/$FROM/$TO/g" ${cargo_toml}
    else
         sed -i "s/$FROM/$TO/g" ${cargo_toml} # Linux
    fi
done
