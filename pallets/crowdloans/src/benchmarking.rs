//! Crowdloans pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::{types::*, *};

use crate::Pallet as Crowdloans;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{assert_ok, pallet_prelude::*, traits::fungibles::Mutate};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::{ump::*, Balance, BlockNumber, CurrencyId, ParaId};
use sp_runtime::traits::{StaticLookup, Zero};
use sp_std::prelude::*;

use sp_runtime::traits::One;

const XCM_FEES: u128 = 50000000000u128;
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

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn initial_set_up<
    T: Config
        + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>
        + pallet_xcm_helper::Config,
>(
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
    <T as pallet_xcm_helper::Config>::Assets::mint_into(
        T::RelayCurrency::get(),
        &caller,
        INITIAL_AMOUNT,
    )
    .ok();

    Crowdloans::<T>::update_xcm_fees(SystemOrigin::Root.into(), XCM_FEES).unwrap();

    <T as pallet_xcm_helper::Config>::Assets::mint_into(
        T::RelayCurrency::get(),
        &pallet_xcm_helper::Pallet::<T>::account_id(),
        INITIAL_RESERVES,
    )
    .unwrap();
}

benchmarks! {
    where_clause {
        where
            T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance> + pallet_xcm_helper::Config
    }

    create_vault {
        let ctoken = 8;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1334u32);
        let cap = 1_000_000_000_000;
        let end_block = 1_000_000_000u32;

        initial_set_up::<T>(caller, ctoken);
    }: _(
        SystemOrigin::Root,
        crowdloan,
        ctoken,
        ContributionStrategy::XCM,
        cap,
        end_block
    )
    verify {
        assert_last_event::<T>(Event::<T>::VaultCreated(crowdloan, ctoken).into())
    }

    update_vault {
        let ctoken = 8;
        let crowdloan = ParaId::from(1334u32);
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller, ctoken);
    }: _(
        SystemOrigin::Root,
        crowdloan,
        Some(1_000_000_000_001),
        Some(1_000_000_001u32),
        Some(ContributionStrategy::XCM)
    )
    verify {
        assert_last_event::<T>(Event::<T>::VaultUpdated(crowdloan).into())
    }

    contribute {
        let ctoken = 9;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1335u32);
        let cap = 1_000_000_000_000;
        let end_block = 1_000_000_000u32;

        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, cap, end_block));
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
        let crowdloan = ParaId::from(1336u32);
        let cap = 1_000_000_000_000;
        let end_block = 1_000_000_000u32;

        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, cap, end_block));
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
        let crowdloan = ParaId::from(1337u32);
        let cap = 1_000_000_000_000;
        let end_block = 1_000_000_000u32;

        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, cap, end_block));
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
        let crowdloan = ParaId::from(1338u32);
        let cap = 1_000_000_000_000;
        let end_block = 1_000_000_000u32;

        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, cap, end_block));
    }: _(
        SystemOrigin::Root
    )
    verify {
        assert_last_event::<T>(Event::VrfDelayToggled(true).into())
    }

    reopen {
        let ctoken = 13;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1339u32);
        let cap = 1_000_000_000_000;
        let end_block = 1_000_000_000u32;

        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, cap, end_block));
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
        let crowdloan = ParaId::from(1340u32);
        let cap = 1_000_000_000_000;
        let end_block = 1_000_000_000u32;

        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, cap, end_block));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
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
        let crowdloan = ParaId::from(1341u32);
        let cap = 1_000_000_000_000;
        let end_block = 1_000_000_000u32;

        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, cap, end_block));
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
        let crowdloan = ParaId::from(1342u32);
        let cap = 1_000_000_000_000;
        let end_block = 1_000_000_000u32;

        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, ContributionStrategy::XCM, cap, end_block));
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
}

impl_benchmark_test_suite!(Crowdloans, crate::mock::new_test_ext(), crate::mock::Test,);
