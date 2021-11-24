//! Crowdloans pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::{types::*, *};

use crate::Pallet as Crowdloans;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::ump::XcmWeightMisc;
use primitives::ParaId;
use primitives::Ratio;
use primitives::{tokens, CurrencyId};
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

use frame_support::{assert_ok, pallet_prelude::*, traits::fungibles::Mutate};

use primitives::Balance;

use sp_runtime::traits::One;

const XCM_FEES_COMPENSATION: u128 = 50000000000u128;
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
const INITIAL_INSURANCE: u128 = 1000000000000u128;
const INITIAL_AMOUNT: u128 = 1000000000000000u128;

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
        tokens::DOT,
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
    T::Assets::mint_into(tokens::DOT, &caller, INITIAL_AMOUNT).ok();

    Crowdloans::<T>::update_xcm_fees_compensation(SystemOrigin::Root.into(), XCM_FEES_COMPENSATION)
        .unwrap();

    T::Assets::mint_into(
        tokens::DOT,
        &Crowdloans::<T>::account_id(),
        INITIAL_INSURANCE,
    )
    .unwrap();
}

benchmarks! {
    where_clause {
        where
            T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>
    }

    create_vault {
        let ctoken = 8;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1335);
        initial_set_up::<T>(caller, ctoken);
    }: _(
        SystemOrigin::Root,
        crowdloan,
        ctoken,
        ContributionStrategy::XCM
    )
    verify {
        assert_last_event::<T>(Event::<T>::VaultCreated(crowdloan, ctoken).into());
    }

    contribute {
        let ctoken = 9;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1336);
        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM));
    }: _(
        SystemOrigin::Signed(caller.clone()),
        crowdloan,
        CONTRIBUTE_AMOUNT
    )
    verify {
        assert_last_event::<T>(Event::VaultContributed(crowdloan, caller, CONTRIBUTE_AMOUNT).into())
    }

    close {
        let ctoken = 11;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1338);
        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM));
    }: _(
        SystemOrigin::Root,
        crowdloan
    )
    verify {
        assert_last_event::<T>(Event::VaultClosed(crowdloan).into())
    }

    auction_failed {
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1339);
        let ctoken = 12;
        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM));
        Crowdloans::<T>::update_reserve_factor(
            SystemOrigin::Root.into(),
            RESERVE_FACTOR,
        )
        .unwrap();
        assert_ok!(Crowdloans::<T>::contribute(SystemOrigin::Signed(caller).into(), crowdloan, CONTRIBUTE_AMOUNT));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));

    }: _(
        SystemOrigin::Root,
        crowdloan
    )
    verify {
        assert_last_event::<T>(Event::VaultAuctionFailed(crowdloan).into())
    }

    claim_refund {
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1340);
        let ctoken = 13;
        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM));
        Crowdloans::<T>::update_reserve_factor(
            SystemOrigin::Root.into(),
            RESERVE_FACTOR,
        )
        .unwrap();
        assert_ok!(Crowdloans::<T>::contribute(SystemOrigin::Signed(caller.clone()).into(), crowdloan, CONTRIBUTE_AMOUNT));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::auction_failed(SystemOrigin::Root.into(), crowdloan));
    }: _(
        SystemOrigin::Signed(caller.clone()),
        crowdloan,
        1_000
    )
    verify {
        assert_last_event::<T>(Event::VaultClaimRefund(crowdloan, caller, 1_000).into())
    }

    slot_expired {
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1341);
        let ctoken = 14;
        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM));
        Crowdloans::<T>::update_reserve_factor(
            SystemOrigin::Root.into(),
            RESERVE_FACTOR,
        )
        .unwrap();
        assert_ok!(Crowdloans::<T>::contribute(SystemOrigin::Signed(caller).into(), crowdloan, CONTRIBUTE_AMOUNT));
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

    update_xcm_fees_compensation {
    }: _(SystemOrigin::Root, XCM_FEES_COMPENSATION)
    verify {
        assert_last_event::<T>(Event::XcmFeesCompensationUpdated(XCM_FEES_COMPENSATION).into())
    }

    update_xcm_weight {
    }: _(SystemOrigin::Root, XCM_WEIGHT)
    verify {
        assert_last_event::<T>(Event::XcmWeightUpdated(XCM_WEIGHT).into())
    }
}

impl_benchmark_test_suite!(Crowdloans, crate::mock::new_test_ext(), crate::mock::Test,);
