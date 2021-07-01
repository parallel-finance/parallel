//! Liquid Staking pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as LiquidStaking;

use frame_benchmarking::account;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::sp_runtime::FixedPointNumber;
use frame_support::{assert_ok, dispatch::UnfilteredDispatchable, traits::Get};
use frame_system::{self, RawOrigin as SystemOrigin};
use orml_traits::MultiCurrency;
use primitives::Rate;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

const INITIAL_AMOUNT: u128 = 1000_000_000_000_000;
const SEED: u32 = 0;

fn initial_set_up<T: Config>(caller: T::AccountId) {
    let currency_id = T::StakingCurrency::get();
    let account_id = LiquidStaking::<T>::account_id();
    T::Currency::deposit(currency_id, &caller, INITIAL_AMOUNT).unwrap();
    T::Currency::deposit(currency_id, &account_id, INITIAL_AMOUNT).unwrap();
    ExchangeRate::<T>::put(Rate::saturating_from_rational(2, 100));
}

fn set_up_accounts<T: Config>(name: &'static str, i: u32) -> (T::AccountId, T::AccountId) {
    let caller: T::AccountId = account(name, i, SEED);
    initial_set_up::<T>(caller.clone());
    let agent: T::AccountId = account(name, i, SEED);
    (caller, agent)
}

fn create_pending_unstakes<T: Config>(
    name: &'static str,
    n: u32,
    amount: u128,
) -> Result<(), &'static str>
where
    [u8; 32]: From<<T as frame_system::Config>::AccountId>,
{
    for i in 0..n {
        process_unstake::<T>(name, i, amount);
    }
    Ok(())
}

fn process_unstake<T: Config>(name: &'static str, i: u32, amount: u128)
where
    [u8; 32]: From<<T as frame_system::Config>::AccountId>,
{
    let (caller, agent): (T::AccountId, T::AccountId) = set_up_accounts::<T>(name, i);
    let unstake_amount = 5_000_000;
    assert_ok!(LiquidStaking::<T>::stake(
        SystemOrigin::Signed(caller.clone()).into(),
        amount
    ));

    assert_ok!(LiquidStaking::<T>::unstake(
        SystemOrigin::Signed(caller.clone()).into(),
        unstake_amount
    ));

    assert_ok!(LiquidStaking::<T>::process_pending_unstake(
        T::WithdrawOrigin::successful_origin(),
        agent,
        caller,
        amount
    ));
}

fn finish_pending_unstakes<T: Config>(
    name: &'static str,
    n: u32,
    amount: u128,
) -> Result<(), &'static str>
where
    [u8; 32]: From<<T as frame_system::Config>::AccountId>,
{
    for i in 0..n {
        process_unstake::<T>(name, i, amount);
        let (caller, agent): (T::AccountId, T::AccountId) = set_up_accounts::<T>(name, i);
        assert_ok!(LiquidStaking::<T>::finish_processed_unstake(
            T::WithdrawOrigin::successful_origin(),
            agent,
            caller,
            amount
        ));
    }
    Ok(())
}

benchmarks! {
    where_clause {
        where
    [u8; 32]: From<<T as frame_system::Config>::AccountId>,
        }
    stake {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount = 100_000;
    }: _(SystemOrigin::Signed(caller.clone()), amount)
    verify {
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
        assert_last_event::<T>(Event::<T>::SlashRecorded(agent, slash_amount).into());
    }

    unstake {
        let caller: T::AccountId = account("Sample", 1, SEED);
        initial_set_up::<T>(caller.clone());
        let amount = 100_000;
        let unstake_amount = 5_000_000;
        let exchange_rate = ExchangeRate::<T>::get();
        let asset_amount = exchange_rate.checked_mul_int(unstake_amount).unwrap();
        assert_ok!(LiquidStaking::<T>::stake(
            SystemOrigin::Signed(caller.clone()).into(),
            amount));
    }: _(SystemOrigin::Signed(caller.clone()), unstake_amount)
    verify {
        assert_last_event::<T>(Event::<T>::Unstaked(caller, unstake_amount, asset_amount).into());
    }

    process_pending_unstake {
        let p in 0 .. T::MaxAccountProcessingUnstake::get() - 1;
        let amount = 100_000;
        create_pending_unstakes::<T>("unstake_agent", p, amount)?;
        let (caller, agent) : (T::AccountId, T::AccountId)= set_up_accounts::<T>("unstake_agent", SEED);
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
        assert_last_event::<T>(Event::<T>::UnstakeProcessing(agent, caller, amount).into());
    }


    finish_processed_unstake {
        let amount = 100_000;
        let p in 0 .. T::MaxAccountProcessingUnstake::get() - 1;
        finish_pending_unstakes::<T>("finish_processed_unstake", p, amount)?;
        let (caller, agent) : (T::AccountId, T::AccountId)= set_up_accounts::<T>("finish_processed_unstake", SEED);
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
            50_000
        ));
        assert_ok!(LiquidStaking::<T>::process_pending_unstake(
            T::WithdrawOrigin::successful_origin(),
            agent.clone(),
            caller.clone(),
            40_000
        ));
        assert_ok!(LiquidStaking::<T>::process_pending_unstake(
            T::WithdrawOrigin::successful_origin(),
            agent.clone(),
            caller.clone(),
            10_000
        ));

        assert_ok!(LiquidStaking::<T>::finish_processed_unstake(
            T::WithdrawOrigin::successful_origin(),
            agent.clone(),
            caller.clone(),
            50_000
        ));

        assert_ok!(LiquidStaking::<T>::finish_processed_unstake(
            T::WithdrawOrigin::successful_origin(),
            agent.clone(),
            caller.clone(),
            40_000
        ));

        let call = Call::<T>::finish_processed_unstake(agent.clone(), caller.clone(), 10_000);
        let origin = T::WithdrawOrigin::successful_origin();
    }: { call.dispatch_bypass_filter(origin)? }
    verify {
        assert_last_event::<T>(Event::<T>::UnstakeProcessed(agent, caller, 10_000).into());
    }

}

impl_benchmark_test_suite!(
    LiquidStaking,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
