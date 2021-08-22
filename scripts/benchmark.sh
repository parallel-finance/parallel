#!/bin/bash

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
	./target/release/parallel benchmark \
		--chain=$parallelChain \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=$p \
		--extrinsic='*' \
		--steps=$steps \
		--repeat=$repeat \
		--raw \
		--output=$parallelOutput/$p.rs

	./target/release/parallel benchmark \
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
