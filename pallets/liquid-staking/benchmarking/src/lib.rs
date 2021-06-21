//! Liquid Staking pallet benchmarking.

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin as SystemOrigin;
use pallet_liquid_staking::{Config as LiquidStakingConfig, Pallet as LiquidStaking};
use sp_std::prelude::*;

pub struct Pallet<T: Config>(LiquidStaking<T>);
pub trait Config: LiquidStakingConfig {}

benchmarks! {

    stake {
        let caller: T::AccountId = whitelisted_caller();
        let amount = 100_000;
    }: {
        let _ = LiquidStaking::<T>::stake(
            SystemOrigin::Signed(caller.clone()).into(),
            amount
        );
    }

}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
