//! AMM pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as AMM;

use frame_benchmarking::{
    benchmarks_instance_pallet, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_support::assert_ok;
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::tokens::*;
use primitives::{tokens, CurrencyId};
use sp_std::prelude::*;

const BASE_ASSET: CurrencyId = XDOT;
const QUOTE_ASSET: CurrencyId = DOT;
const INITIAL_AMOUNT: u128 = 1000_000_000_000_000;
const ASSET_ID: u32 = 10;

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn initial_set_up<T: Config<I>, I: 'static>(caller: T::AccountId)
where
    BalanceOf<T, I>: FixedPointOperand,
    AssetIdOf<T, I>: AtLeast32BitUnsigned,
    <T::Assets as Inspect<T::AccountId>>::Balance: From<u128>,
{
    let account_id = T::Lookup::unlookup(caller.clone());

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        tokens::XDOT.into(),
        account_id.clone(),
        true,
        One::one(),
    )
    .ok();

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        tokens::DOT.into(),
        account_id.clone(),
        true,
        One::one(),
    )
    .ok();

    T::Assets::mint_into(BASE_ASSET.into(), &caller, INITIAL_AMOUNT.into()).ok();
    T::Assets::mint_into(QUOTE_ASSET.into(), &caller, INITIAL_AMOUNT.into()).ok();
}

benchmarks_instance_pallet! {
    where_clause {
        where
            BalanceOf<T, I>: FixedPointOperand,
            AssetIdOf<T, I>: AtLeast32BitUnsigned,
            <T::Assets as Inspect<T::AccountId>>::Balance: From<u128>,
            <T::Assets as Inspect<T::AccountId>>::AssetId: From<u32>,

    }
    add_liquidity_non_existing_pool {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let base_amount = 100_000;
        let quote_amount = 200_000;
    }: add_liquidity(SystemOrigin::Signed(caller.clone()), (BASE_ASSET.into(), QUOTE_ASSET.into()), (base_amount.into(), quote_amount.into()),
            (5.into(), 5.into()), ASSET_ID.into())
    verify {
        assert_last_event::<T, I>(Event::LiquidityAdded(caller, BASE_ASSET.into(), QUOTE_ASSET.into()).into());
    }

    add_liquidity_existing_pool {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let base_amount = 100_000;
        let quote_amount = 200_000;
        assert_ok!(AMM::<T, I>::add_liquidity(SystemOrigin::Signed(caller.clone()).into(),
            (BASE_ASSET.into(), QUOTE_ASSET.into()), (base_amount.into(), quote_amount.into()),
            (5.into(), 5.into()), ASSET_ID.into()));
    }: add_liquidity(SystemOrigin::Signed(caller.clone()), (BASE_ASSET.into(), QUOTE_ASSET.into()),
        (base_amount.into(), quote_amount.into()), (5.into(), 5.into()), ASSET_ID.into())
    verify {
        assert_last_event::<T, I>(Event::LiquidityAdded(caller, BASE_ASSET.into(), QUOTE_ASSET.into()).into());
    }

    remove_liquidity {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let base_amount = 100_000;
        let quote_amount = 900_000;
        assert_ok!(AMM::<T, I>::add_liquidity(SystemOrigin::Signed(caller.clone()).into(),
            (BASE_ASSET.into(), QUOTE_ASSET.into()), (base_amount.into(), quote_amount.into()),
            (5.into(), 5.into()), ASSET_ID.into()));
    }: _(SystemOrigin::Signed(caller.clone()), (BASE_ASSET.into(), QUOTE_ASSET.into()), 300_000.into())
    verify {
        assert_last_event::<T, I>(Event::LiquidityRemoved(caller, BASE_ASSET.into(), QUOTE_ASSET.into()).into());
    }

  force_create_pool {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let base_amount = 100_000;
        let quote_amount = 200_000;
    }: _(SystemOrigin::Root, (BASE_ASSET.into(), QUOTE_ASSET.into()), (base_amount.into(), quote_amount.into()),
            caller.clone(), ASSET_ID.into())
    verify {
        assert_last_event::<T, I>(Event::LiquidityAdded(caller, BASE_ASSET.into(), QUOTE_ASSET.into()).into());
    }
}

impl_benchmark_test_suite!(AMM, crate::mock::new_test_ext(), crate::mock::Test,);
