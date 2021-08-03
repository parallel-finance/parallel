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
	# pallet_sudo
	# pallet_democracy
	# pallet_collective
	# pallet_treasury
	# pallet_scheduler
	# pallet_elections_phragmen
	# pallet_membership
	# pallet_transaction_payment
	# cumulus_pallet_parachain_system
	# parachain_info
	# cumulus_pallet_xcmp_queue
	# cumulus_pallet_dmp_queue
	# pallet_xcm
	# cumulus_pallet_xcm
	# pallet_authorship
	# pallet_collator_selection
	# pallet_session
	# pallet_aura
	# cumulus_pallet_aura_ext
	# pallet_liquidation
	# pallet_prices
	# pallet_membership
	# pallet_nominee_election
)

for p in ${pallets[@]}
do
	./target/release/parallel benchmark \
		--chain=$parallelChain \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=$p  \
		--extrinsic='*' \
		--steps=$steps  \
		--repeat=$repeat \
		--raw  \
		--output=$parallelOutput/$p.rs

	# ./target/release/parallel benchmark \
	# 	--chain=$heikoChain \
	# 	--execution=wasm \
	# 	--wasm-execution=compiled \
	# 	--pallet=$p  \
	# 	--extrinsic='*' \
	# 	--steps=$steps  \
	# 	--repeat=$repeat \
	# 	--raw  \
	# 	--output=$heikoOutput/$p.rs

done
