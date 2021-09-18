#!/bin/bash
pushd $PWD
cd ./launch
yarn && yarn run ts-node index.ts 
popd
