//! AMM pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as AMM;

use frame_benchmarking::{
    benchmarks_instance_pallet, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_support::{assert_ok, dispatch::UnfilteredDispatchable, traits::EnsureOrigin};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::{
    tokens::{self, *},
    CurrencyId,
};
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

const BASE_ASSET: CurrencyId = SDOT;
const QUOTE_ASSET: CurrencyId = DOT;
const INITIAL_AMOUNT: u128 = 1_000_000_000_000_000;
const ASSET_ID: u32 = 10;
const MINIMUM_LIQUIDITY: u128 = 1_000u128;

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn initial_set_up<
    T: Config<I> + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>,
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
        where T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>
    }

    add_liquidity {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let base_amount = 100_000u128;
        let quote_amount = 200_000u128;
        assert_ok!(AMM::<T, I>::create_pool(T::CreatePoolOrigin::successful_origin(),
            (BASE_ASSET, QUOTE_ASSET), (base_amount, quote_amount),
            caller.clone(), ASSET_ID));
    }: _(
        SystemOrigin::Signed(caller.clone()),
        (BASE_ASSET, QUOTE_ASSET),
        (base_amount, quote_amount),
        (5u128, 5u128)
    )
    verify {
        assert_last_event::<T, I>(Event::<T, I>::LiquidityAdded(
            caller,
            BASE_ASSET,
            QUOTE_ASSET,
            base_amount,
            quote_amount,
            ASSET_ID,
            base_amount * 2,
            quote_amount * 2,
        ).into());
    }

    remove_liquidity {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let base_amount = 100_000u128;
        let quote_amount = 900_000u128;
        assert_ok!(AMM::<T, I>::create_pool(T::CreatePoolOrigin::successful_origin(),
            (BASE_ASSET, QUOTE_ASSET), (base_amount, quote_amount),
            caller.clone(), ASSET_ID));
    }: _(
        SystemOrigin::Signed(caller.clone()),
        (BASE_ASSET, QUOTE_ASSET),
        300_000u128 - MINIMUM_LIQUIDITY
    )
    verify {
        assert_last_event::<T, I>(Event::<T, I>::LiquidityRemoved(
            caller,
            BASE_ASSET,
            QUOTE_ASSET,
            300_000u128 - MINIMUM_LIQUIDITY,
            99666,
            897000,
            ASSET_ID,
            334,
            3000,
        ).into());
    }

    create_pool {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let base_amount = 100_000u128;
        let quote_amount = 200_000u128;
        let origin = T::CreatePoolOrigin::successful_origin();
        let call = Call::<T, I>::create_pool {
            pair: (BASE_ASSET, QUOTE_ASSET),
            liquidity_amounts: (base_amount, quote_amount),
            lptoken_receiver: caller.clone(),
            lp_token_id: ASSET_ID
        };
    }: {
        call.dispatch_bypass_filter(origin)?
    }
    verify {
        assert_last_event::<T, I>(Event::<T, I>::LiquidityAdded(
            caller,
            BASE_ASSET,
            QUOTE_ASSET,
            base_amount,
            quote_amount,
            ASSET_ID,
            base_amount,
            quote_amount,
        ).into());
    }
}

impl_benchmark_test_suite!(AMM, crate::mock::new_test_ext(), crate::mock::Test,);
