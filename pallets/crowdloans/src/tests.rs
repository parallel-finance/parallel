use super::*;
use crate::mock::*;
use frame_support::assert_ok;
use primitives::tokens;
use sp_runtime::{
    traits::{One, UniqueSaturatedInto}
};
use frame_system::{RawOrigin};

#[test]
fn create_new_vault_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = 1337;
        let currency = tokens::DOT;
        let ctoken = 10;

        let contribution_strategy = ContributionStrategy::XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            Crowdloan::account_id(),
            true,
            One::one(),
        ));

        assert_ok!(Crowdloan::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            currency,                             // token
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        if let Some(just_created_vault) = Crowdloan::vaults(crowdloan) {
            assert_eq!(
                just_created_vault,
                Vault {
                    ctoken,
                    currency,
                    phase: VaultPhase::CollectingContributions,
                    contribution_strategy: contribution_strategy,
                    contributed: Zero::zero(),
                }
            );
        }
    });
}

#[test]
fn contribute_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = 1337;
        let currency = tokens::DOT;
        let ctoken = 10;
        let amount = 1_000;

        let contribution_strategy = ContributionStrategy::XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            Crowdloan::account_id(),
            true,
            One::one(),
        ));

        // create a vault to contribute to
        assert_ok!(Crowdloan::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            currency,                             // token
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        // do contribute
        assert_ok!(Crowdloan::contribute(
            Origin::signed(ALICE), // origin
            crowdloan,             // crowdloan
            amount,                // amount
        ));

        // check that we're in the right phase
        if let Some(vault) = Crowdloan::vaults(crowdloan) {
            assert_eq!(vault.phase, VaultPhase::CollectingContributions);

            // check if ctoken minted to user
            let ctoken_balance = Assets::balance(vault.ctoken, ALICE);

            assert_eq!(ctoken_balance, amount);

            // check user balance
            let pallet_balance = Assets::balance(vault.currency, Crowdloan::account_id());

            // check pallet balance
            assert_eq!(pallet_balance, amount);
        }
    });
}

#[test]
fn participate_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = 1337;
        let currency = tokens::DOT;
        let ctoken = 10;

        let contribution_strategy = ContributionStrategy::XCM; //XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloan::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            currency,                             // token
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        // do contribute
        assert_ok!(Crowdloan::participate(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // vault.contributed should equal total_issuance(vault.currency)
        if let Some(vault) = Crowdloan::vaults(crowdloan) {
            let currency_issuance = Assets::total_issuance(vault.currency);
            assert_eq!(vault.contributed, currency_issuance);
        }
    });
}

#[test]
fn close_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = 1337;
        let currency = tokens::DOT;
        let ctoken = 10;

        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloan::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            currency,                             // token
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        // do close
        assert_ok!(Crowdloan::close(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // check that we're in the right phase
        if let Some(vault) = Crowdloan::vaults(crowdloan) {
            assert_eq!(vault.phase, VaultPhase::Closed)
        }
    });
}

#[test]
fn auction_failed_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = 1337;
        let currency = tokens::DOT;
        let ctoken = 10;

        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloan::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            currency,                             // token
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        // do close
        assert_ok!(Crowdloan::close(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // set to failed
        assert_ok!(Crowdloan::auction_failed(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // check that we're in the right phase
        if let Some(vault) = Crowdloan::vaults(crowdloan) {
            assert_eq!(vault.phase, VaultPhase::Failed)
        }
    });
}

#[test]
fn claim_refund_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = 1337;
        let currency = tokens::DOT;
        let ctoken = 10;
        let amount = 1_000;

        let contribution_strategy = ContributionStrategy::XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            Crowdloan::account_id(),
            true,
            One::one(),
        ));

        // create a vault to contribute to
        assert_ok!(Crowdloan::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            currency,                             // token
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        // do contribute
        assert_ok!(Crowdloan::contribute(
            Origin::signed(ALICE), // origin
            crowdloan,             // crowdloan
            amount,                // amount
        ));

        // do close
        assert_ok!(Crowdloan::close(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // set to failed
        assert_ok!(Crowdloan::auction_failed(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // do claim
        assert_ok!(Crowdloan::claim_refund(
            Origin::signed(ALICE), // origin
            crowdloan,             // crowdloan
            amount                 // amount
        ));

        // check that we're in the right phase
        if let Some(vault) = Crowdloan::vaults(crowdloan) {
            assert_eq!(vault.phase, VaultPhase::Closed)
        }
    });
}

#[test]
fn slot_expired_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = 1337;
        let currency = tokens::DOT;
        let ctoken = 10;

        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloan::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            currency,                             // token
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        // do close
        assert_ok!(Crowdloan::slot_expired(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // check that we're in the right phase
        if let Some(vault) = Crowdloan::vaults(crowdloan) {
            assert_eq!(vault.phase, VaultPhase::Expired)
        }
    });
}
