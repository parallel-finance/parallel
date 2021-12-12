use super::{types::*, *};
use crate::mock::*;
use primitives::BlockNumber;

use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use pallet_xcm_helper::{XcmFees, XcmWeight};
use primitives::{ump::*, ParaId};
use sp_runtime::{
    traits::{One, UniqueSaturatedInto, Zero},
    MultiAddress::Id,
};

pub const VAULT_ID: u32 = 0;

#[test]
fn create_new_vault_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let ctoken = 10;
        let cap_limit = 1_000_000_000_000;
        let end_block = BlockNumber::from(10);

        let contribution_strategy = ContributionStrategy::XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            sp_runtime::MultiAddress::Id(Crowdloans::account_id()),
            true,
            One::one(),
        ));

        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
            cap_limit,                            // cap_limit
            end_block,                            // end_block
        ));

        let just_created_vault = Crowdloans::vaults(crowdloan, VAULT_ID).unwrap();
        assert_eq!(
            just_created_vault,
            Vault {
                id: VAULT_ID,
                ctoken,
                phase: VaultPhase::Pending,
                contributed: Zero::zero(),
                pending: Zero::zero(),
                contribution_strategy,
            }
        );
    });
}

#[test]
fn create_new_vault_should_not_work_if_vault_is_already_created() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let ctoken = 10;
        let cap_limit = 1_000_000_000_000;
        let end_block = BlockNumber::from(10);

        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            sp_runtime::MultiAddress::Id(Crowdloans::account_id()),
            true,
            One::one(),
        ));

        Assets::mint(
            Origin::signed(Crowdloans::account_id()),
            ctoken,
            Id(ALICE),
            dot(100f64),
        )
        .unwrap();

        assert_noop!(
            Crowdloans::create_vault(
                frame_system::RawOrigin::Root.into(), // origin
                crowdloan,                            // crowdloan
                ctoken,                               // ctoken
                ContributionStrategy::XCM,            // contribution_strategy
            ),
            Error::<Test>::CTokenAlreadyTaken
        );
    });
}

#[test]
fn create_new_vault_should_not_work_if_crowdloan_already_exists() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let ctoken = 10;
        let cap_limit = 1_000_000_000_000;
        let end_block = BlockNumber::from(10);

        let contribution_strategy = ContributionStrategy::XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            sp_runtime::MultiAddress::Id(Crowdloans::account_id()),
            true,
            One::one(),
        ));

        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        assert_noop!(
            Crowdloans::create_vault(
                frame_system::RawOrigin::Root.into(), // origin
                crowdloan,                            // crowdloan
                ctoken,                               // ctoken
                contribution_strategy,                // contribution_strategy
            ),
            Error::<Test>::CTokenAlreadyTaken
        );
    });
}

#[test]
fn contribute_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let ctoken = 10;
        let amount = 1_000;
        let cap_limit = 1_000_000_000_000;
        let end_block = BlockNumber::from(10);

        let contribution_strategy = ContributionStrategy::XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            sp_runtime::MultiAddress::Id(Crowdloans::account_id()),
            true,
            One::one(),
        ));

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        // do open
        assert_ok!(Crowdloans::open(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // do contribute
        assert_ok!(Crowdloans::contribute(
            Origin::signed(ALICE), // origin
            crowdloan,             // crowdloan
            amount,                // amount
            Vec::new()
        ));

        // check that we're in the right phase
        let vault = Crowdloans::vaults(crowdloan, VAULT_ID).unwrap();
        assert_eq!(vault.phase, VaultPhase::Contributing);

        // check if ctoken minted to user
        let ctoken_balance = Assets::balance(vault.ctoken, ALICE);

        assert_eq!(ctoken_balance, amount);
    });
}

#[test]
fn toggle_vrf_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let ctoken = 10;
        let amount = 1_000;
        let cap_limit = 1_000_000_000_000;
        let end_block = BlockNumber::from(10);

        let contribution_strategy = ContributionStrategy::XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            sp_runtime::MultiAddress::Id(Crowdloans::account_id()),
            true,
            One::one(),
        ));

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        assert_ok!(Crowdloans::toggle_vrf_delay(
            frame_system::RawOrigin::Root.into(), // origin
        ));

        // do contribute
        assert_noop!(
            Crowdloans::contribute(
                Origin::signed(ALICE), // origin
                crowdloan,             // crowdloan
                amount,                // amount
                Vec::new()
            ),
            Error::<Test>::VrfDelayInProgress
        );
    });
}

#[test]
fn contribute_should_fail_insufficent_funds() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let ctoken = 10;
        let amount = 1_000;
        let cap_limit = 1_000_000_000_000;
        let end_block = BlockNumber::from(10);

        let contribution_strategy = ContributionStrategy::XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            sp_runtime::MultiAddress::Id(Crowdloans::account_id()),
            true,
            One::one(),
        ));

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
            cap_limit,                            // cap_limit
            end_block,                            // end_block
        ));

        // do contribute
        assert_noop!(
            Crowdloans::contribute(
                Origin::signed(BOB), // origin
                crowdloan,           // crowdloan
                amount,              // amount
                Vec::new()
            ),
            pallet_assets::Error::<Test>::BalanceLow
        );
    });
}

#[test]
fn close_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let ctoken = 10;
        let cap_limit = 1_000_000_000_000;
        let end_block = BlockNumber::from(10);

        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
            cap_limit,                            // cap_limit
            end_block,                            // end_block
        ));

        // do open
        assert_ok!(Crowdloans::open(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // do close
        assert_ok!(Crowdloans::close(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // check that we're in the right phase
        let vault = Crowdloans::vaults(crowdloan, VAULT_ID).unwrap();
        assert_eq!(vault.phase, VaultPhase::Closed)
    });
}

#[test]
fn reopen_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let ctoken = 10;
        let cap_limit = 1_000_000_000_000;
        let end_block = BlockNumber::from(10);

        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
            cap_limit,                            // cap_limit
            end_block,                            // end_block
        ));

        // do open
        assert_ok!(Crowdloans::open(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // do close
        assert_ok!(Crowdloans::close(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // do reopen
        assert_ok!(Crowdloans::reopen(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // check that we're in the right phase
        let vault = Crowdloans::vaults(crowdloan, VAULT_ID).unwrap();
        assert_eq!(vault.phase, VaultPhase::Contributing)
    });
}

#[test]
fn auction_failed_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = 1337;
        let ctoken = 10;
        let cap_limit = 1_000_000_000_000;
        let end_block = BlockNumber::from(10);

        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
            cap_limit,                            // cap_limit
            end_block,                            // end_block
        ));

        // do open
        assert_ok!(Crowdloans::open(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
        ));

        // do close
        assert_ok!(Crowdloans::close(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
        ));

        // set to failed
        assert_ok!(Crowdloans::auction_failed(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
        ));

        // check that we're in the right phase
        let vault = Crowdloans::vaults(ParaId::from(crowdloan), VAULT_ID).unwrap();
        assert_eq!(vault.phase, VaultPhase::Failed)
    });
}

#[test]
fn claim_refund_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = 1337;
        let ctoken = 10;
        let amount = 1_000u128;
        let cap_limit = 1_000_000_000_000;
        let end_block = BlockNumber::from(10);

        let contribution_strategy = ContributionStrategy::XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            Id(Crowdloans::account_id()),
            true,
            One::one(),
        ));

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
            cap_limit,                            // cap_limit
            end_block,                            // end_block
        ));

        // do open
        assert_ok!(Crowdloans::open(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
        ));

        // do contribute
        assert_ok!(Crowdloans::contribute(
            Origin::signed(ALICE),   // origin
            ParaId::from(crowdloan), // crowdloan
            amount,                  // amount
            Vec::new()
        ));

        // do close
        assert_ok!(Crowdloans::close(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
        ));

        // set to failed
        assert_ok!(Crowdloans::auction_failed(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
        ));

        // do claim
        assert_ok!(Crowdloans::claim_refund(
            Origin::signed(ALICE), // origin
            ctoken,                // ctoken
            amount                 // amount
        ));

        // check that we're in the right phase
        let vault = Crowdloans::vaults(ParaId::from(crowdloan), VAULT_ID).unwrap();
        // vault should be in a state we allow
        assert!(
            vault.phase == VaultPhase::Failed || vault.phase == VaultPhase::Expired,
            "Vault in incorrect state"
        );
    });
}

#[test]
fn slot_expired_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = 1337;
        let ctoken = 10;
        let cap_limit = 1_000_000_000_000;
        let end_block = BlockNumber::from(10);

        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
            cap_limit,                            // cap_limit
            end_block,                            // end_block
        ));

        // do open
        assert_ok!(Crowdloans::open(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
        ));

        // do close
        assert_ok!(Crowdloans::close(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
        ));

        assert_ok!(Crowdloans::slot_expired(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
        ));

        // check that we're in the right phase
        let vault = Crowdloans::vaults(ParaId::from(crowdloan), VAULT_ID).unwrap();
        assert_eq!(vault.phase, VaultPhase::Expired)
    });
}

#[test]
fn update_xcm_fees_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Crowdloans::update_xcm_fees(
            frame_system::RawOrigin::Root.into(), // origin
            One::one()                            // fees
        ));

        assert_eq!(XcmFees::<Test>::get(), One::one());
    });
}

#[test]
fn update_xcm_weight_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Crowdloans::update_xcm_weight(
            frame_system::RawOrigin::Root.into(), // origin
            XcmWeightMisc::default()              // xcm_weight_misc
        ));

        assert_eq!(XcmWeight::<Test>::get(), XcmWeightMisc::default());
    });
}
