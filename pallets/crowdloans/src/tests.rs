use super::{types::*, *};
use crate::mock::*;

use codec::Encode;
use frame_support::{
    assert_err, assert_noop, assert_ok,
    storage::child,
    traits::{Hooks, OneSessionHandler},
};
use frame_system::RawOrigin;
use polkadot_parachain::primitives::{HeadData, ValidationCode};
use primitives::{tokens::DOT, BlockNumber, ParaId};
use sp_runtime::{
    traits::{One, UniqueSaturatedInto, Zero},
    DispatchError,
    MultiAddress::Id,
};
use xcm_simulator::TestExt;

pub const LEASE_START: u32 = 0;
pub const LEASE_END: u32 = 7;

#[test]
fn create_new_vault_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
        let contribution_strategy = ContributionStrategy::XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            Id(Crowdloans::account_id()),
            true,
            One::one(),
        ));

        assert_noop!(
            Crowdloans::create_vault(
                Origin::signed(EVE),   // origin
                crowdloan,             // crowdloan
                ctoken,                // ctoken
                LEASE_START,           // lease_start
                LEASE_END,             // lease_end
                contribution_strategy, // contribution_strategy
                cap,                   // cap
                end_block              // end_block
            ),
            DispatchError::BadOrigin
        );

        assert_ok!(Crowdloans::create_vault(
            Origin::signed(ALICE), // origin
            crowdloan,             // crowdloan
            ctoken,                // ctoken
            LEASE_START,           // lease_start
            LEASE_END,             // lease_end
            contribution_strategy, // contribution_strategy
            cap,                   // cap
            end_block              // end_block
        ));

        assert_noop!(
            Crowdloans::create_vault(
                Origin::signed(CHARLIE), // origin
                crowdloan,               // crowdloan
                ctoken,                  // ctoken
                LEASE_START,             // lease_start
                LEASE_END,               // lease_end
                contribution_strategy,   // contribution_strategy
                cap,                     // cap
                end_block                // end_block
            ),
            Error::<Test>::VaultAlreadyExists
        );

        let just_created_vault =
            Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        assert_eq!(
            just_created_vault,
            Vault {
                ctoken,
                phase: VaultPhase::Pending,
                contributed: Zero::zero(),
                pending: Zero::zero(),
                flying: Zero::zero(),
                contribution_strategy,
                cap,
                end_block,
                trie_index: Zero::zero(),
                lease_start: LEASE_START,
                lease_end: LEASE_END
            }
        );
    });
}

#[test]
fn create_new_vault_should_not_work_if_ctoken_is_different() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);

        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            Id(Crowdloans::account_id()),
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

        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            ContributionStrategy::XCM,            // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
        ));

        // cDOT-0-7 has been created, but now for the same lease we are using a different ctoken
        assert_noop!(
            Crowdloans::create_vault(
                frame_system::RawOrigin::Root.into(), // origin
                crowdloan,                            // crowdloan
                ctoken + 1,                           // ctoken
                LEASE_START,                          // lease_start
                LEASE_END,                            // lease_end
                ContributionStrategy::XCM,            // contribution_strategy
                cap,                                  // cap
                end_block                             // end_block
            ),
            Error::<Test>::InvalidCToken
        );
    });
}

#[test]
fn open_should_work() {
    new_test_ext().execute_with(|| {
        // Prepare vault
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
        let contribution_strategy = ContributionStrategy::XCM;
        let amount = dot(5f64);

        // create the ctoken asset
        (Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            Id(Crowdloans::account_id()),
            true,
            One::one(),
        ))
        .unwrap();

        (Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block,                            // end_block
        ))
        .unwrap();

        let vault = Crowdloans::current_vault(crowdloan).unwrap();

        Crowdloans::contribute(
            RawOrigin::Signed(ALICE).into(),
            crowdloan,
            amount,
            vec![12, 34],
        )
        .unwrap();
        let (pending, referral_code) =
            Crowdloans::contribution_get(vault.trie_index, &ALICE, ChildStorageKind::Pending);
        assert!(referral_code == vec![12, 34]);
        assert!(pending == amount);

        Crowdloans::migrate_pending(RawOrigin::Root.into(), crowdloan).unwrap();
        let (flying, referral_code2) =
            Crowdloans::contribution_get(vault.trie_index, &ALICE, ChildStorageKind::Flying);
        assert!(referral_code2 == vec![12, 34]);
        assert!(flying == amount);

        Crowdloans::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        )
        .unwrap();

        let (contributed, referral_code3) =
            Crowdloans::contribution_get(vault.trie_index, &ALICE, ChildStorageKind::Contributed);
        assert!(referral_code3 == vec![12, 34]);
        assert!(contributed == amount);
    })
}

#[test]
fn create_new_vault_should_not_work_if_crowdloan_already_exists() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
        let contribution_strategy = ContributionStrategy::XCM;

        // create the ctoken asset
        assert_ok!(Assets::force_create(
            RawOrigin::Root.into(),
            ctoken.unique_saturated_into(),
            Id(Crowdloans::account_id()),
            true,
            One::one(),
        ));

        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
        ));

        assert_noop!(
            Crowdloans::create_vault(
                frame_system::RawOrigin::Root.into(), // origin
                crowdloan,                            // crowdloan
                ctoken,                               // ctoken
                LEASE_START,                          // lease_start
                LEASE_END,                            // lease_end
                contribution_strategy,                // contribution_strategy
                cap,                                  // cap
                end_block                             // end_block
            ),
            Error::<Test>::VaultAlreadyExists
        );
    });
}

#[test]
fn set_vrf_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let amount = 1_000;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
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
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
        ));

        // create a sibling vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan + 1,                        // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
        ));

        // do open
        assert_ok!(Crowdloans::open(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        assert_ok!(Crowdloans::set_vrf(
            frame_system::RawOrigin::Root.into(),
            true
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

        assert_noop!(
            Crowdloans::contribute(
                Origin::signed(ALICE), // origin
                crowdloan + 1,         // crowdloan
                amount,                // amount
                Vec::new()
            ),
            Error::<Test>::VrfDelayInProgress
        );
    })
}

#[test]
fn contribute_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let amount = 1_000;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
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
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
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
            vec![12, 34],
        ));

        // check that we're in the right phase
        let vault = Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        assert_eq!(vault.phase, VaultPhase::Contributing);

        Crowdloans::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        )
        .unwrap();

        // check if ctoken minted to user
        // let ctoken_balance = Assets::balance(vault.ctoken, ALICE);

        let (contributed, referral_code) =
            Crowdloans::contribution_get(vault.trie_index, &ALICE, ChildStorageKind::Contributed);
        assert!(referral_code == vec![12, 34]);
        assert!(contributed == amount);
        // assert_eq!(ctoken_balance, amount);
    });
}

#[test]
fn contribute_should_fail_insufficent_funds() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let amount = 1_000;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
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
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
        ));

        // do contribute
        assert_noop!(
            Crowdloans::contribute(
                Origin::signed(BOB), // origin
                crowdloan,           // crowdloan
                amount,              // amount
                Vec::new()
            ),
            pallet_assets::Error::<Test>::NoAccount
        );
    });
}

#[test]
fn close_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
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
        let vault = Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        assert_eq!(vault.phase, VaultPhase::Closed)
    });
}

#[test]
fn reopen_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
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
        let vault = Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        assert_eq!(vault.phase, VaultPhase::Contributing)
    });
}

#[test]
fn auction_failed_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
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

        // set to failed
        assert_ok!(Crowdloans::auction_failed(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        Crowdloans::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        )
        .unwrap();

        // check that we're in the right phase
        let vault = Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        assert_eq!(vault.phase, VaultPhase::Failed)
    });
}

#[test]
fn claim_failed_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10u32;
        let amount = 1_000u128;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
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
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
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

        assert_eq!(Assets::balance(DOT, ALICE), dot(100f64) - amount);

        Crowdloans::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        )
        .unwrap();

        // do close
        assert_ok!(Crowdloans::close(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // set to failed
        assert_ok!(Crowdloans::auction_failed(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        Crowdloans::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            1,
            Response::ExecutionResult(None),
        )
        .unwrap();

        // do withdraw
        assert_ok!(Crowdloans::withdraw(
            Origin::signed(ALICE), // origin
            crowdloan,             // ctoken
            LEASE_START,           // lease_start
            LEASE_END,             // lease_end
        ));
        assert_eq!(Assets::balance(DOT, ALICE), dot(100f64));

        // check that we're in the right phase
        let vault = Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        // vault should be in a state we allow
        assert!(
            vault.phase == VaultPhase::Failed || vault.phase == VaultPhase::Expired,
            "Vault in incorrect state"
        );
    });
}

#[test]
fn claim_succeed_and_expired_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10u32;
        let amount = 1_000u128;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
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
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
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

        Crowdloans::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        )
        .unwrap();

        // do close
        assert_ok!(Crowdloans::close(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        //////////////////////////////////
        // set to succeed
        assert_ok!(Crowdloans::auction_succeeded(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // do claim succeed
        assert_ok!(Crowdloans::claim(
            Origin::signed(ALICE), // origin
            crowdloan,             // ctoken
            LEASE_START,           // lease_start
            LEASE_END,             // lease_end
        ));
        assert_eq!(Assets::balance(ctoken, ALICE), amount);
        assert_eq!(Assets::balance(DOT, ALICE), dot(100f64) - amount);

        let vault = Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        assert!(
            vault.phase == VaultPhase::Succeeded,
            "Vault in incorrect state"
        );

        //////////////////////////////////
        // set to expired
        assert_ok!(Crowdloans::slot_expired(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        Crowdloans::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            1,
            Response::ExecutionResult(None),
        )
        .unwrap();

        // do redeem expired
        assert_ok!(Crowdloans::redeem(
            Origin::signed(ALICE), // origin
            crowdloan,             // ctoken
            LEASE_START,           // lease_start
            LEASE_END,             // lease_end
            amount                 // amount
        ));
        assert_eq!(Assets::balance(ctoken, ALICE), 0u128);
        assert_eq!(Assets::balance(DOT, ALICE), dot(100f64));

        let vault = Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        assert!(
            vault.phase == VaultPhase::Expired,
            "Vault in incorrect state"
        );
    });
}

#[test]
fn slot_expired_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
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

        // do succeed
        assert_ok!(Crowdloans::auction_succeeded(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        assert_ok!(Crowdloans::slot_expired(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        Crowdloans::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        )
        .unwrap();

        // check that we're in the right phase
        let vault = Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        assert_eq!(vault.phase, VaultPhase::Expired)
    });
}

#[test]
fn suceed_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
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

        // do succeed
        assert_ok!(Crowdloans::auction_succeeded(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // check that we're in the right phase
        let vault = Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        assert_eq!(vault.phase, VaultPhase::Succeeded)
    });
}

#[test]
fn xcm_contribute_should_work() {
    TestNet::reset();
    let crowdloan = parathread_id();
    let ctoken = 10;
    let amount = 1_000_000_000_000;
    let cap = 1_000_000_000_000_000;
    let end_block = BlockNumber::from(1_000_000_000u32);
    let contribution_strategy = ContributionStrategy::XCM;

    Relay::execute_with(|| {
        assert_ok!(RelayRegistrar::force_register(
            frame_system::RawOrigin::Root.into(),
            ALICE,
            1000,
            parathread_id(),
            HeadData(vec![]),
            ValidationCode(vec![1, 2, 3]),
        ));

        assert_ok!(RelayParas::force_queue_action(
            RawOrigin::Root.into(),
            crowdloan
        ));
        pallet_session::CurrentIndex::<KusamaRuntime>::put(1);
        <RelayInitializer as OneSessionHandler<AccountId>>::on_new_session(
            false,
            vec![].into_iter(),
            vec![].into_iter(),
        );
        RelayInitializer::on_finalize(3);
        assert_ok!(RelayCrowdloan::create(
            kusama_runtime::Origin::signed(ALICE),
            crowdloan,
            amount,
            0,
            7,
            10000,
            None
        ));
    });

    ParaA::execute_with(|| {
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
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
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
        let vault = Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        assert_eq!(vault.phase, VaultPhase::Contributing);
    });
    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Crowdloan(RelayCrowdloanEvent::Contributed(
            Crowdloans::para_account_id(),
            crowdloan,
            amount,
        )));
        // println!("relay: {:?}", RelaySystem::events());
    });
    // ParaA::execute_with(|| {
    //     println!("para: {:?}", System::events());
    // });
}

#[test]
fn put_contribution_should_work() {
    new_test_ext().execute_with(|| {
        Crowdloans::contribution_put(
            0u32,
            &ALICE,
            &dot(5.0f64),
            &[0u8],
            ChildStorageKind::Pending,
        );
        assert!(ALICE.using_encoded(|b| {
            child::exists(
                &Crowdloans::id_from_index(0u32, ChildStorageKind::Pending),
                b,
            )
        }))
    })
}

#[test]
fn kill_contribution_should_work() {
    new_test_ext().execute_with(|| {
        Crowdloans::contribution_put(
            0u32,
            &ALICE,
            &dot(5.0f64),
            &[0u8],
            ChildStorageKind::Pending,
        );
        Crowdloans::contribution_kill(0u32, &ALICE, ChildStorageKind::Pending);
        assert!(!ALICE.using_encoded(|b| {
            child::exists(
                &Crowdloans::id_from_index(0u32, ChildStorageKind::Pending),
                b,
            )
        }))
    })
}

#[test]
fn dissolve_vault_wrong_state_should_not_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
        ));

        // do open
        assert_ok!(Crowdloans::open(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // do close
        assert_noop!(
            Crowdloans::dissolve_vault(
                frame_system::RawOrigin::Root.into(), // origin
                crowdloan,                            // crowdloan
                LEASE_START,
                LEASE_END
            ),
            Error::<Test>::IncorrectVaultPhase
        );
    })
}

#[test]
fn dissolve_vault_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
        let contribution_strategy = ContributionStrategy::XCM;

        // create a vault to contribute to
        assert_ok!(Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
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

        // ctoken of the crowdloan should match above
        // we can be sure the vault exists
        assert_eq!(
            Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END))
                .unwrap()
                .ctoken,
            ctoken
        );

        // do dissolve
        // should work because no contributions were added and the vault is closed
        assert_ok!(Crowdloans::dissolve_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            LEASE_START,
            LEASE_END
        ));

        // vault does not exist anymore
        assert_eq!(
            Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)),
            None
        );
    })
}

#[test]
fn refund_should_fail_without_vault() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);

        // Execution of refund without valid vaults.
        assert_err!(
            Crowdloans::refund(
                frame_system::RawOrigin::Root.into(), // origin
                crowdloan,                            // crowdloan
                LEASE_START,                          // lease_start
                LEASE_END,                            // lease_end
            ),
            Error::<Test>::VaultDoesNotExist,
        );
    })
}

#[test]
fn refund_should_fail_when_vault_phase_is_pending() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
        let contribution_strategy = ContributionStrategy::XCM;

        // Create a vault and try refund.
        Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block,                            // end_block
        )
        .ok();

        // Execution of refund when vault phase is pending.
        assert_err!(
            Crowdloans::refund(
                frame_system::RawOrigin::Root.into(), // origin
                crowdloan,                            // crowdloan
                LEASE_START,                          // lease_start
                LEASE_END,                            // lease_end
            ),
            Error::<Test>::IncorrectVaultPhase,
        );
    })
}

#[test]
fn refund_should_work_when_vault_phase_is_closed() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
        let contribution_strategy = ContributionStrategy::XCM;
        let amount = 1_000;

        // Create a vault and try refund
        Crowdloans::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block,                            // end_block
        )
        .ok();

        // Open Vault
        Crowdloans::open(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        )
        .ok();

        Crowdloans::contribute(
            Origin::signed(ALICE), // origin
            crowdloan,             // crowdloan
            amount,                // amount
            vec![],
        )
        .ok();

        let vault = Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        assert_eq!(Crowdloans::total_contribution(&vault).unwrap(), amount);

        Crowdloans::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        )
        .unwrap();

        // Close Vault
        Crowdloans::close(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        )
        .ok();

        assert_ok!(Crowdloans::refund(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
        ));
        let vault = Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        assert_eq!(Crowdloans::total_contribution(&vault).unwrap(), 0);
    })
}

#[test]
fn claim_for_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10u32;
        let amount = 1_000u128;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
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
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
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

        Crowdloans::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        )
        .unwrap();

        // do close
        assert_ok!(Crowdloans::close(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        //////////////////////////////////
        // set to succeed
        assert_ok!(Crowdloans::auction_succeeded(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        // do claim_for succeed
        assert_ok!(Crowdloans::claim_for(
            Origin::signed(BOB), // origin
            Id(ALICE),           //dest
            crowdloan,           // ctoken
            LEASE_START,         // lease_start
            LEASE_END,           // lease_end
        ));
        assert_eq!(Assets::balance(ctoken, ALICE), amount);
        assert_eq!(Assets::balance(DOT, ALICE), dot(100f64) - amount);

        let vault = Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        assert!(
            vault.phase == VaultPhase::Succeeded,
            "Vault in incorrect state"
        );
    });
}

#[test]
fn withdraw_for_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = ParaId::from(1337u32);
        let ctoken = 10u32;
        let amount = 1_000u128;
        let cap = 1_000_000_000_000;
        let end_block = BlockNumber::from(1_000_000_000u32);
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
            crowdloan,                            // crowdloan
            ctoken,                               // ctoken
            LEASE_START,                          // lease_start
            LEASE_END,                            // lease_end
            contribution_strategy,                // contribution_strategy
            cap,                                  // cap
            end_block                             // end_block
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

        assert_eq!(Assets::balance(DOT, ALICE), dot(100f64) - amount);

        Crowdloans::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        )
        .unwrap();

        // do close
        assert_ok!(Crowdloans::close(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        //////////////////////////////////
        // set to succeed
        assert_ok!(Crowdloans::auction_failed(
            frame_system::RawOrigin::Root.into(), // origin
            crowdloan,                            // crowdloan
        ));

        Crowdloans::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            1,
            Response::ExecutionResult(None),
        )
        .unwrap();

        // do withdraw_for succeed
        assert_ok!(Crowdloans::withdraw_for(
            Origin::signed(BOB), // origin
            Id(ALICE),           //dest
            crowdloan,           // ctoken
            LEASE_START,         // lease_start
            LEASE_END,           // lease_end
        ));
        assert_eq!(Assets::balance(DOT, ALICE), dot(100f64));

        let vault = Crowdloans::vaults((&crowdloan, &LEASE_START, &LEASE_END)).unwrap();
        assert!(
            vault.phase == VaultPhase::Failed,
            "Vault in incorrect state"
        );
    });
}
