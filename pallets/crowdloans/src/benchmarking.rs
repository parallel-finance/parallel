//! Crowdloans pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::{types::*, *};

use crate::Pallet as Crowdloans;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{assert_ok, pallet_prelude::*, traits::fungibles::Mutate};
use frame_system::{self, RawOrigin as SystemOrigin};
use pallet_traits::ump::{XcmCall, XcmWeightFeeMisc};
use primitives::{Balance, CurrencyId, ParaId};
use sp_runtime::traits::One;
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;
use xcm::latest::prelude::*;

const XCM_WEIGHT_FEE: XcmWeightFeeMisc<Weight, Balance> = XcmWeightFeeMisc {
    weight: 3_000_000_000,
    fee: 50000000000u128,
};
const CONTRIBUTE_AMOUNT: u128 = 20000000000000u128;
const INITIAL_FEES: u128 = 1000000000000000u128;
const INITIAL_AMOUNT: u128 = 1000000000000000u128;
const LARGE_CAP: u128 = 1_000_000_000_000_000u128;
const CAP: u128 = 1_000_000_000_000_000u128;
const LEASE_START: u32 = 0;
const LEASE_END: u32 = 7;
const END_BLOCK: u32 = 1_000_000_000u32;
const START_TRIE_INDEX: u32 = 0;

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

    pallet_xcm_helper::Pallet::<T>::update_xcm_weight_fee(
        SystemOrigin::Root.into(),
        XcmCall::AddMemo,
        XCM_WEIGHT_FEE,
    )
    .unwrap();
    // fund caller with dot
    <T as pallet_xcm_helper::Config>::Assets::mint_into(
        T::RelayCurrency::get(),
        &caller,
        INITIAL_AMOUNT,
    )
    .ok();

    <T as pallet_xcm_helper::Config>::Assets::mint_into(
        T::RelayCurrency::get(),
        &pallet_xcm_helper::Pallet::<T>::account_id(),
        INITIAL_FEES,
    )
    .unwrap();
}

benchmarks! {
    where_clause {
        where
            T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance> + pallet_xcm_helper::Config,
            <T as frame_system::Config>::Origin: From<pallet_xcm::Origin>
    }

    create_vault {
        let ctoken = 8;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1334u32);

        initial_set_up::<T>(caller, ctoken);
    }: _(
        SystemOrigin::Root,
        crowdloan,
        ctoken,
        LEASE_START,
        LEASE_END,
        ContributionStrategy::XCM,
        CAP,
        END_BLOCK.into()
    )
    verify {
        assert_last_event::<T>(Event::<T>::VaultCreated(
            crowdloan,
            (LEASE_START, LEASE_END),
            ctoken,
            VaultPhase::Pending,
            ContributionStrategy::XCM,
            CAP,
            END_BLOCK.into(),
            START_TRIE_INDEX
        ).into())
    }

    update_vault {
        let ctoken = 8;
        let crowdloan = ParaId::from(1334u32);
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller, ctoken);
        // create vault before update
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, CAP, END_BLOCK.into()));
    }: _(
        SystemOrigin::Root,
        crowdloan,
        Some(1_000_000_000_001),
        Some(1_000_000_001u32.into()),
        Some(ContributionStrategy::XCM)
    )
    verify {
        assert_last_event::<T>(Event::<T>::VaultUpdated(
            crowdloan,
            (LEASE_START, LEASE_END),
            ContributionStrategy::XCM,
            1_000_000_000_001u128,
            1_000_000_001u32.into(),
        ).into())
    }

    contribute {
        let ctoken = 9;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1335u32);

        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, CAP, END_BLOCK.into()));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
    }: _(
        SystemOrigin::Signed(caller.clone()),
        crowdloan,
        CONTRIBUTE_AMOUNT,
        Vec::new()
    )
    verify {
        assert_last_event::<T>(Event::VaultDoContributing(crowdloan, (LEASE_START, LEASE_END), caller, CONTRIBUTE_AMOUNT, Vec::new()).into())
    }

    open {
        let ctoken = 10;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1336u32);

        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, CAP, END_BLOCK.into()));
    }: _(
        SystemOrigin::Root,
        crowdloan
    )
    verify {
        assert_last_event::<T>(Event::VaultPhaseUpdated(
            crowdloan,
            (LEASE_START, LEASE_END),
            VaultPhase::Pending,
            VaultPhase::Contributing,
        ).into())
    }

    close {
        let ctoken = 11;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1337u32);

        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, CAP, END_BLOCK.into()));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
    }: _(
        SystemOrigin::Root,
        crowdloan
    )
    verify {
        assert_last_event::<T>(Event::VaultPhaseUpdated(
            crowdloan,
            (LEASE_START, LEASE_END),
            VaultPhase::Contributing,
            VaultPhase::Closed,
        ).into())
    }

    set_vrf {
    }: _(
        SystemOrigin::Root,
        true
    )
    verify {
        assert_last_event::<T>(Event::VrfUpdated(true).into())
    }

    reopen {
        let ctoken = 13;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1339u32);

        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, CAP, END_BLOCK.into()));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));
    }: _(
        SystemOrigin::Root,
        crowdloan
    )
    verify {
        assert_last_event::<T>(Event::VaultPhaseUpdated(
            crowdloan,
            (LEASE_START, LEASE_END),
            VaultPhase::Closed,
            VaultPhase::Contributing,
        ).into())
    }

    auction_succeeded {
        let ctoken = 13;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1339u32);

        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, CAP, END_BLOCK.into()));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));
    }: _(
        SystemOrigin::Root,
        crowdloan
    )
    verify {
        assert_last_event::<T>(Event::VaultPhaseUpdated(
            crowdloan,
            (LEASE_START, LEASE_END),
            VaultPhase::Closed,
            VaultPhase::Succeeded,
        ).into())
    }

    auction_failed {
        let ctoken = 14;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1340u32);

        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, LARGE_CAP, END_BLOCK.into()));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::contribute(SystemOrigin::Signed(caller).into(), crowdloan, CONTRIBUTE_AMOUNT, Vec::new()));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));

    }: _(
        SystemOrigin::Root,
        crowdloan
    )
    verify {
        assert_last_event::<T>(Event::VaultDoWithdrawing(crowdloan, (LEASE_START, LEASE_END), 0, VaultPhase::Failed).into())
    }

    claim {
        let ctoken = 15;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1341u32);

        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, LARGE_CAP, END_BLOCK.into()));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::contribute(SystemOrigin::Signed(caller.clone()).into(), crowdloan, CONTRIBUTE_AMOUNT, Vec::new()));
        assert_ok!(Crowdloans::<T>::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::auction_succeeded(SystemOrigin::Root.into(), crowdloan));
    }: _(
        SystemOrigin::Signed(caller.clone()),
        crowdloan,
        LEASE_START,
        LEASE_END
    )
    verify {
        assert_last_event::<T>(Event::VaultClaimed(crowdloan, (LEASE_START, LEASE_END), ctoken, caller, CONTRIBUTE_AMOUNT, VaultPhase::Succeeded).into())
    }

    withdraw {
        let ctoken = 15;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1341u32);

        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, LARGE_CAP, END_BLOCK.into()));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::contribute(SystemOrigin::Signed(caller.clone()).into(), crowdloan, CONTRIBUTE_AMOUNT, Vec::new()));
        assert_ok!(Crowdloans::<T>::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::auction_failed(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            1,
            Response::ExecutionResult(None),
        ));
    }: _(
        SystemOrigin::Signed(caller.clone()),
        crowdloan,
        LEASE_START,
        LEASE_END
    )
    verify {
        assert_last_event::<T>(Event::VaultWithdrew(crowdloan, (LEASE_START, LEASE_END), caller, CONTRIBUTE_AMOUNT, VaultPhase::Failed).into())
    }

    redeem {
        let ctoken = 15;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1341u32);

        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, LARGE_CAP, END_BLOCK.into()));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::contribute(SystemOrigin::Signed(caller.clone()).into(), crowdloan, CONTRIBUTE_AMOUNT, Vec::new()));
        assert_ok!(Crowdloans::<T>::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::auction_succeeded(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::slot_expired(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::claim(SystemOrigin::Signed(caller.clone()).into(), crowdloan, LEASE_START, LEASE_END));
        assert_ok!(Crowdloans::<T>::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            1,
            Response::ExecutionResult(None),
        ));
    }: _(
        SystemOrigin::Signed(caller.clone()),
        crowdloan,
        LEASE_START,
        LEASE_END,
        CONTRIBUTE_AMOUNT
    )
    verify {
        assert_last_event::<T>(Event::VaultRedeemed(crowdloan, (LEASE_START, LEASE_END), ctoken, caller, CONTRIBUTE_AMOUNT, VaultPhase::Expired).into())
    }

    slot_expired {
        let ctoken = 16;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1342u32);

        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, LARGE_CAP, END_BLOCK.into()));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::contribute(SystemOrigin::Signed(caller).into(), crowdloan, CONTRIBUTE_AMOUNT, Vec::new()));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::auction_succeeded(SystemOrigin::Root.into(), crowdloan));
    }: _(
        SystemOrigin::Root,
        crowdloan
    )
    verify {
        assert_last_event::<T>(Event::VaultDoWithdrawing(crowdloan, (LEASE_START, LEASE_END), 0, VaultPhase::Expired).into())
    }

    migrate_pending {
        let ctoken = 17;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1343u32);

        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, LARGE_CAP, END_BLOCK.into()));
        for _ in 0..10 {
            assert_ok!(Crowdloans::<T>::contribute(SystemOrigin::Signed(caller.clone()).into(), crowdloan, CONTRIBUTE_AMOUNT, Vec::new()));
        }
    }: _(
        SystemOrigin::Root,
        crowdloan
    )
    verify {
        assert_last_event::<T>(Event::AllMigrated(crowdloan, (LEASE_START, LEASE_END)).into())
    }

    notification_received {
        let ctoken = 18;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1344u32);

        initial_set_up::<T>(caller.clone(), ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, LARGE_CAP, END_BLOCK.into()));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::contribute(SystemOrigin::Signed(caller).into(), crowdloan, CONTRIBUTE_AMOUNT, Vec::new()));
    }: _(
        pallet_xcm::Origin::Response(MultiLocation::parent()),
        0u64,
        Response::ExecutionResult(None)
    )
    verify {
    }

    refund {
        let ctoken = 10;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1335u32);

        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, LARGE_CAP, END_BLOCK.into()));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));
    }: _(
        SystemOrigin::Root,
        crowdloan,
        LEASE_START,
        LEASE_END
    )
    verify {
        assert_last_event::<T>(Event::AllRefunded(crowdloan, (LEASE_START, LEASE_END)).into())
    }

    dissolve_vault {
        let ctoken = 10;
        let caller: T::AccountId = whitelisted_caller();
        let crowdloan = ParaId::from(1335u32);

        initial_set_up::<T>(caller, ctoken);
        assert_ok!(Crowdloans::<T>::create_vault(SystemOrigin::Root.into(), crowdloan, ctoken, LEASE_START, LEASE_END, ContributionStrategy::XCM, LARGE_CAP, END_BLOCK.into()));
        assert_ok!(Crowdloans::<T>::open(SystemOrigin::Root.into(), crowdloan));
        assert_ok!(Crowdloans::<T>::close(SystemOrigin::Root.into(), crowdloan));
    }: _(
        SystemOrigin::Root,
        crowdloan,
        LEASE_START,
        LEASE_END
    )
    verify {
        assert_last_event::<T>(Event::VaultDissolved(crowdloan, (LEASE_START, LEASE_END)).into())
    }

}

impl_benchmark_test_suite!(Crowdloans, crate::mock::new_test_ext(), crate::mock::Test,);
