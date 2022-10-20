
//! Autogenerated weights for `pallet_loans`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-19, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `ip-172-88-3-164`, CPU: `Intel(R) Xeon(R) Platinum 8124M CPU @ 3.00GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("heiko-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/parallel
// benchmark
// pallet
// --chain=heiko-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_loans
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/heiko/src/weights/pallet_loans.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_loans`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_loans::WeightInfo for WeightInfo<T> {
	// Storage: Loans Markets (r:2 w:1)
	// Storage: Loans UnderlyingAssetId (r:1 w:1)
	// Storage: Loans ExchangeRate (r:0 w:1)
	// Storage: Loans BorrowIndex (r:0 w:1)
	fn add_market() -> Weight {
		(44_186_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: Loans Markets (r:1 w:1)
	fn activate_market() -> Weight {
		(30_552_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Loans Markets (r:1 w:1)
	fn update_rate_model() -> Weight {
		(31_971_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Loans Markets (r:1 w:1)
	fn update_market() -> Weight {
		(34_016_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Loans UnderlyingAssetId (r:1 w:1)
	// Storage: Loans Markets (r:1 w:1)
	fn force_update_market() -> Weight {
		(40_663_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: System Account (r:1 w:1)
	fn add_reward() -> Weight {
		(72_254_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: System Account (r:1 w:1)
	fn withdraw_missing_reward() -> Weight {
		(58_509_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Loans RewardSupplySpeed (r:1 w:1)
	// Storage: Loans RewardBorrowSpeed (r:1 w:1)
	// Storage: Loans RewardSupplyState (r:1 w:1)
	// Storage: Loans RewardBorrowState (r:1 w:1)
	fn update_market_reward_speed() -> Weight {
		(63_937_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Loans RewardSupplyState (r:1 w:1)
	// Storage: Loans RewardSupplySpeed (r:1 w:0)
	// Storage: Loans TotalSupply (r:1 w:0)
	// Storage: Loans RewardSupplierIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	// Storage: Loans AccountDeposits (r:1 w:0)
	// Storage: Loans RewardBorrowState (r:1 w:1)
	// Storage: Loans RewardBorrowSpeed (r:1 w:0)
	// Storage: Loans TotalBorrows (r:1 w:0)
	// Storage: Loans BorrowIndex (r:1 w:0)
	// Storage: Loans RewardBorrowerIndex (r:1 w:1)
	// Storage: Loans AccountBorrows (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	fn claim_reward() -> Weight {
		(174_138_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(15 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: Loans RewardSupplyState (r:1 w:1)
	// Storage: Loans RewardSupplySpeed (r:1 w:0)
	// Storage: Loans TotalSupply (r:1 w:0)
	// Storage: Loans RewardSupplierIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	// Storage: Loans AccountDeposits (r:1 w:0)
	// Storage: Loans RewardBorrowState (r:1 w:1)
	// Storage: Loans RewardBorrowSpeed (r:1 w:0)
	// Storage: Loans TotalBorrows (r:1 w:0)
	// Storage: Loans BorrowIndex (r:1 w:0)
	// Storage: Loans RewardBorrowerIndex (r:1 w:1)
	// Storage: Loans AccountBorrows (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	fn claim_reward_for_market() -> Weight {
		(159_672_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(13 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Assets Account (r:2 w:2)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Loans LastAccruedInterestTime (r:1 w:1)
	// Storage: Loans RewardSupplyState (r:1 w:1)
	// Storage: Loans RewardSupplySpeed (r:1 w:0)
	// Storage: Loans RewardSupplierIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	// Storage: Loans AccountDeposits (r:1 w:1)
	// Storage: Loans TotalSupply (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Loans TotalBorrows (r:1 w:0)
	// Storage: Loans TotalReserves (r:1 w:0)
	// Storage: Loans AccountEarned (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn mint() -> Weight {
		(186_943_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(17 as Weight))
			.saturating_add(T::DbWeight::get().writes(11 as Weight))
	}
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Loans LastAccruedInterestTime (r:1 w:1)
	// Storage: Loans TotalBorrows (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: Loans TotalReserves (r:1 w:0)
	// Storage: Prices EmergencyPrice (r:2 w:0)
	// Storage: Assets Metadata (r:2 w:0)
	// Storage: Loans AccountBorrows (r:2 w:1)
	// Storage: Loans AccountDeposits (r:1 w:0)
	// Storage: Loans TotalSupply (r:1 w:0)
	// Storage: Loans LiquidationFreeCollaterals (r:1 w:0)
	// Storage: Loans RewardBorrowState (r:1 w:1)
	// Storage: Loans RewardBorrowSpeed (r:1 w:0)
	// Storage: Loans RewardBorrowerIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	// Storage: Loans BorrowIndex (r:1 w:0)
	fn borrow() -> Weight {
		(260_353_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(23 as Weight))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
	}
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Loans LastAccruedInterestTime (r:1 w:1)
	// Storage: Loans TotalSupply (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: Loans TotalBorrows (r:1 w:0)
	// Storage: Loans TotalReserves (r:1 w:0)
	// Storage: Loans AccountDeposits (r:1 w:1)
	// Storage: Loans AccountEarned (r:1 w:1)
	// Storage: Loans RewardSupplyState (r:1 w:1)
	// Storage: Loans RewardSupplySpeed (r:1 w:0)
	// Storage: Loans RewardSupplierIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	fn redeem() -> Weight {
		(207_430_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(16 as Weight))
			.saturating_add(T::DbWeight::get().writes(10 as Weight))
	}
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Loans LastAccruedInterestTime (r:1 w:1)
	// Storage: Loans TotalSupply (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: Loans TotalBorrows (r:1 w:0)
	// Storage: Loans TotalReserves (r:1 w:0)
	// Storage: Loans AccountDeposits (r:1 w:1)
	// Storage: Loans AccountEarned (r:1 w:1)
	// Storage: Loans RewardSupplyState (r:1 w:1)
	// Storage: Loans RewardSupplySpeed (r:1 w:0)
	// Storage: Loans RewardSupplierIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn redeem_all() -> Weight {
		(221_141_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(17 as Weight))
			.saturating_add(T::DbWeight::get().writes(11 as Weight))
	}
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Loans LastAccruedInterestTime (r:1 w:1)
	// Storage: Loans AccountBorrows (r:1 w:1)
	// Storage: Loans BorrowIndex (r:1 w:0)
	// Storage: Loans RewardBorrowState (r:1 w:1)
	// Storage: Loans RewardBorrowSpeed (r:1 w:0)
	// Storage: Loans RewardBorrowerIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: Loans TotalBorrows (r:1 w:1)
	fn repay_borrow() -> Weight {
		(167_615_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(14 as Weight))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
	}
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Loans LastAccruedInterestTime (r:1 w:1)
	// Storage: Loans AccountBorrows (r:1 w:1)
	// Storage: Loans BorrowIndex (r:1 w:0)
	// Storage: Loans RewardBorrowState (r:1 w:1)
	// Storage: Loans RewardBorrowSpeed (r:1 w:0)
	// Storage: Loans RewardBorrowerIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: Loans TotalBorrows (r:1 w:1)
	fn repay_borrow_all() -> Weight {
		(181_531_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(14 as Weight))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
	}
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Loans AccountDeposits (r:1 w:1)
	fn collateral_asset() -> Weight {
		(64_626_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Loans LiquidationFreeCollaterals (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Loans LastAccruedInterestTime (r:2 w:2)
	// Storage: Loans Markets (r:3 w:0)
	// Storage: Loans AccountBorrows (r:3 w:1)
	// Storage: Loans BorrowIndex (r:1 w:0)
	// Storage: Prices EmergencyPrice (r:2 w:0)
	// Storage: Assets Metadata (r:2 w:0)
	// Storage: Loans AccountDeposits (r:4 w:3)
	// Storage: Loans TotalSupply (r:1 w:0)
	// Storage: Assets Asset (r:2 w:1)
	// Storage: Assets Account (r:3 w:2)
	// Storage: Loans TotalBorrows (r:2 w:1)
	// Storage: Loans TotalReserves (r:1 w:0)
	// Storage: Loans RewardBorrowState (r:1 w:1)
	// Storage: Loans RewardBorrowSpeed (r:1 w:0)
	// Storage: Loans RewardBorrowerIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:3 w:3)
	// Storage: Loans RewardSupplyState (r:1 w:1)
	// Storage: Loans RewardSupplySpeed (r:1 w:0)
	// Storage: Loans RewardSupplierIndex (r:3 w:3)
	fn liquidate_borrow() -> Weight {
		(509_387_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(39 as Weight))
			.saturating_add(T::DbWeight::get().writes(19 as Weight))
	}
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Loans TotalReserves (r:1 w:1)
	fn add_reserves() -> Weight {
		(105_996_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Loans TotalReserves (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	fn reduce_reserves() -> Weight {
		(94_887_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: Loans LiquidationFreeCollaterals (r:1 w:1)
	fn update_liquidation_free_collateral() -> Weight {
		(27_228_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
