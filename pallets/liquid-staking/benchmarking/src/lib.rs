//! Liquid Staking pallet benchmarking.

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;

use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, impl_benchmark_test_suite};


benchmarks! {
	stake {
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller), 10)
}

impl_benchmark_test_suite!(
	Pallet,
	crate::mock::new_test_ext(),
	crate::mock::Test,
);
