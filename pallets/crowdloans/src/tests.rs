use super::{types::*, *};
use crate::mock::*;

use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use primitives::{tokens, ParaId};
use sp_runtime::{
    traits::{One, UniqueSaturatedInto, Zero},
    MultiAddress::Id,
};

#[test]
fn create_new_vault_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let ctoken = 10;

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

        let just_created_vault = Crowdloans::vaults(crowdloan).unwrap();
        assert_eq!(
            just_created_vault,
            Vault {
                ctoken,
                phase: VaultPhase::CollectingContributions,
                contribution_strategy: contribution_strategy,
                contributed: Zero::zero(),
            }
        );
    });
}

#[test]
fn contribute_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let ctoken = 10;
        let amount = 1_000;

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

        // do contribute
        assert_ok!(Crowdloans::contribute(
            Origin::signed(ALICE), // origin
            crowdloan,             // crowdloan
            amount,                // amount
        ));

        // check that we're in the right phase
        let vault = Crowdloans::vaults(crowdloan).unwrap();
        assert_eq!(vault.phase, VaultPhase::CollectingContributions);

        // check if ctoken minted to user
        let ctoken_balance = Assets::balance(vault.ctoken, ALICE);

        assert_eq!(ctoken_balance, amount);
    });
}

#[test]
fn contribute_should_fail_insufficent_funds() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let ctoken = 10;
        let amount = 1_000;

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

        // do contribute
        assert_noop!(
            Crowdloans::contribute(
                Origin::signed(BOB), // origin
                crowdloan,           // crowdloan
                amount,              // amount
            ),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn participate_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let ctoken = 10;
        let amount = 100_000;

        let contribution_strategy = ContributionStrategy::XCM; //XCM;

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

        // do contribute
        assert_ok!(Crowdloans::contribute(
            Origin::signed(ALICE), // origin
            crowdloan,             // crowdloan
            amount,                // amount
        ));

        // do participate
        assert_ok!(Crowdloans::participate(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        let dot_bal = Assets::balance(tokens::DOT, ALICE);

        assert_eq!(999_999_900_000, dot_bal)
    });
}

#[test]
fn close_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let ctoken = 10;

        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        // do close
        assert_ok!(Crowdloans::close(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // check that we're in the right phase
        let vault = Crowdloans::vaults(crowdloan).unwrap();
        assert_eq!(vault.phase, VaultPhase::Closed)
    });
}

#[test]
fn auction_failed_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = 1337;
        let ctoken = 10;

        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
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
        let vault = Crowdloans::vaults(ParaId::from(crowdloan)).unwrap();
        assert_eq!(vault.phase, VaultPhase::Failed)
    });
}

#[test]
fn claim_refund_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = 1337;
        let ctoken = 10;
        let amount = 1_000;

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
        ));

        // do contribute
        assert_ok!(Crowdloans::contribute(
            Origin::signed(ALICE),   // origin
            ParaId::from(crowdloan), // crowdloan
            amount,                  // amount
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
            Origin::signed(ALICE),   // origin
            ParaId::from(crowdloan), // crowdloan
            amount                   // amount
        ));

        // check that we're in the right phase
        let vault = Crowdloans::vaults(ParaId::from(crowdloan)).unwrap();
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

        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
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
        let vault = Crowdloans::vaults(ParaId::from(crowdloan)).unwrap();
        assert_eq!(vault.phase, VaultPhase::Expired)
    });
}
