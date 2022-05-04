#!/bin/bash

DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

cd $DIR/..

set -xe

steps=50
repeat=20
parallelOutput=./runtime/parallel/src/weights
heikoOutput=./runtime/heiko/src/weights
vanillaOutput=./runtime/vanilla/src/weights
kerriaOutput=./runtime/kerria/src/weights
parallelChain=parallel-dev
heikoChain=heiko-dev
vanillaChain=vanilla-dev
kerriaChain=kerria-dev

pallets=(
  frame_system
  pallet_balances
  pallet_timestamp
  pallet_multisig
  pallet_membership
  pallet_amm
  pallet_asset_registry
  pallet_bridge
  pallet_crowdloans
  pallet_farming
  pallet_loans
  pallet_router
  pallet_xcm_helper
  pallet_liquid_staking
)

for p in ${pallets[@]}
do
	cargo run --release --features runtime-benchmarks -- benchmark \
    pallet \
		--chain=$vanillaChain \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=$p \
		--extrinsic='*' \
		--steps=$steps \
		--repeat=$repeat \
		--output=$vanillaOutput/$p.rs

	cargo run --release --features runtime-benchmarks -- benchmark \
    pallet \
		--chain=$kerriaChain \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=$p \
		--extrinsic='*' \
		--steps=$steps \
		--repeat=$repeat \
		--output=$kerriaOutput/$p.rs

	cargo run --release --features runtime-benchmarks -- benchmark \
    pallet \
		--chain=$parallelChain \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=$p \
		--extrinsic='*' \
		--steps=$steps \
		--repeat=$repeat \
		--output=$parallelOutput/$p.rs

	cargo run --release --features runtime-benchmarks -- benchmark \
    pallet \
		--chain=$heikoChain \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=$p \
		--extrinsic='*' \
		--steps=$steps \
		--repeat=$repeat \
		--output=$heikoOutput/$p.rs
done
