use super::*;
use crate::mock::*;
use crowdloan_structs::*;
use cumulus_primitives_core::ParaId;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use primitives::tokens;
use sp_runtime::traits::Zero;
use sp_runtime::traits::{One, UniqueSaturatedInto};
use sp_runtime::MultiAddress::Id;
use xcm_simulator::TestExt;

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
fn participate_should_work() {
    // set vars we'll use in both para and relay chain env
    let xcm_transfer_amount = 2 * DOT_DECIMAL;
    let crowdloan = ParaId::from(2000);
    let currency = tokens::DOT;
    let ctoken = 10;

    // make sure we start with correct amount
    Relay::execute_with(|| {
        // show account we should fund on relay
        let fund_account = RelayCrowdloan::fund_account_id(crowdloan);
        println!("Fund Account:\t{:?}", fund_account);

        // get fund account starting balance
        let fund_account_starting_balance = RelayBalances::free_balance(fund_account);
        println!(
            "Fund Account Starting Balance:\t{:?}",
            fund_account_starting_balance
        );

        // get derivative para account starting balance
        let relay_crowdloan_starting_bal =
            RelayBalances::free_balance(Crowdloan::derivative_para_account_id());
        println!("Starting Balance:\t{:?}", relay_crowdloan_starting_bal);

        // relay derivative para account should be empty
        assert_eq!(relay_crowdloan_starting_bal, 0)
    });

    // execute crowdloan function on para chain
    ParaA::execute_with(|| {
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

        let dot_bal = Assets::balance(tokens::DOT, Crowdloan::account_id());
        println!("Before Contribute:\t{:?}", dot_bal);

        // do contribute
        assert_ok!(Crowdloan::contribute(
            Origin::signed(ALICE), // origin
            crowdloan,             // crowdloan
            xcm_transfer_amount,   // amount
        ));

        let dot_bal = Assets::balance(tokens::DOT, Crowdloan::account_id());
        println!("After Contribute:\t{:?}", dot_bal);

        // pallet should have a balance equal to the amount transfered
        assert_eq!(dot_bal, xcm_transfer_amount);

        // do participate
        assert_ok!(Crowdloan::participate(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        let dot_bal = Assets::balance(tokens::DOT, Crowdloan::account_id());

        // TODO:
        // the balance should be 0 after calling participate
        // we should have moved all of the tokens to the target
        // crowdloan at this point
        println!("After Participate:\t{:?}", dot_bal);
        // assert_eq!(dot_bal, 0);
    });

    // now lets view the events and balance on the relay
    Relay::execute_with(|| {
        print_events::<westend_runtime::Runtime>("Relay");

        // TODO:
        // check that the event we expect to happen has been emitted on the relaychain
        // let expected_event = RelayEvent::Crowdloan(RelayCrowdloanEvent::Contributed(
        //     Crowdloan::derivative_para_account_id(),
        //     crowdloan,
        //     xcm_transfer_amount,
        // ));
        // RelaySystem::assert_has_event(x);

        // TODO:
        // derivative para account should be 0 again!
        // we should have transfered tokens to this account
        // then we should have moved those tokens into the crowdloan
        let relay_crowdloan_ending_bal =
            RelayBalances::free_balance(Crowdloan::derivative_para_account_id());
        println!("Ending Balance:\t{:?}", relay_crowdloan_ending_bal);

        // relay derivative para account should be empty
        assert_eq!(relay_crowdloan_ending_bal, 0);

        // TODO:
        // get the final balance of the crowdloan we funded
        // this value should be equal to xcm_transfer_amount minus fees
        let fund_account_starting_balance =
            RelayBalances::free_balance(RelayCrowdloan::fund_account_id(crowdloan));
        println!(
            "Fund Account Ending Balance:\t{:?}",
            fund_account_starting_balance
        );

        assert_eq!(fund_account_starting_balance, xcm_transfer_amount - 20_000);
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
        println!("{:#?}", r.event);
    });
}
