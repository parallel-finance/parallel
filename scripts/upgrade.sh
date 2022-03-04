#!/usr/bin/env bash
set -e
# usage: TO=v0.9.18 scripts/upgrade.sh

# The following line ensure we run from the project root
PROJECT_ROOT=`git rev-parse --show-toplevel`
cd ${PROJECT_ROOT}


FROM=`grep "^substrate-build-script-utils" ./node/parallel/Cargo.toml | egrep -o "(v[0-9\.]+)"`
TO=${TO}

cargo_toml_list=$(find . -name "Cargo.toml" -not -path "./target/*")
echo "upgrade substrate from polkadot-${FROM} to polkadot-${TO}"
for cargo_toml in $cargo_toml_list
do
    if [ "$(uname)" == "Darwin" ];then  # Mac
        sed -i "" "s/$FROM/$TO/g" ${cargo_toml}
    else
         sed -i "s/$FROM/$TO/g" ${cargo_toml} # Linux
    fi
done
