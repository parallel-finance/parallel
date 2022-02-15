//! Farming pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as LM;

use frame_benchmarking::{
    benchmarks_instance_pallet, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_support::{assert_ok, dispatch::UnfilteredDispatchable, traits::EnsureOrigin};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::{
    tokens::{self, *},
    CurrencyId,
};
use sp_runtime::traits::{One, StaticLookup};
use sp_std::{convert::TryInto, prelude::*};

const ASSET: CurrencyId = XDOT;
const SHARES: CurrencyId = DOT;
const INITIAL_AMOUNT: u128 = 1_000_000_000_000_000;
const ASSET_ID: u32 = 10;

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

    T::Assets::mint_into(ASSET, &caller, INITIAL_AMOUNT).ok();
    T::Assets::mint_into(SHARES, &caller, INITIAL_AMOUNT).ok();
}

benchmarks_instance_pallet! {
    where_clause {
        where
            T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>
    }

    create {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let stash = T::Lookup::unlookup(caller);
        let origin = T::CreateOrigin::successful_origin();
        let call = Call::<T, I>::create {
            asset: ASSET,
            stash,
            start: T::BlockNumber::from(3u32),
            end: T::BlockNumber::from(5u32),
            rewards: vec![(1, ASSET); 1000].try_into().unwrap(),
            asset_id: ASSET_ID
        };
    }: { call.dispatch_bypass_filter(origin)? }
    verify {
        assert_last_event::<T, I>(Event::PoolAdded(ASSET).into());
    }

    deposit {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let amount = 100_000u128;
        assert_ok!(LM::<T, I>::create(T::CreateOrigin::successful_origin(),
            ASSET, T::Lookup::unlookup(caller.clone()),
            T::BlockNumber::zero(),T::BlockNumber::from(15u32), vec![(1, ASSET); 1000].try_into().unwrap(),ASSET_ID));
    }: _(SystemOrigin::Signed(caller.clone()), ASSET, amount)
    verify {
        assert_last_event::<T, I>(Event::AssetsDeposited(caller, ASSET).into());
    }

    withdraw {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let amount = 100_000u128;
        assert_ok!(LM::<T, I>::create(T::CreateOrigin::successful_origin(),
            ASSET, T::Lookup::unlookup(caller.clone()),
            T::BlockNumber::zero(),T::BlockNumber::from(15u32), vec![(1, ASSET); 1000].try_into().unwrap(),ASSET_ID));

        assert_ok!(LM::<T, I>::deposit(SystemOrigin::Signed(caller.clone()).into(), ASSET, amount));
    }: _(SystemOrigin::Signed(caller.clone()), ASSET, amount)
    verify {
        assert_last_event::<T, I>(Event::AssetsWithdrew(caller, ASSET).into());
    }

}

impl_benchmark_test_suite!(LM, crate::mock::new_test_ext(), crate::mock::Test,);
