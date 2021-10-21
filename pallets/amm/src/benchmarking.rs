//! AMM pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as AMM;

use frame_benchmarking::{
    benchmarks_instance_pallet, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_support::assert_ok;
use frame_support::dispatch::UnfilteredDispatchable;
use frame_support::traits::EnsureOrigin;
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::tokens::*;
use primitives::{tokens, CurrencyId};
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

const BASE_ASSET: CurrencyId = XDOT;
const QUOTE_ASSET: CurrencyId = DOT;
const INITIAL_AMOUNT: u128 = 1_000_000_000_000_000;
const ASSET_ID: u32 = 10;

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn initial_set_up<T: Config<I>, I: 'static>(caller: T::AccountId)
where
    <T::Assets as Inspect<T::AccountId>>::Balance: From<u128>,
{
    let account_id = T::Lookup::unlookup(caller.clone());

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        tokens::XDOT,
        account_id.clone(),
        true,
        One::one(),
    )
    .ok();

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        tokens::DOT,
        account_id.clone(),
        true,
        One::one(),
    )
    .ok();

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        ASSET_ID,
        account_id,
        true,
        One::one(),
    )
    .ok();

    T::Assets::mint_into(BASE_ASSET, &caller, INITIAL_AMOUNT).ok();
    T::Assets::mint_into(QUOTE_ASSET, &caller, INITIAL_AMOUNT).ok();
}

benchmarks_instance_pallet! {
    where_clause {
        where
            <T::Assets as Inspect<T::AccountId>>::Balance: From<u128>,
            <T::Assets as Inspect<T::AccountId>>::AssetId: From<u32>,

    }

    add_liquidity {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let base_amount = 100_000u128;
        let quote_amount = 200_000u128;
        assert_ok!(AMM::<T, I>::create_pool(T::CreatePoolOrigin::successful_origin(),
            (BASE_ASSET, QUOTE_ASSET), (base_amount, quote_amount),
            caller.clone(), ASSET_ID));
    }: _(SystemOrigin::Signed(caller.clone()), (BASE_ASSET, QUOTE_ASSET),
        (base_amount, quote_amount), (5u128, 5u128))
    verify {
        assert_last_event::<T, I>(Event::LiquidityAdded(caller, BASE_ASSET, QUOTE_ASSET).into());
    }

    remove_liquidity {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let base_amount = 100_000u128;
        let quote_amount = 900_000u128;
        assert_ok!(AMM::<T, I>::create_pool(T::CreatePoolOrigin::successful_origin(),
            (BASE_ASSET, QUOTE_ASSET), (base_amount, quote_amount),
            caller.clone(), ASSET_ID));
    }: _(SystemOrigin::Signed(caller.clone()), (BASE_ASSET, QUOTE_ASSET), 300_000u128)
    verify {
        assert_last_event::<T, I>(Event::LiquidityRemoved(caller, BASE_ASSET, QUOTE_ASSET).into());
    }

  create_pool {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let base_amount = 100_000u128;
        let quote_amount = 200_000u128;
        let origin = T::CreatePoolOrigin::successful_origin();
        let call = Call::<T, I>::create_pool {
            pool: (BASE_ASSET, QUOTE_ASSET),
            liquidity_amounts: (base_amount, quote_amount),
            lptoken_receiver: caller.clone(),
            asset_id: ASSET_ID
        };
    }: { call.dispatch_bypass_filter(origin)? }
    verify {
        assert_last_event::<T, I>(Event::LiquidityAdded(caller, BASE_ASSET, QUOTE_ASSET).into());
    }
}

impl_benchmark_test_suite!(AMM, crate::mock::new_test_ext(), crate::mock::Test,);
