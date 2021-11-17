//! Benchmarks for Bridge Pallet

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Bridge;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin as SystemOrigin;
use primitives::{
    ChainId,
    CurrencyId,
};

const ETH: ChainId = 1;

const HKO: CurrencyId = 0;
const EHKO: CurrencyId = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
    where_clause {
        where
            T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>
    }

    set_vote_threshold {
        // let caller: T::AccountId = whitelisted_caller();
    }: _(SystemOrigin::Root, 1)
    verify {
        assert_last_event::<T>(Event::VoteThresholdChanged(1).into())
    }

    register_chain {
        // let caller: T::AccountId = whitelisted_caller();
    }: _(SystemOrigin::Root, ETH)
    verify {
        assert_last_event::<T>(Event::ChainRegistered(ETH).into())
    }
    
    unregister_chain {
        // let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_chain(SystemOrigin::Root.into(), ETH));
    }: _(SystemOrigin::Root, ETH)
    verify {
        assert_last_event::<T>(Event::ChainRemoved(ETH).into())
    }
    
    register_currency {
        // let caller: T::AccountId = whitelisted_caller();
    }: _(SystemOrigin::Root, HKO, EHKO)
    verify {
        assert_last_event::<T>(Event::CurrencyRegistered(HKO, EHKO).into())
    }

    unregister_currency {
        // let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_currency(SystemOrigin::Root.into(), HKO, EHKO));
    }: _(SystemOrigin::Root, EHKO)
    verify {
        assert_last_event::<T>(Event::CurrencyRemoved(HKO, EHKO).into())
    }

    teleport {
        // let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_chain(SystemOrigin::Root.into(), ETH));
        assert_ok!(Bridge::<T>::register_currency(SystemOrigin::Root.into(), HKO, EHKO));
        let tele: TeleAccount = whitelisted_caller();
    }: _(SystemOrigin::Root, ETH, 0, tele.clone(), 10000000000)
    verify {
        assert_last_event::<T>(Event::Burned(ETH, 0, EHKO, tele, 10000000000).into());
    }
    
    vote_materialize {
        // let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_chain(SystemOrigin::Root.into(), ETH));
        assert_ok!(Bridge::<T>::register_currency(SystemOrigin::Root.into(), HKO, EHKO));
        let recipient: T::AccountId = whitelisted_caller();
    }: _(SystemOrigin::Root, ETH, 0, EHKO, recipient.clone(), 10000000000, true)
    verify {
        assert_last_event::<T>(Event::Minted(ETH, 0, EHKO, recipient, 10000000000).into());
    }
}
impl_benchmark_test_suite!{Bridge, crate::mock::new_test_ext(), crate::mock::Test}

