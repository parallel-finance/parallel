//! Liquid Staking pallet benchmarking.

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;

use frame_benchmarking::account;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::assert_ok;
use frame_support::sp_runtime::FixedPointNumber;
use frame_system::RawOrigin as SystemOrigin;
use orml_traits::MultiCurrency;
use pallet_liquid_staking::{Config as LiquidStakingConfig, Pallet as LiquidStaking};
use primitives::{CurrencyId, Rate};
use sp_std::prelude::*;

pub struct Pallet<T: Config>(LiquidStaking<T>);
pub trait Config: LiquidStakingConfig {}

fn assert_last_event<T: Config>(generic_event: <T as LiquidStakingConfig>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

const KSM: CurrencyId = CurrencyId::KSM;
const INITIAL_AMOUNT: u128 = 100_000_000_000;
const SEED: u32 = 0;

fn initial_set_up<T: Config>(caller: T::AccountId) {
    let account_id = LiquidStaking::<T>::account_id();
    <T as LiquidStakingConfig>::Currency::deposit(KSM, &caller, INITIAL_AMOUNT).unwrap();
    <T as LiquidStakingConfig>::Currency::deposit(KSM, &account_id, INITIAL_AMOUNT).unwrap();
    pallet_liquid_staking::ExchangeRate::<T>::put(Rate::saturating_from_rational(2, 100));
}

benchmarks! {

    stake {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount = 100_000;
    }: {
        let _ = LiquidStaking::<T>::stake(
            SystemOrigin::Signed(caller.clone()).into(),
            amount
        );
    }
    verify {
        assert_eq!(pallet_liquid_staking::TotalStakingAsset::<T>::get(), amount);
        assert_eq!(pallet_liquid_staking::TotalVoucher::<T>::get(), 5_000_000);

        // Check balance is correct
        assert_eq!(
            <T as LiquidStakingConfig>::Currency::free_balance(CurrencyId::KSM, &caller),
            INITIAL_AMOUNT - amount
        );
        assert_eq!(
            <T as LiquidStakingConfig>::Currency::free_balance(CurrencyId::xKSM, &caller),
            5_000_000
        );
        assert_eq!(
            <T as LiquidStakingConfig>::Currency::free_balance(CurrencyId::KSM, &LiquidStaking::<T>::account_id()),
            INITIAL_AMOUNT + amount
        );
        assert_last_event::<T>(pallet_liquid_staking::Event::Staked(caller, amount).into());
    }

    withdraw {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let agent: T::AccountId = account("Sample", 6, SEED);
        let amount = 100_000;
        assert_ok!(LiquidStaking::<T>::stake(
            SystemOrigin::Signed(caller.clone()).into(),
            amount));
    }: {
        let _ = LiquidStaking::<T>::withdraw(
            SystemOrigin::Root.into(),
            agent.clone(),
            amount
        );
    }
    verify {
        // Check balance is correct
        assert_eq!(
            <T as LiquidStakingConfig>::Currency::free_balance(CurrencyId::KSM, &caller),
            INITIAL_AMOUNT - amount
        );

        assert_eq!(
            <T as LiquidStakingConfig>::Currency::free_balance(CurrencyId::KSM, &agent),
            amount
        );

        assert_last_event::<T>(pallet_liquid_staking::Event::WithdrawSuccess(agent, amount).into());
    }

    record_rewards {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let agent: T::AccountId = account("Sample", 6, SEED);
        let amount = 100_000;
        assert_ok!(LiquidStaking::<T>::stake(
            SystemOrigin::Signed(caller.clone()).into(),
            amount));
    }: {
        let _ = LiquidStaking::<T>::record_rewards(
            SystemOrigin::Root.into(),
            agent.clone(),
            amount
        );
    }
    verify {
        assert_eq!(pallet_liquid_staking::TotalStakingAsset::<T>::get(), 200_000);
        assert_eq!(pallet_liquid_staking::TotalVoucher::<T>::get(), 5_000_000);

        assert_last_event::<T>(pallet_liquid_staking::Event::RewardsRecorded(agent, amount).into());
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
    }: {
        let _ = LiquidStaking::<T>::record_slash(
            SystemOrigin::Root.into(),
            agent.clone(),
            slash_amount
        );
    }
    verify {
        assert_eq!(pallet_liquid_staking::TotalStakingAsset::<T>::get(), slash_amount);
        assert_eq!(pallet_liquid_staking::TotalVoucher::<T>::get(), 5_000_000);

        assert_last_event::<T>(pallet_liquid_staking::Event::SlashRecorded(agent, slash_amount).into());
    }

    unstake {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount = 100_000;
        let unstake_amount = 5_000_000;
        assert_ok!(LiquidStaking::<T>::stake(
            SystemOrigin::Signed(caller.clone()).into(),
            amount));
    }: {
        let _ = LiquidStaking::<T>::unstake(
            SystemOrigin::Signed(caller.clone()).into(),
            unstake_amount
        );
    }
    verify {
        assert_eq!(pallet_liquid_staking::TotalStakingAsset::<T>::get(), 0);
        assert_eq!(pallet_liquid_staking::TotalVoucher::<T>::get(), 0);
        assert_eq!(
            <T as LiquidStakingConfig>::Currency::free_balance(CurrencyId::KSM, &caller),
            INITIAL_AMOUNT - amount
        );
        assert_eq!(
            <T as LiquidStakingConfig>::Currency::free_balance(CurrencyId::KSM, &LiquidStaking::<T>::account_id()),
            INITIAL_AMOUNT + amount
        );

        assert_last_event::<T>(pallet_liquid_staking::Event::Unstaked(caller, unstake_amount).into());
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
    }: {
        let _ = LiquidStaking::<T>::process_pending_unstake(
            SystemOrigin::Root.into(),
            agent.clone(),
            caller.clone(),
            amount
        );
    }
    verify {
        assert_eq!(pallet_liquid_staking::AccountPendingUnstake::<T>::get(&caller), None,);
        let processing_unstake = pallet_liquid_staking::AccountProcessingUnstake::<T>::get(&agent, &caller).unwrap();
        assert_eq!(processing_unstake.len(), 1);
        assert_eq!(processing_unstake[0].amount, amount);
        assert_eq!(
            processing_unstake[0].block_number,
            frame_system::Pallet::<T>::block_number()
        );

        assert_last_event::<T>(pallet_liquid_staking::Event::UnstakeProcessing(agent, caller, amount).into());
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
            SystemOrigin::Root.into(),
            agent.clone(),
            caller.clone(),
            amount
        ));
    }: {
        let _ = LiquidStaking::<T>::finish_processed_unstake(
            SystemOrigin::Root.into(),
            agent.clone(),
            caller.clone(),
            amount
        );
    }
    verify {
        assert_eq!(
            <T as LiquidStakingConfig>::Currency::free_balance(CurrencyId::KSM, &caller),
            INITIAL_AMOUNT
        );
        assert_eq!(
            <T as LiquidStakingConfig>::Currency::free_balance(CurrencyId::KSM, &LiquidStaking::<T>::account_id()),
            INITIAL_AMOUNT
        );

        assert_last_event::<T>(pallet_liquid_staking::Event::UnstakeProcessed(agent, caller, amount).into());
    }

}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
