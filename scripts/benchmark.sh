#!/bin/bash

DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

cd $DIR/..

set -xe

steps=50
repeat=20
parallelOutput=./runtime/parallel/src/weights
heikoOutput=./runtime/heiko/src/weights
parallelChain=parallel-dev
heikoChain=heiko-dev

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
  pallet_streaming
  pallet_liquid_staking
  pallet_assets
  pallet_session
  pallet_collator_selection
  pallet_proxy
  pallet_utility
  cumulus_pallet_xcmp_queue
  pallet_prices
  pallet_identity
  pallet_democracy
  pallet_collective
  pallet_preimage
  pallet_scheduler
  pallet_treasury
)

for p in ${pallets[@]}
do
	./target/release/parallel benchmark \
    pallet \
		--chain=$parallelChain \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=$p \
		--extrinsic='*' \
		--steps=$steps \
		--repeat=$repeat \
		--output=$parallelOutput/$p.rs

	./target/release/parallel benchmark \
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
