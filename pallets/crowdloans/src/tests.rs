use super::{types::*, *};
use crate::mock::*;

use codec::Encode;
use frame_support::{
    assert_noop, assert_ok,
    storage::child,
    traits::{Hooks, OneSessionHandler},
};
use frame_system::RawOrigin;
use polkadot_parachain::primitives::{HeadData, ValidationCode};
use primitives::{tokens::DOT, BlockNumber, ParaId};
use sp_runtime::{
    traits::{One, UniqueSaturatedInto, Zero},
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
fn set_vrfs_should_work() {
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

        assert_ok!(Crowdloans::set_vrfs(
            frame_system::RawOrigin::Root.into(),
            vec![crowdloan]
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

        assert_ok!(Crowdloans::contribute(
            Origin::signed(ALICE), // origin
            crowdloan + 1,         // crowdloan
            amount,                // amount
            Vec::new()
        ),);
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
            pallet_assets::Error::<Test>::BalanceLow
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
            ValidationCode(vec![]),
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

fn build_storage_trie_key_path(asset_id: u32, account: AccountId) -> Vec<u8> {
    // convert module and storage key name
    let prefix = frame_support::storage::storage_prefix("Assets".as_bytes(), "Account".as_bytes());
    let hashed_asset = sp_io::hashing::blake2_128(&asset_id.to_le_bytes());
    let hashed_account = sp_io::hashing::blake2_128(account.as_ref());

    // build key based on known format
    // TODO: should probably be an array of fixed size
    let mut acc: Vec<u8> = Vec::new();
    acc.extend(&prefix);
    acc.extend(&hashed_asset);
    acc.extend(&asset_id.to_le_bytes());
    acc.extend(&hashed_account);
    acc.extend(account.encode());

    acc
}

#[test]
fn test_storage_trie_key_path() {
    new_test_ext().execute_with(|| {
        let ctoken = 15;
        let storage_key = build_storage_trie_key_path(
            ctoken,
            ALICE,
        );
        assert_eq!(
            hex::encode(storage_key),
            String::from("682a59d51ab9e48a8c8cc418ff9708d2b99d880ec681799c0cf30e8886371da96d2503563a0d2a77084af6a058b8754f0f000000c035f853fcd0f0589e30c9e2dc1a0f570101010101010101010101010101010101010101010101010101010101010101")
        );
    })
}

#[test]
fn test_storage_verify_staging_proof() {
    new_test_ext().execute_with(|| {
        // this proof was generated using the tool inside of `scripts/proof-builder/`
        use sp_trie::StorageProof;
        use sp_runtime::traits::BlakeTwo256;

        // the remote root is the "storage hash" for the block
        let remote_root = sp_core::hash::H256::from_slice(&hex::decode("9e20dfb089ee946963579c513896e7185ce5c4957c598a87f46fcf2de6630b0c").unwrap());

        // hex encoded bytes can be retrieved using the RPC api with the `getReadProof` command
        let proof_bytes = [
            hex::decode("80ffff8071c9f4a81da3724bd57947b66b622d1917588c24a363c9a0f1ff24d52957ae85809571105693d0655b50673dea6410008036bee265884cedb05983b0cd37d21cf78041206fb4f64615efdab2c19d9be85d19f43c6724634421f0066372388eb5eec580c870bdb91a347ffc3328d7fdb2a7cc8f7b6b833015102dec6aa6cbae10849d3b80421db6bc1c3fe6339ee9745e929138d1278b913fd4da4a65b194f0e73a89f5d6804a82837c1c46d11bea7eac9778de2787cbd0ac0613f7f73d09414dc8d0e22cfe804cbd3e582d6190e5f7e5010472ab1fafd3551e45042151cf10e6ab9656ce6c8a804b0195f31fc9506949b3b6ac472c785c482ad0ab1d08c5d038e6b5cc4d08e88080a92359c2d00fe1f7db043cf43d7981698cecbb616b3b3f53553ed80343a6596a807c4e9d8e70475f5ceea71644a799571644fa8c9c230135975a3dcf718db75a1f80d2bcbcc776ea608d5c181368ff2c64f6b4de78b9cecb090d862be7c47363922380acf2bbc2cb73dd2a56832f80ef5bf6867caa108cdea8191f1e768a1017bea9f380fb13b0500cab8daa90872e0c46af695ec0d43c872589327d454be5be9a2f69b88085ba31b59e40c53c4a00a0c0fbe984b967dbe21550d448db14547f644946cfeb8042d09f96bc264d850e15fd4d1f190ec25a39132aaf63ee003045323d2a80e4d580052404801076ed7511cf073f4e22279874fb4eeeb999864ce4719a38660162ca").unwrap(),
            hex::decode("80200280d550b43b862e04f0ee3ad3ee25cca4c31ae444e56668a984c2768d9132a22e768037b00c95cf699b5a922f8e83ba6d4efd88cd22a2f7e9d85dba193ecfcae9c043").unwrap(),
            hex::decode("800069805f0f307b29ac2f9ab9f01a601691d8f9e305d9b20b55074ede416347de1247bc80a301178471ef1983a10e430f4d2eb2d0ed005da2ad3029971fd33743979038e380b037bcf1a7fb96b07c43af32e1db919449e9f878905df294147f9a85b7cd4d7b8004a6f474c9573b238c3a724ec00c103cff86f2df6810cdfdd5a6a09d85d2119c").unwrap(),
            hex::decode("9e9d880ec681799c0cf30e8886371da9110280c7fbc75bc4fe1ada38b53c5103a534bd4fde99d6ae8246b2a874e9a250d614c48080ac1cf083a9e877c7da52069c477f5b132c1c76f7a48b44d519301cacad4ba280bcf03ade535c14bd0c4f8686b7186d1db18b39ffe996d318701080ab2caee522").unwrap(),
            hex::decode("9e2a59d51ab9e48a8c8cc418ff9708d21028505f0e7b9012096b41c4eb3aaf947f6ea429080000806dab79fec997f7ce468149838d183a9309cc8d41e9476e29662870554c4a93b48054f012bfa6a591cde79b914ac91a4e790a6eea962a6f70d72294fcb427754288").unwrap(),
            hex::decode("a700f701b9051ba8de3db0172b733bc759660000001021801d7f766ec9b1ce8830fbfa51458c2e1fbc1cfcc304b0f28582a3c2270a1ea2c18075b386c108347a574db0c83c4f077b7fc5a0d8786ba147bfc877acb4865fcecb803e6bca0d9d83e6a226896fddf5ce87285c61665957bc7901f4d245e08b4bbef9").unwrap(),
            hex::decode("7f200f9aea1afa791265fae359272badc1cf8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a484880588d490000000000000000000000000001").unwrap(),
          ];

        // convert trie path into proof
        let remote_proof = StorageProof::new(proof_bytes.to_vec());

        // specify the key to verify
        let key = hex::decode("682a59d51ab9e48a8c8cc418ff9708d2b99d880ec681799c0cf30e8886371da990f701b9051ba8de3db0172b733bc759660000004f9aea1afa791265fae359272badc1cf8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48").unwrap();

        let local_result =
            sp_state_machine::read_proof_check::<BlakeTwo256, _>(
                remote_root,
                remote_proof.clone(),
                [key],
            ).unwrap();

        // flatten bytes into varible for decoding
        let local_result_as_bytes = local_result.into_iter().collect::<Vec<_>>();
        let value_bytes = local_result_as_bytes[0].1.clone().unwrap();

        // first 16 bytes are a u128 encoded as le
        // this represents the balance
        let mut balance_buffer: [u8; 16] = Default::default();
        balance_buffer.copy_from_slice(&value_bytes[0..16]);
        let asset_account_balance = u128::from_le_bytes(balance_buffer);

        assert_eq!(asset_account_balance, 1_234_000_000);
    })
}
