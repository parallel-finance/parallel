//! Liquid Staking pallet benchmarking.

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin as SystemOrigin;
use orml_traits::MultiCurrency;
use pallet_liquid_staking::{Config as LiquidStakingConfig, Pallet as LiquidStaking};
use primitives::CurrencyId;
use sp_std::prelude::*;

pub struct Pallet<T: Config>(LiquidStaking<T>);
pub trait Config: LiquidStakingConfig {}

fn assert_last_event<T: Config>(generic_event: <T as LiquidStakingConfig>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

const DOT: CurrencyId = CurrencyId::DOT;
const INITIAL_AMOUNT: u128 = 100_000_000_000;

fn initial_set_up<T: Config>(caller: T::AccountId) {
    let account_id = LiquidStaking::<T>::account_id();
    <T as LiquidStakingConfig>::Currency::deposit(DOT, &caller, INITIAL_AMOUNT).unwrap();
    <T as LiquidStakingConfig>::Currency::deposit(DOT, &account_id, INITIAL_AMOUNT).unwrap();
}

benchmarks! {

    stake {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount = 100_000;
    }: {
        let _ = LiquidStaking::<T>::stake(
            SystemOrigin::Signed(caller.clone()).into(),
            amount
        );
    }
    verify {
        assert_eq!(pallet_liquid_staking::TotalStakingAsset::<T>::get(), amount);
        assert_eq!(pallet_liquid_staking::TotalVoucher::<T>::get(), 5_000_000);

        // Check balance is correct
        assert_eq!(
            <T as LiquidStakingConfig>::Currency::free_balance(CurrencyId::DOT, &caller),
            INITIAL_AMOUNT - amount
        );
        assert_eq!(
            <T as LiquidStakingConfig>::Currency::free_balance(CurrencyId::xDOT, &caller),
            5_000_000
        );
        assert_eq!(
            <T as LiquidStakingConfig>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::<T>::account_id()),
            INITIAL_AMOUNT + amount
        );
        assert_last_event::<T>(pallet_liquid_staking::Event::Staked(caller, amount).into());
    }

}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
