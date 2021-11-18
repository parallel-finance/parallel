//! Crowdloans pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as Crowdloans;

use frame_benchmarking::{
    benchmarks_instance_pallet, impl_benchmark_test_suite, 
};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::tokens::*;
use primitives::ump::XcmWeightMisc;
use primitives::ParaId;
use primitives::Ratio;
use primitives::{tokens, CurrencyId};
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

use frame_support::{
    pallet_prelude::*,
    traits::{
        fungibles::{Inspect, Mutate},
    },
};

use primitives::Balance;

use sp_runtime::traits::{
    One,
};

const CTOKEN: u32 = 10;
const CROWDLOAN: ParaId = ParaId::from(1337);
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
const ASSET: CurrencyId = DOT;
const INITIAL_AMOUNT: u128 = 1_000_000_000_000_000;

fn assert_last_event<T: Config + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>>(
    generic_event: <T as Config>::Event,
) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn initial_set_up<T: Config + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>>(
    caller: T::AccountId,
) where
    <T::Assets as Inspect<T::AccountId>>::Balance: From<u128>,
{
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
        CTOKEN,
        account_id.clone(),
        true,
        One::one(),
    )
    .ok();

    // fund caller with dot
    T::Assets::mint_into(tokens::DOT, &caller, INITIAL_AMOUNT).ok();
}

benchmarks_instance_pallet! {
    where_clause {
        where
            <T::Assets as Inspect<T::AccountId>>::Balance: From<u128>,
            <T::Assets as Inspect<T::AccountId>>::AssetId: From<u32>,
    }

    create_vault {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
    }: _(
        SystemOrigin::Root,
        CROWDLOAN,
        CTOKEN,
        ContributionStrategy::XCM,
    )
    verify {
        assert_last_event::<T>(Event::VaultCreated(CROWDLOAN, CTOKEN));
    }

    contribute {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
    }: _(
        SystemOrigin::Signed(caller.clone()),
        CROWDLOAN,
        CONTRIBUTE_AMOUNT,
    )
    verify {
        assert_last_event::<T>(Event::VaultContributed(CROWDLOAN, caller, CONTRIBUTE_AMOUNT))
    }

    participate {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());

        // contribute before participating
        assert_ok!(Crowdloans::contribute(
            SystemOrigin::Signed(caller.clone()),
            CROWDLOAN,
            CONTRIBUTE_AMOUNT,
        ));

    }: _(
        SystemOrigin::Signed(caller.clone()),
        CROWDLOAN,
    )
    verify {
        assert_last_event::<T>(Event::VaultParticipated(CROWDLOAN, CONTRIBUTE_AMOUNT))
    }

    close {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
    }: _(
        SystemOrigin::Signed(caller.clone()),
        CROWDLOAN,
    )
    verify {
        assert_last_event::<T>(Event::VaultClosed(CROWDLOAN))
    }

    auction_failed {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
    }: _(
        SystemOrigin::Signed(caller.clone()),
        CROWDLOAN,
    )
    verify {
        assert_last_event::<T>(Event::VaultAuctionFailed(CROWDLOAN))
    }

    claim_refund {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
    }: _(
        SystemOrigin::Signed(caller.clone()),
        CROWDLOAN,
    )
    verify {
        assert_last_event::<T>(Event::VaultClaimRefund(CROWDLOAN, caller, CONTRIBUTE_AMOUNT))
    }

    slot_expired {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
    }: _(
        SystemOrigin::Signed(caller.clone()),
        CROWDLOAN,
    )
    verify {
        assert_last_event::<T>(Event::VaultSlotExpired(CROWDLOAN))
    }

    update_reserve_factor {
    }: _(SystemOrigin::Root, RESERVE_FACTOR)
    verify {
        assert_last_event::<T>(Event::ReserveFactorUpdated(RESERVE_FACTOR))
    }

    update_xcm_fees_compensation {
    }: _(SystemOrigin::Root, XCM_FEES_COMPENSATION)
    verify {
        assert_last_event::<T>(Event::XcmFeesCompensationUpdated(XCM_FEES_COMPENSATION))
    }

    update_xcm_weight {
    }: _(SystemOrigin::Root, XCM_WEIGHT)
    verify {
        assert_last_event::<T>(Event::XcmWeightUpdated(XCM_WEIGHT))
    }
}

impl_benchmark_test_suite!(Crowdloans, crate::mock::new_test_ext(), crate::mock::Test,);
