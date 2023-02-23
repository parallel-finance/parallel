//! Benchmarks for Streaming Pallet

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Streaming;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin as SystemOrigin;
use primitives::tokens::KSM;
use sp_runtime::traits::StaticLookup;

const SEED: u32 = 0;
const INITIAL_AMOUNT: u128 = 100_000_000_000_000;

fn transfer_initial_balance<
    T: Config + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>,
>(
    caller: T::AccountId,
) {
    let account_id = T::Lookup::unlookup(caller.clone());
    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        KSM.into(),
        account_id,
        true,
        1,
    )
    .ok();
    pallet_assets::Pallet::<T>::force_set_metadata(
        SystemOrigin::Root.into(),
        KSM.into(),
        b"kusama".to_vec(),
        b"KSM".to_vec(),
        12,
        true,
    )
    .ok();
    T::Assets::mint_into(KSM, &caller, INITIAL_AMOUNT).unwrap();
}

pub fn dollar(d: u128) -> u128 {
    d.saturating_mul(10_u128.pow(12))
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
    where_clause {
        where
            T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>
        + pallet_balances::Config<Balance = Balance>
        + pallet_timestamp::Config<Moment = u64>
    }

    create {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        assert_ok!(Streaming::<T>::set_minimum_deposit(SystemOrigin::Root.into(), KSM, 0));

        let recipient: T::AccountId = account("Streaming", 101, SEED);
        let deposit_amount: u128 = dollar(5);
        let start_time: u64 = 6;
        let end_time: u64 = 18;
    }: _(SystemOrigin::Signed(caller.clone()), recipient.clone(), deposit_amount, KSM, start_time, end_time, true)
    verify {
        assert_last_event::<T>(Event::StreamCreated(0, caller, recipient, deposit_amount, KSM, start_time, end_time, true).into())
    }

    cancel {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        assert_ok!(Streaming::<T>::set_minimum_deposit(SystemOrigin::Root.into(), KSM, 0));
        let recipient: T::AccountId = account("Streaming", 101, SEED);
        let deposit_amount: u128 = dollar(5);
        let start_time: u64 = 6;
        let end_time: u64 = 18;
        assert_ok!(Streaming::<T>::create(SystemOrigin::Signed(caller.clone()).into(), recipient.clone(), deposit_amount, KSM, start_time, end_time, true));
        let stream_id: u128 = 0;
    }: _(SystemOrigin::Signed(caller.clone()), stream_id)
    verify {
        assert_last_event::<T>(Event::StreamCancelled(stream_id, caller, recipient, KSM, deposit_amount, 0).into())
    }

    withdraw {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        assert_ok!(Streaming::<T>::set_minimum_deposit(SystemOrigin::Root.into(), KSM, 0));

        let recipient: T::AccountId = account("Streaming", 101, SEED);

        let deposit_amount: u128 = dollar(5);
        let start_time: u64 = 6;
        let end_time: u64 = 18;
        assert_ok!(Streaming::<T>::create(SystemOrigin::Signed(caller).into(), recipient.clone(), deposit_amount, KSM, start_time, end_time, true));
        pallet_timestamp::Pallet::<T>::set_timestamp(18000);

        let stream_id: u128 = 0;
        let withdraw_amount: u128 = dollar(2);
    }: _(SystemOrigin::Signed(recipient.clone()), stream_id, withdraw_amount)
    verify {
        assert_last_event::<T>(Event::StreamWithdrawn(stream_id, recipient, KSM, withdraw_amount).into())
    }

    set_minimum_deposit {
        let minimum_deposit_amount: u128 = dollar(1);
    }: _(SystemOrigin::Root, KSM, minimum_deposit_amount)
    verify {
        assert_last_event::<T>(Event::MinimumDepositSet(KSM, minimum_deposit_amount).into())
    }
}

impl_benchmark_test_suite!(Streaming, crate::mock::new_test_ext(), crate::mock::Test,);
