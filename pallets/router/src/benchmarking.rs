//! Router pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

extern crate alloc;

use super::*;
#[allow(unused_imports)]
use crate::Pallet as AMMRoute;
use core::convert::TryFrom;
use frame_benchmarking::{
    account, benchmarks_instance_pallet, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_support::{
    assert_ok,
    traits::tokens::fungibles::{Inspect, Mutate},
    BoundedVec,
};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::{tokens, CurrencyId};
use sp_runtime::traits::StaticLookup;

const DOT: CurrencyId = tokens::DOT;
const XDOT: CurrencyId = tokens::XDOT;
const INITIAL_AMOUNT: u128 = 1000_000_000_000_000;

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn initial_set_up<T: Config<I>, I: 'static>(caller: T::AccountId) {
    let account_id = T::Lookup::unlookup(caller.clone());

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        tokens::XDOT,
        account_id.clone(),
        true,
        1,
    )
    .ok();

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        tokens::DOT,
        account_id.clone(),
        true,
        1,
    )
    .ok();

    <T as crate::Config<I>>::Assets::mint_into(DOT, &caller, INITIAL_AMOUNT.into()).ok();

    let pool_creator = account("pool_creator", 1, 0);
    <T as crate::Config<I>>::Assets::mint_into(DOT, &pool_creator, INITIAL_AMOUNT.into()).ok();
    <T as crate::Config<I>>::Assets::mint_into(XDOT, &pool_creator, INITIAL_AMOUNT.into()).ok();

    assert_ok!(pallet_amm::Pallet::<T>::add_liquidity(
        SystemOrigin::Signed(pool_creator).into(),
        (DOT, XDOT),
        (100_000_000, 100_000_000),
        (99_999, 99_999),
        10
    ));
}

benchmarks_instance_pallet! {
    trade {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let amount_in = 1_000;
        let original_amount_in = amount_in;
        let min_amount_out = 980;
        let expiry = u32::MAX;
        let routes: BoundedVec<_, <T as Config<I>>::MaxLengthRoute> = Route::<T, I>::try_from(alloc::vec![(DOT, XDOT)]).unwrap();
    }: trade(SystemOrigin::Signed(caller.clone()), routes.clone(), amount_in, min_amount_out, expiry.into())

    verify {
        let amount_out = <T as crate::Config<I>>::Assets::balance(XDOT, &caller);

        assert_eq!(amount_out, 994);
        assert_last_event::<T, I>(Event::TradedSuccessfully(caller, original_amount_in, routes, amount_out).into());
    }
}

impl_benchmark_test_suite!(AMMRoute, crate::mock::new_test_ext(), crate::mock::Runtime,);
