//! Router pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

extern crate alloc;

use super::*;
use crate::pallet::BalanceOf;

#[allow(unused_imports)]
use crate::Pallet as AMMRoute;
use frame_benchmarking::{
    account, benchmarks_instance_pallet, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_support::{
    assert_ok,
    traits::{
        fungibles::{Inspect, Mutate},
        EnsureOrigin,
    },
};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::{tokens, Balance, CurrencyId};
use sp_runtime::traits::{One, StaticLookup};
use sp_std::{vec, vec::Vec};

const DOT: CurrencyId = tokens::DOT;
const SDOT: CurrencyId = tokens::SDOT;
const INITIAL_AMOUNT: u128 = 1_000_000_000_000_000;
const ASSET_ID: u32 = 11;

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn initial_set_up<
    T: Config<I> + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance> + pallet_amm::Config,
    I: 'static,
>(
    caller: T::AccountId,
) {
    let account_id = T::Lookup::unlookup(caller.clone());

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        tokens::SDOT,
        account_id.clone(),
        true,
        One::one(),
    )
    .ok();

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        tokens::DOT,
        account_id,
        true,
        One::one(),
    )
    .ok();

    <T as crate::Config<I>>::Assets::mint_into(DOT, &caller, INITIAL_AMOUNT).ok();

    let pool_creator = account("pool_creator", 1, 0);
    <T as crate::Config<I>>::Assets::mint_into(DOT, &pool_creator, INITIAL_AMOUNT).ok();
    <T as crate::Config<I>>::Assets::mint_into(SDOT, &pool_creator, INITIAL_AMOUNT).ok();

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        ASSET_ID,
        T::Lookup::unlookup(pool_creator.clone()),
        true,
        One::one(),
    )
    .ok();

    assert_ok!(pallet_amm::Pallet::<T>::create_pool(
        T::CreatePoolOrigin::successful_origin(),
        (DOT, SDOT),
        (100_000_000u128, 100_000_000u128),
        pool_creator.clone(),
        ASSET_ID
    ));

    assert_ok!(pallet_amm::Pallet::<T>::add_liquidity(
        SystemOrigin::Signed(pool_creator).into(),
        (DOT, SDOT),
        (100_000_000u128, 100_000_000u128),
        (99_999u128, 99_999u128)
    ));
}

benchmarks_instance_pallet! {
    where_clause {
        where
            T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance> + pallet_amm::Config
    }
    swap_exact_tokens_for_tokens {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let amount_in = 1_000u128;
        let min_amount_out = 980u128;
        let expiry = u32::MAX;
        let routes: Vec<_> = vec![DOT, SDOT];
    }: swap_exact_tokens_for_tokens(SystemOrigin::Signed(caller.clone()), routes, amount_in, min_amount_out)

    verify {
        let routes: Vec<_> = vec![DOT, SDOT];
        let amount_out: BalanceOf<T, I> = <T as crate::Config<I>>::Assets::balance(SDOT, &caller);
        let expected = 994u128;

        assert_eq!(amount_out, expected);
        assert_last_event::<T, I>(Event::Traded(caller, amount_in, routes, expected).into());
    }

    swap_tokens_for_exact_tokens {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let balance_before_trade: BalanceOf<T, I> = <T as crate::Config<I>>::Assets::balance(DOT, &caller);
        let amount_out = 980u128;
        let max_amount_in = 1_000u128;
        let expiry = u32::MAX;
        let routes: Vec<_> = vec![DOT, SDOT];
    }: swap_tokens_for_exact_tokens(SystemOrigin::Signed(caller.clone()), routes, amount_out, max_amount_in)

    verify {
        let routes: Vec<_> = vec![DOT, SDOT];
        let balance_after_trade: BalanceOf<T, I> = <T as crate::Config<I>>::Assets::balance(DOT, &caller);
        let amount_in = balance_before_trade - balance_after_trade;
        let expected = 986u128;

        assert_eq!(amount_in, expected);
        assert_last_event::<T, I>(Event::Traded(caller, expected, routes, amount_out).into());
    }
}

impl_benchmark_test_suite!(AMMRoute, crate::mock::new_test_ext(), crate::mock::Runtime,);
