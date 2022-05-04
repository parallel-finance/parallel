
//! Autogenerated weights for `pallet_loans`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-02, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("vanilla-dev"), DB CACHE: 1024

// Executed Command:
// target/release/parallel
// benchmark
// pallet
// --chain=vanilla-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_loans
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/vanilla/src/weights/pallet_loans.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_loans`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_loans::WeightInfo for WeightInfo<T> {
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:2 w:1)
	// Storage: Loans UnderlyingAssetId (r:1 w:1)
	// Storage: Loans ExchangeRate (r:0 w:1)
	// Storage: Loans BorrowIndex (r:0 w:1)
	fn add_market() -> Weight {
		(85_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:1 w:1)
	fn activate_market() -> Weight {
		(55_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:1 w:1)
	fn update_rate_model() -> Weight {
		(58_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:1 w:1)
	fn update_market() -> Weight {
		(58_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans UnderlyingAssetId (r:1 w:1)
	// Storage: Loans Markets (r:1 w:1)
	fn force_update_market() -> Weight {
		(74_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn add_reward() -> Weight {
		(102_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn withdraw_missing_reward() -> Weight {
		(81_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Loans RewardSupplyState (r:1 w:1)
	// Storage: Loans MarketRewardSpeed (r:1 w:1)
	// Storage: Loans RewardBorrowState (r:1 w:1)
	fn update_market_reward_speed() -> Weight {
		(82_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Loans RewardSupplyState (r:1 w:1)
	// Storage: Loans MarketRewardSpeed (r:1 w:0)
	// Storage: Loans TotalSupply (r:1 w:0)
	// Storage: Loans RewardSupplierIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	// Storage: Loans AccountDeposits (r:1 w:0)
	// Storage: Loans RewardBorrowState (r:1 w:1)
	// Storage: Loans TotalBorrows (r:1 w:0)
	// Storage: Loans BorrowIndex (r:1 w:0)
	// Storage: Loans RewardBorrowerIndex (r:1 w:1)
	// Storage: Loans AccountBorrows (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	fn claim_reward() -> Weight {
		(209_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(15 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans RewardSupplyState (r:1 w:1)
	// Storage: Loans MarketRewardSpeed (r:1 w:0)
	// Storage: Loans TotalSupply (r:1 w:0)
	// Storage: Loans RewardSupplierIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	// Storage: Loans AccountDeposits (r:1 w:0)
	// Storage: Loans RewardBorrowState (r:1 w:1)
	// Storage: Loans TotalBorrows (r:1 w:0)
	// Storage: Loans BorrowIndex (r:1 w:0)
	// Storage: Loans RewardBorrowerIndex (r:1 w:1)
	// Storage: Loans AccountBorrows (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	fn claim_reward_for_market() -> Weight {
		(191_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(13 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Assets Account (r:2 w:2)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Loans LastAccruedInterestTime (r:1 w:1)
	// Storage: Loans RewardSupplyState (r:1 w:1)
	// Storage: Loans MarketRewardSpeed (r:1 w:0)
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
		(225_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(18 as Weight))
			.saturating_add(T::DbWeight::get().writes(12 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Loans LastAccruedInterestTime (r:1 w:1)
	// Storage: Loans TotalBorrows (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: Loans TotalReserves (r:1 w:0)
	// Storage: Prices EmergencyPrice (r:1 w:0)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Loans AccountBorrows (r:1 w:1)
	// Storage: Loans AccountDeposits (r:1 w:0)
	// Storage: Loans TotalSupply (r:1 w:0)
	// Storage: Loans RewardBorrowState (r:1 w:1)
	// Storage: Loans MarketRewardSpeed (r:1 w:0)
	// Storage: Loans RewardBorrowerIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	// Storage: Loans BorrowIndex (r:1 w:0)
	fn borrow() -> Weight {
		(249_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(20 as Weight))
			.saturating_add(T::DbWeight::get().writes(10 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
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
	// Storage: Loans MarketRewardSpeed (r:1 w:0)
	// Storage: Loans RewardSupplierIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	fn redeem() -> Weight {
		(202_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(17 as Weight))
			.saturating_add(T::DbWeight::get().writes(11 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
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
	// Storage: Loans MarketRewardSpeed (r:1 w:0)
	// Storage: Loans RewardSupplierIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn redeem_all() -> Weight {
		(219_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(18 as Weight))
			.saturating_add(T::DbWeight::get().writes(12 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Loans LastAccruedInterestTime (r:1 w:1)
	// Storage: Loans AccountBorrows (r:1 w:1)
	// Storage: Loans BorrowIndex (r:1 w:0)
	// Storage: Loans RewardBorrowState (r:1 w:1)
	// Storage: Loans MarketRewardSpeed (r:1 w:0)
	// Storage: Loans RewardBorrowerIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: Loans TotalBorrows (r:1 w:1)
	fn repay_borrow() -> Weight {
		(175_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(15 as Weight))
			.saturating_add(T::DbWeight::get().writes(10 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Loans LastAccruedInterestTime (r:1 w:1)
	// Storage: Loans AccountBorrows (r:1 w:1)
	// Storage: Loans BorrowIndex (r:1 w:0)
	// Storage: Loans RewardBorrowState (r:1 w:1)
	// Storage: Loans MarketRewardSpeed (r:1 w:0)
	// Storage: Loans RewardBorrowerIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: Loans TotalBorrows (r:1 w:1)
	fn repay_borrow_all() -> Weight {
		(152_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(15 as Weight))
			.saturating_add(T::DbWeight::get().writes(10 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Loans AccountDeposits (r:1 w:1)
	fn collateral_asset() -> Weight {
		(42_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
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
	// Storage: Loans MarketRewardSpeed (r:2 w:0)
	// Storage: Loans RewardBorrowerIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:3 w:3)
	// Storage: Loans RewardSupplyState (r:1 w:1)
	// Storage: Loans RewardSupplierIndex (r:3 w:3)
	fn liquidate_borrow() -> Weight {
		(440_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(39 as Weight))
			.saturating_add(T::DbWeight::get().writes(20 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Loans TotalReserves (r:1 w:1)
	fn add_reserves() -> Weight {
		(92_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Loans TotalReserves (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	fn reduce_reserves() -> Weight {
		(83_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
}
