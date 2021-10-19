#!/bin/bash

DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

cd $DIR/..

set -xe

steps=50
repeat=20
parallelOutput=./runtime/parallel/src/weights/
heikoOutput=./runtime/heiko/src/weights/
parallelChain=parallel
heikoChain=heiko
pallets=(
	frame_system
	pallet_balances
	pallet_timestamp
	pallet_multisig
	pallet_membership
)

for p in ${pallets[@]}
do
	cargo run --release --features runtime-benchmarks -- benchmark \
		--chain=$parallelChain \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=$p \
		--extrinsic='*' \
		--steps=$steps \
		--repeat=$repeat \
		--raw \
		--output=$parallelOutput/$p.rs

	cargo run --release --features runtime-benchmarks -- benchmark \
		--chain=$heikoChain \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=$p \
		--extrinsic='*' \
		--steps=$steps \
		--repeat=$repeat \
		--raw \
		--output=$heikoOutput/$p.rs
done
