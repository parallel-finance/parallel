//! Crowdloans pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::{types::*, *};

use crate::Pallet as Crowdloans;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{assert_ok, pallet_prelude::*, traits::fungibles::Mutate};
use frame_system::{self, RawOrigin as SystemOrigin};
use pallet_parallel_xcm::ParallelXCM;
use primitives::{ump::*, Balance, CurrencyId, ParaId, Ratio};
use sp_runtime::traits::{StaticLookup, Zero};
use sp_std::prelude::*;

use sp_runtime::traits::One;

const XCM_FEES: u128 = 50000000000u128;
const RESERVE_FACTOR: Ratio = Ratio::from_perthousand(5);
const XCM_WEIGHT: XcmWeightMisc<Weight> = XcmWeightMisc {
    bond_weight: 3_000_000_000,
    bond_extra_weight: 3_000_000_000,
    unbond_weight: 3_000_000_000,
    rebond_weight: 3_000_000_000,
    withdraw_unbonded_weight: 3_000_000_000,
    nominate_weight: 3_000_000_000,
    contribute_weight: 3_000_000_000,
    withdraw_weight: 3_000_000_000,
    add_memo_weight: 3_000_000_000,
};
const CONTRIBUTE_AMOUNT: u128 = 20000000000000u128;
const INITIAL_RESERVES: u128 = 1000000000000u128;
const INITIAL_AMOUNT: u128 = 1000000000000000u128;
const ADD_RESERVES_AMOUNT: u128 = 500000000000000u128;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn initial_set_up<T: Config + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>>(
    caller: T::AccountId,
    ctoken: u32,
) {
    let account_id = T::Lookup::unlookup(caller.clone());

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        T::RelayCurrency::get(),
        account_id.clone(),
        true,
        One::one(),
    )
    .ok();

    // force create a new ctoken asset
    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        ctoken,
        account_id,
        true,
        One::one(),
    )
    .ok();

    // fund caller with dot
    T::Assets::mint_into(T::RelayCurrency::get(), &caller, INITIAL_AMOUNT).ok();

    Crowdloans::<T>::update_xcm_fees(SystemOrigin::Root.into(), XCM_FEES).unwrap();

    T::Assets::mint_into(
        T::RelayCurrency::get(),
        &Crowdloans::<T>::account_id(),
        INITIAL_RESERVES,
    )
    .unwrap();

    T::XCM::update_reserves(INITIAL_RESERVES).unwrap();
}

benchmarks! {
    where_clause {
        where
            T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>
    }

    create_vault {
        let ctoken = 8;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1334);
        initial_set_up::<T>(caller, ctoken);
    }: _(
        SystemOrigin::Root,
        crowdloan,
        ctoken,
        ContributionStrategy::XCM,
        XcmFeesPaymentStrategy::Reserves
    )
    verify {
        assert_last_event::<T>(Event::<T>::VaultCreated(crowdloan, ctoken).into());
    }

    contribute {
        let ctoken = 9;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1335);
        initial_set_up::<T>(caller.clone(), ctoken);
        Crowdloans::<T>::update_reserve_factor(
            SystemOrigin::Root.into(),
            RESERVE_FACTOR,
        )
        .unwrap();
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, XcmFeesPaymentStrategy::Reserves));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
    }: _(
        SystemOrigin::Signed(caller.clone()),
        crowdloan,
        CONTRIBUTE_AMOUNT,
        Vec::new()
    )
    verify {
        assert_last_event::<T>(Event::VaultContributed(crowdloan, caller, CONTRIBUTE_AMOUNT, Vec::new()).into())
    }

    open {
        let ctoken = 10;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1336);
        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, XcmFeesPaymentStrategy::Reserves));
    }: _(
        SystemOrigin::Root,
        crowdloan
    )
    verify {
        assert_last_event::<T>(Event::VaultOpened(crowdloan, Zero::zero()).into())
    }

    close {
        let ctoken = 11;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1337);
        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, XcmFeesPaymentStrategy::Reserves));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
    }: _(
        SystemOrigin::Root,
        crowdloan
    )
    verify {
        assert_last_event::<T>(Event::VaultClosed(crowdloan).into())
    }

    toggle_vrf_delay {
        let ctoken = 12;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1338);
        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, XcmFeesPaymentStrategy::Reserves));
    }: _(
        SystemOrigin::Root
    )
    verify {
        assert_last_event::<T>(Event::VrfDelayToggled(true).into())
    }

    reopen {
        let ctoken = 13;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1339);
        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, XcmFeesPaymentStrategy::Reserves));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));
    }: _(
        SystemOrigin::Root,
        crowdloan
    )
    verify {
        assert_last_event::<T>(Event::VaultReOpened(crowdloan).into())
    }

    auction_failed {
        let ctoken = 14;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1340);
        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, XcmFeesPaymentStrategy::Reserves));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
        Crowdloans::<T>::update_reserve_factor(
            SystemOrigin::Root.into(),
            RESERVE_FACTOR,
        )
        .unwrap();
        assert_ok!(Crowdloans::<T>::contribute(SystemOrigin::Signed(caller).into(), crowdloan, CONTRIBUTE_AMOUNT, Vec::new()));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));

    }: _(
        SystemOrigin::Root,
        crowdloan
    )
    verify {
        assert_last_event::<T>(Event::VaultAuctionFailed(crowdloan).into())
    }

    claim_refund {
        let ctoken = 15;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1341);
        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, XcmFeesPaymentStrategy::Reserves));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::contribute(SystemOrigin::Signed(caller.clone()).into(), crowdloan, CONTRIBUTE_AMOUNT, Vec::new()));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::auction_failed(SystemOrigin::Root.into(), crowdloan));
    }: _(
        SystemOrigin::Signed(caller.clone()),
        ctoken,
        1_000
    )
    verify {
        assert_last_event::<T>(Event::VaultClaimRefund(ctoken, caller, 1_000).into())
    }

    slot_expired {
        let ctoken = 16;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1342);
        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, XcmFeesPaymentStrategy::Reserves));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::contribute(SystemOrigin::Signed(caller).into(), crowdloan, CONTRIBUTE_AMOUNT, Vec::new()));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));
    }: _(
        SystemOrigin::Root,
        crowdloan
    )
    verify {
        assert_last_event::<T>(Event::VaultSlotExpired(crowdloan).into())
    }

    update_reserve_factor {
    }: _(SystemOrigin::Root, RESERVE_FACTOR)
    verify {
        assert_last_event::<T>(Event::ReserveFactorUpdated(RESERVE_FACTOR).into())
    }

    update_xcm_fees {
    }: _(SystemOrigin::Root, XCM_FEES)
    verify {
        assert_last_event::<T>(Event::XcmFeesUpdated(XCM_FEES).into())
    }

    update_xcm_weight {
    }: _(SystemOrigin::Root, XCM_WEIGHT)
    verify {
        assert_last_event::<T>(Event::XcmWeightUpdated(XCM_WEIGHT).into())
    }

    add_reserves {
        let ctoken = 17;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1343);
        initial_set_up::<T>(caller.clone(), ctoken);
    }: _(
        SystemOrigin::Root,
        T::Lookup::unlookup(caller.clone()),
        ADD_RESERVES_AMOUNT
    )
    verify {
        assert_last_event::<T>(Event::ReservesAdded(caller, ADD_RESERVES_AMOUNT).into())
    }
}

impl_benchmark_test_suite!(Crowdloans, crate::mock::new_test_ext(), crate::mock::Test,);
