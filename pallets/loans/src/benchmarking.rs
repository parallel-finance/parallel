//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_system::RawOrigin as SystemOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, impl_benchmark_test_suite};

use crate::Pallet as Loan;

benchmarks! {
	mint {
		let caller: T::AccountId = whitelisted_caller();
		let amount = 10000;
	}: _(SystemOrigin::Signed(caller.clone()), CurrencyId::DOT, amount)
}

impl_benchmark_test_suite!(
	Loan,
	crate::mock::ExtBuilder::default().build(),
	crate::mock::Runtime,
);
