//! Liquid Staking pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as LiquidStaking;

use frame_benchmarking::account;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::sp_runtime::FixedPointNumber;
use frame_support::{
    assert_ok,
    traits::{EnsureOrigin, Get, UnfilteredDispatchable},
};
use frame_system::{self, RawOrigin as SystemOrigin};
use orml_traits::MultiCurrency;
use primitives::Rate;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

const INITIAL_AMOUNT: u128 = 100_000_000_000;
const SEED: u32 = 0;

fn initial_set_up<T: Config>(caller: T::AccountId) {
    let currency_id = T::StakingCurrency::get();
    let account_id = LiquidStaking::<T>::account_id();
    T::Currency::deposit(currency_id, &caller, INITIAL_AMOUNT).unwrap();
    T::Currency::deposit(currency_id, &account_id, INITIAL_AMOUNT).unwrap();
    ExchangeRate::<T>::put(Rate::saturating_from_rational(2, 100));
}

benchmarks! {

    stake {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount = 100_000;
    }: _(SystemOrigin::Signed(caller.clone()), amount)
    verify {
        assert_eq!(TotalStakingAsset::<T>::get(), amount);
        assert_eq!(TotalVoucher::<T>::get(), 5_000_000);

        // Check balance is correct
        assert_eq!(
            <T as Config>::Currency::free_balance(T::StakingCurrency::get(), &caller),
            INITIAL_AMOUNT - amount
        );
        assert_eq!(
            <T as Config>::Currency::free_balance(T::LiquidCurrency::get(), &caller),
            5_000_000
        );
        assert_eq!(
            <T as Config>::Currency::free_balance(T::StakingCurrency::get(), &LiquidStaking::<T>::account_id()),
            INITIAL_AMOUNT + amount
        );
        assert_last_event::<T>(Event::<T>::Staked(caller, amount).into());
    }

    withdraw {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let agent: T::AccountId = account("Sample", 6, SEED);
        let amount = 100_000;
        let withdraw_amount = T::MaxWithdrawAmount::get() - 10;
        assert_ok!(LiquidStaking::<T>::stake(
            SystemOrigin::Signed(caller.clone()).into(),
            amount));
        let call = Call::<T>::withdraw(agent.clone(), withdraw_amount);
        let origin = T::WithdrawOrigin::successful_origin();
    }: { call.dispatch_bypass_filter(origin)? }
    verify {
        // Check balance is correct
        assert_eq!(
            T::Currency::free_balance(T::StakingCurrency::get(), &caller),
            INITIAL_AMOUNT - amount
        );

        assert_eq!(
            T::Currency::free_balance(T::StakingCurrency::get(), &agent),
            withdraw_amount
        );

        assert_last_event::<T>(Event::<T>::WithdrawSuccess(agent, withdraw_amount).into());
    }

    record_rewards {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let agent: T::AccountId = account("Sample", 6, SEED);
        let amount = 100_000;
        assert_ok!(LiquidStaking::<T>::stake(
            SystemOrigin::Signed(caller.clone()).into(),
            amount));
        let call = Call::<T>::record_rewards(agent.clone(), amount);
        let origin = T::WithdrawOrigin::successful_origin();
    }: { call.dispatch_bypass_filter(origin)? }
    verify {
        assert_eq!(TotalStakingAsset::<T>::get(), 200_000);
        assert_eq!(TotalVoucher::<T>::get(), 5_000_000);

        assert_last_event::<T>(Event::<T>::RewardsRecorded(agent, amount).into());
    }

    record_slash {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let agent: T::AccountId = account("Sample", 6, SEED);
        let amount = 100_000;
        let slash_amount = 50_000;
        assert_ok!(LiquidStaking::<T>::stake(
            SystemOrigin::Signed(caller.clone()).into(),
            amount));
        let call = Call::<T>::record_slash(agent.clone(), slash_amount);
        let origin = T::WithdrawOrigin::successful_origin();
    }: { call.dispatch_bypass_filter(origin)? }
    verify {
        assert_eq!(TotalStakingAsset::<T>::get(), slash_amount);
        assert_eq!(TotalVoucher::<T>::get(), 5_000_000);

        assert_last_event::<T>(Event::<T>::SlashRecorded(agent, slash_amount).into());
    }


    unstake {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount = 100_000;
        let unstake_amount = 5_000_000;
        let exchange_rate = ExchangeRate::<T>::get();
        let asset_amount = exchange_rate.checked_mul_int(amount).unwrap();
        assert_ok!(LiquidStaking::<T>::stake(
            SystemOrigin::Signed(caller.clone()).into(),
            amount));
    }: _(SystemOrigin::Signed(caller.clone()), unstake_amount)
    verify {
        assert_eq!(TotalStakingAsset::<T>::get(), 0);
        assert_eq!(TotalVoucher::<T>::get(), 0);
        assert_eq!(
            <T as Config>::Currency::free_balance(T::StakingCurrency::get(), &caller),
            INITIAL_AMOUNT - amount
        );
        assert_eq!(
            <T as Config>::Currency::free_balance(T::StakingCurrency::get(), &LiquidStaking::<T>::account_id()),
            INITIAL_AMOUNT + amount
        );

        assert_last_event::<T>(Event::<T>::Unstaked(caller, unstake_amount, asset_amount).into());
    }

    process_pending_unstake {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let agent: T::AccountId = account("Sample", 6, SEED);
        let amount = 100_000;
        let unstake_amount = 5_000_000;
        assert_ok!(LiquidStaking::<T>::stake(
            SystemOrigin::Signed(caller.clone()).into(),
            amount));

        assert_ok!(LiquidStaking::<T>::unstake(
            SystemOrigin::Signed(caller.clone()).into(),
            unstake_amount));
        let call = Call::<T>::process_pending_unstake(agent.clone(), caller.clone(), amount);
        let origin = T::WithdrawOrigin::successful_origin();
    }: { call.dispatch_bypass_filter(origin)? }
    verify {
        assert_eq!(AccountPendingUnstake::<T>::get(&caller), None,);
        let processing_unstake = AccountProcessingUnstake::<T>::get(&agent, &caller).unwrap();
        assert_eq!(processing_unstake.len(), 1);
        assert_eq!(processing_unstake[0].amount, amount);
        assert_eq!(
            processing_unstake[0].block_number,
            frame_system::Pallet::<T>::block_number()
        );

        assert_last_event::<T>(Event::<T>::UnstakeProcessing(agent, caller, amount).into());
    }


    finish_processed_unstake {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let agent: T::AccountId = account("Sample", 6, SEED);
        let amount = 100_000;
        let unstake_amount = 5_000_000;
        assert_ok!(LiquidStaking::<T>::stake(
            SystemOrigin::Signed(caller.clone()).into(),
            amount));

        assert_ok!(LiquidStaking::<T>::unstake(
            SystemOrigin::Signed(caller.clone()).into(),
            unstake_amount));

        assert_ok!(LiquidStaking::<T>::process_pending_unstake(
            T::WithdrawOrigin::successful_origin(),
            agent.clone(),
            caller.clone(),
            amount
        ));
        let call = Call::<T>::finish_processed_unstake(agent.clone(), caller.clone(), amount);
        let origin = T::WithdrawOrigin::successful_origin();
    }: { call.dispatch_bypass_filter(origin)? }
    verify {
        assert_eq!(
            <T as Config>::Currency::free_balance(T::StakingCurrency::get(), &caller),
            INITIAL_AMOUNT
        );
        assert_eq!(
            <T as Config>::Currency::free_balance(T::StakingCurrency::get(), &LiquidStaking::<T>::account_id()),
            INITIAL_AMOUNT
        );

        assert_last_event::<T>(Event::<T>::UnstakeProcessed(agent, caller, amount).into());
    }

}

impl_benchmark_test_suite!(
    LiquidStaking,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
