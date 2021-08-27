//! AMM pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as AMM;

use frame_benchmarking::{
    benchmarks_instance_pallet, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_support::assert_ok;
use frame_system::{self, RawOrigin as SystemOrigin};
use orml_traits::MultiCurrency;
use sp_std::prelude::*;

const BASE_ASSET: CurrencyId = CurrencyId::xDOT;
const QUOTE_ASSET: CurrencyId = CurrencyId::DOT;
const INITIAL_AMOUNT: u128 = 1000_000_000_000_000;

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn initial_set_up<T: Config<I>, I: 'static>(caller: T::AccountId) {
    T::Currency::deposit(BASE_ASSET, &caller, INITIAL_AMOUNT).unwrap();
    T::Currency::deposit(QUOTE_ASSET, &caller, INITIAL_AMOUNT).unwrap();
}

benchmarks_instance_pallet! {
    add_liquidity {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let base_amount = 100_000;
        let quote_amount = 200_000;
    }: _(SystemOrigin::Signed(caller.clone()), (BASE_ASSET, QUOTE_ASSET), (base_amount, quote_amount))
    verify {
        assert_last_event::<T, I>(Event::LiquidityAdded(caller, BASE_ASSET, QUOTE_ASSET).into());
    }

    remove_liquidity {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let base_amount = 100_000;
        let quote_amount = 900_000;
        assert_ok!(AMM::<T, I>::add_liquidity(SystemOrigin::Signed(caller.clone()).into(), (BASE_ASSET, QUOTE_ASSET), (base_amount, quote_amount)));
    }: _(SystemOrigin::Signed(caller.clone()), (BASE_ASSET, QUOTE_ASSET), 300_000)
    verify {
        assert_last_event::<T, I>(Event::LiquidityRemoved(caller, BASE_ASSET, QUOTE_ASSET).into());
    }
}

impl_benchmark_test_suite!(AMM, crate::mock::new_test_ext(), crate::mock::Test,);
