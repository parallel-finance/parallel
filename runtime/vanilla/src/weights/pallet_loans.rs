
//! Autogenerated weights for `pallet_loans`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-30, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("vanilla-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/parallel
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
		(58_085_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:1 w:1)
	fn activate_market() -> Weight {
		(41_608_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:1 w:1)
	fn update_rate_model() -> Weight {
		(43_131_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:1 w:1)
	fn update_market() -> Weight {
		(44_331_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans UnderlyingAssetId (r:1 w:1)
	// Storage: Loans Markets (r:1 w:1)
	fn force_update_market() -> Weight {
		(53_841_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn add_reward() -> Weight {
		(96_816_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn withdraw_missing_reward() -> Weight {
		(79_923_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Loans RewardSupplySpeed (r:1 w:1)
	// Storage: Loans RewardBorrowSpeed (r:1 w:1)
	// Storage: Loans RewardSupplyState (r:1 w:1)
	// Storage: Loans RewardBorrowState (r:1 w:1)
	fn update_market_reward_speed() -> Weight {
		(84_212_000 as u64)
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
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
		(234_056_000 as u64)
			.saturating_add(T::DbWeight::get().reads(16 as u64))
			.saturating_add(T::DbWeight::get().writes(7 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
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
		(210_486_000 as u64)
			.saturating_add(T::DbWeight::get().reads(14 as u64))
			.saturating_add(T::DbWeight::get().writes(7 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
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
		(245_750_000 as u64)
			.saturating_add(T::DbWeight::get().reads(18 as u64))
			.saturating_add(T::DbWeight::get().writes(12 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
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
		(349_397_000 as u64)
			.saturating_add(T::DbWeight::get().reads(24 as u64))
			.saturating_add(T::DbWeight::get().writes(10 as u64))
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
	// Storage: Loans RewardSupplySpeed (r:1 w:0)
	// Storage: Loans RewardSupplierIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	fn redeem() -> Weight {
		(254_901_000 as u64)
			.saturating_add(T::DbWeight::get().reads(17 as u64))
			.saturating_add(T::DbWeight::get().writes(11 as u64))
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
	// Storage: Loans RewardSupplySpeed (r:1 w:0)
	// Storage: Loans RewardSupplierIndex (r:1 w:1)
	// Storage: Loans RewardAccured (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn redeem_all() -> Weight {
		(274_975_000 as u64)
			.saturating_add(T::DbWeight::get().reads(18 as u64))
			.saturating_add(T::DbWeight::get().writes(12 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
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
		(223_263_000 as u64)
			.saturating_add(T::DbWeight::get().reads(15 as u64))
			.saturating_add(T::DbWeight::get().writes(10 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
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
		(223_289_000 as u64)
			.saturating_add(T::DbWeight::get().reads(15 as u64))
			.saturating_add(T::DbWeight::get().writes(10 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Loans AccountDeposits (r:1 w:1)
	fn collateral_asset() -> Weight {
		(71_290_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
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
		(628_566_000 as u64)
			.saturating_add(T::DbWeight::get().reads(40 as u64))
			.saturating_add(T::DbWeight::get().writes(20 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Loans TotalReserves (r:1 w:1)
	fn add_reserves() -> Weight {
		(141_330_000 as u64)
			.saturating_add(T::DbWeight::get().reads(8 as u64))
			.saturating_add(T::DbWeight::get().writes(6 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans Markets (r:2 w:0)
	// Storage: Loans TotalReserves (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	fn reduce_reserves() -> Weight {
		(127_121_000 as u64)
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Loans LiquidationFreeCollaterals (r:1 w:1)
	fn update_liquidation_free_collateral() -> Weight {
		(34_686_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
}
