//! Farming pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as Farming;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{assert_ok, traits::EnsureOrigin};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::{
    tokens::{self, *},
    CurrencyId,
};
use sp_runtime::traits::{One, StaticLookup};
use sp_std::prelude::*;

const ASSET: CurrencyId = XDOT;
const REWARD_ASSET: CurrencyId = DOT;
const ISSUE_AMOUNT: u128 = 4_000_000_000_000_000;
const STAKING_AMOUNT: u128 = 2_000_000_000_000_000;
const REWARD_AMOUNT: u128 = 2_000_000_000_000_000;
const WITHDRAW_AMOUNT: u128 = 1_000_000_000_000_000;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn initial_set_up<T: Config + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>>(
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

    T::Assets::mint_into(ASSET, &caller, ISSUE_AMOUNT).ok();
    T::Assets::mint_into(REWARD_ASSET, &caller, ISSUE_AMOUNT).ok();
}

benchmarks! {
    where_clause {
        where T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>
    }

    create {
    }: _(SystemOrigin::Root, ASSET, REWARD_ASSET, T::BlockNumber::from(10u32))
    verify {
        assert_last_event::<T>(Event::PoolAdded(ASSET, REWARD_ASSET).into());
    }

    set_pool_status {
        assert_ok!(Farming::<T>::create(
            T::UpdaterOrigin::successful_origin(),
            ASSET,
            REWARD_ASSET,
            T::BlockNumber::from(10u32)),
        );
    }: _(SystemOrigin::Root, ASSET, REWARD_ASSET, false)

    set_pool_lock_duration {
        assert_ok!(Farming::<T>::create(
            T::UpdaterOrigin::successful_origin(),
            ASSET,
            REWARD_ASSET,
            T::BlockNumber::from(10u32)),
        );
    }: _(SystemOrigin::Root, ASSET, REWARD_ASSET, T::BlockNumber::from(20u32))

    deposit {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        assert_ok!(Farming::<T>::create(
            T::UpdaterOrigin::successful_origin(),
            ASSET,
            REWARD_ASSET,
            T::BlockNumber::from(10u32)),
        );
    }: _(SystemOrigin::Signed(caller.clone()), ASSET, REWARD_ASSET, STAKING_AMOUNT)
    verify {
        assert_last_event::<T>(Event::AssetsDeposited(caller, ASSET, REWARD_ASSET, STAKING_AMOUNT).into());
    }

    withdraw {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        assert_ok!(Farming::<T>::create(
            T::UpdaterOrigin::successful_origin(),
            ASSET,
            REWARD_ASSET,
            T::BlockNumber::from(10u32)),
        );

        assert_ok!(Farming::<T>::deposit(SystemOrigin::Signed(caller.clone()).into(), ASSET, REWARD_ASSET, STAKING_AMOUNT));
    }: _(SystemOrigin::Signed(caller.clone()), ASSET, REWARD_ASSET, WITHDRAW_AMOUNT)
    verify {
        assert_last_event::<T>(Event::AssetsWithdrew(caller, ASSET, REWARD_ASSET, WITHDRAW_AMOUNT).into());
    }

    withdraw_from_lock_pool {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        assert_ok!(Farming::<T>::create(
            T::UpdaterOrigin::successful_origin(),
            ASSET,
            REWARD_ASSET,
            T::BlockNumber::from(10u32)),
        );

        assert_ok!(Farming::<T>::deposit(SystemOrigin::Signed(caller.clone()).into(), ASSET, REWARD_ASSET, STAKING_AMOUNT));
        assert_ok!(Farming::<T>::withdraw(SystemOrigin::Signed(caller.clone()).into(), ASSET, REWARD_ASSET, WITHDRAW_AMOUNT));
        assert_ok!(Farming::<T>::set_pool_lock_duration(T::UpdaterOrigin::successful_origin(), ASSET, REWARD_ASSET, T::BlockNumber::from(0u32)));
    }: _(SystemOrigin::Signed(caller.clone()), ASSET, REWARD_ASSET)
    verify {
        assert_last_event::<T>(Event::AssetsWithdrewFromLockPool(caller, ASSET, REWARD_ASSET, WITHDRAW_AMOUNT).into());
    }

    get_reward {
        let caller: T::AccountId = whitelisted_caller();
        let payer = T::Lookup::unlookup(caller.clone());
        initial_set_up::<T>(caller.clone());
        assert_ok!(Farming::<T>::create(
            T::UpdaterOrigin::successful_origin(),
            ASSET,
            REWARD_ASSET,
            T::BlockNumber::from(10u32)),
        );
        assert_ok!(Farming::<T>::dispatch_reward(
            T::UpdaterOrigin::successful_origin(),
            ASSET,
            REWARD_ASSET,
            payer,
            REWARD_AMOUNT,
            T::BlockNumber::from(10u32))
        );

        assert_ok!(Farming::<T>::deposit(SystemOrigin::Signed(caller.clone()).into(), ASSET, REWARD_ASSET, STAKING_AMOUNT));
        let target_height = frame_system::Pallet::<T>::block_number().saturating_add(One::one());
        frame_system::Pallet::<T>::set_block_number(target_height);
    }: _(SystemOrigin::Signed(caller.clone()), ASSET, REWARD_ASSET)
    verify {
        assert_last_event::<T>(Event::RewardPaid(caller, ASSET, REWARD_ASSET, 0).into());
    }

    dispatch_reward {
        let caller: T::AccountId = whitelisted_caller();
        let payer = T::Lookup::unlookup(caller.clone());
        initial_set_up::<T>(caller.clone());
        assert_ok!(Farming::<T>::create(
            T::UpdaterOrigin::successful_origin(),
            ASSET,
            REWARD_ASSET,
            T::BlockNumber::from(10u32)),
        );
    }: _(SystemOrigin::Root, ASSET, REWARD_ASSET, payer, REWARD_AMOUNT, T::BlockNumber::from(10u32))
    verify {
        assert_last_event::<T>(Event::RewardAdded(ASSET, REWARD_ASSET, REWARD_AMOUNT).into());
    }
}

impl_benchmark_test_suite!(Farming, crate::mock::new_test_ext(), crate::mock::Test,);
