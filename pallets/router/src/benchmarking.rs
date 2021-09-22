//! AMM route pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

extern crate alloc;

use super::*;
use crate::Pallet as AMMRoute;
use core::convert::TryFrom;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    traits::tokens::fungibles::{Inspect, Mutate},
    BoundedVec,
};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::{currency::CurrencyId, tokens};

const DOT: CurrencyId = CurrencyId::Asset(tokens::DOT);
const XDOT: CurrencyId = CurrencyId::Asset(tokens::XDOT);
const INITIAL_AMOUNT: u128 = 10_000;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn initial_set_up<T: Config>(caller: T::AccountId) {
    T::AMMCurrency::mint_into(DOT, &caller, INITIAL_AMOUNT.into()).unwrap();
}

benchmarks! {
    trade {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount_in = 1_000;
        let original_amount_in = amount_in;
        let min_amount_out = 980;
        let expiry = u32::MAX;
        let routes: BoundedVec<_, <T as Config>::MaxLengthRoute> = Route::<T>::try_from(alloc::vec![(DOT, XDOT)]).unwrap();
    }: _(SystemOrigin::Signed(caller.clone()), routes.clone(), amount_in, min_amount_out, expiry.into())

    verify {
        let amount_out = T::AMMCurrency::balance(XDOT, &caller);
        assert_eq!(amount_out, 994);
        assert_last_event::<T>(Event::TradedSuccessfully(caller, original_amount_in, routes, amount_out).into());
    }
}

impl_benchmark_test_suite!(
    AMMRoute,
    crate::mock::benchmark_test_ext(),
    crate::mock::Runtime,
);
