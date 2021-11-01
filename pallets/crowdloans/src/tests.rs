use super::*;
use crate::mock::*;
use crowdloan_structs::*;
use cumulus_primitives_core::ParaId;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use primitives::tokens;
use sp_runtime::traits::Zero;
use sp_runtime::traits::{One, UniqueSaturatedInto};
use sp_runtime::MultiAddress::Id;

#[test]
fn create_new_vault_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let relay_currency = tokens::DOT;
        let ctoken = 10;

        let contribution_strategy = ContributionStrategy::XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            sp_runtime::MultiAddress::Id(Crowdloan::account_id()),
            true,
            One::one(),
        ));

        assert_ok!(Crowdloan::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            relay_currency,                       // token
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        if let Some(just_created_vault) = Crowdloan::vaults(crowdloan) {
            assert_eq!(
                just_created_vault,
                Vault {
                    ctoken,
                    relay_currency,
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
        let crowdloan = ParaId::from(1337);
        let currency = tokens::DOT;
        let ctoken = 10;
        let amount = 1_000;

        let contribution_strategy = ContributionStrategy::XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            sp_runtime::MultiAddress::Id(Crowdloan::account_id()),
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
            let pallet_balance = Assets::balance(vault.relay_currency, Crowdloan::account_id());

            // check pallet balance
            assert_eq!(pallet_balance, amount);
        }
    });
}

#[test]
fn contribute_should_fail_insufficent_funds() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337);
        let currency = tokens::XDOT;
        let ctoken = 10;
        let amount = 1_000;

        let contribution_strategy = ContributionStrategy::XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            sp_runtime::MultiAddress::Id(Crowdloan::account_id()),
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
        assert_noop!(
            Crowdloan::contribute(
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
        let currency = tokens::DOT;
        let ctoken = 10;
        let amount = 100_000;

        let contribution_strategy = ContributionStrategy::XCM; //XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            sp_runtime::MultiAddress::Id(Crowdloan::account_id()),
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

        // do participate
        assert_ok!(Crowdloan::participate(
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
            ParaId::from(crowdloan),              // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        // do close
        assert_ok!(Crowdloan::close(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
        ));

        // set to failed
        assert_ok!(Crowdloan::auction_failed(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
        ));

        // check that we're in the right phase
        if let Some(vault) = Crowdloan::vaults(ParaId::from(crowdloan)) {
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
            Id(Crowdloan::account_id()),
            true,
            One::one(),
        ));

        // create a vault to contribute to
        assert_ok!(Crowdloan::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            currency,                             // token
            ParaId::from(crowdloan),              // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        // do contribute
        assert_ok!(Crowdloan::contribute(
            Origin::signed(ALICE),   // origin
            ParaId::from(crowdloan), // crowdloan
            amount,                  // amount
        ));

        // do close
        assert_ok!(Crowdloan::close(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
        ));

        // set to failed
        assert_ok!(Crowdloan::auction_failed(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
        ));

        // do claim
        assert_ok!(Crowdloan::claim_refund(
            Origin::signed(ALICE),   // origin
            ParaId::from(crowdloan), // crowdloan
            amount                   // amount
        ));

        // check that we're in the right phase
        if let Some(vault) = Crowdloan::vaults(ParaId::from(crowdloan)) {
            // vault should be in a state we allow
            assert!(
                vault.phase == VaultPhase::Failed || vault.phase == VaultPhase::Expired,
                "Vault in incorrect state"
            );
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
            ParaId::from(crowdloan),              // crowdloan
            ctoken,                               // ctoken
            contribution_strategy,                // contribution_strategy
        ));

        // do close
        assert_ok!(Crowdloan::slot_expired(
            frame_system::RawOrigin::Root.into(), // origin
            ParaId::from(crowdloan),              // crowdloan
        ));

        // check that we're in the right phase
        if let Some(vault) = Crowdloan::vaults(ParaId::from(crowdloan)) {
            assert_eq!(vault.phase, VaultPhase::Expired)
        }
    });
}

#[allow(dead_code)]
/// helper for showing events on other chains
fn print_events<T: frame_system::Config>(context: &str) {
    println!("------ {:?} events ------", context);
    frame_system::Pallet::<T>::events().iter().for_each(|r| {
        println!("{:?}", r.event);
    });
}
