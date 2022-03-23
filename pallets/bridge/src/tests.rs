#![cfg(test)]

use super::{mock::*, Event, *};
use frame_support::{assert_noop, assert_ok};
use primitives::tokens::HKO;

#[test]
fn change_bridge_members_works() {
    new_test_ext().execute_with(|| {
        // Get members count works
        assert_eq!(Bridge::get_members_count(), 3);
        assert_eq!(Bridge::vote_threshold(), 3);

        // After remove and swap, members count should be 2
        BridgeMembership::remove_member(Origin::root(), ALICE).unwrap();
        BridgeMembership::swap_member(Origin::root(), BOB, DAVE).unwrap();
        assert_eq!(Bridge::get_members_count(), 2);
        assert_eq!(Bridge::vote_threshold(), 2);

        BridgeMembership::add_member(Origin::root(), ALICE).unwrap();
        BridgeMembership::add_member(Origin::root(), BOB).unwrap();
        BridgeMembership::add_member(Origin::root(), EVE).unwrap();
        assert_eq!(Bridge::vote_threshold(), 4);
    });
}

#[test]
fn test_valid_threshold() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Bridge::ensure_valid_threshold(0, 1),
            Error::<Test>::InvalidVoteThreshold,
        );
        assert_ok!(Bridge::ensure_valid_threshold(1, 1));
        assert_noop!(
            Bridge::ensure_valid_threshold(2, 3),
            Error::<Test>::InvalidVoteThreshold,
        );
        assert_ok!(Bridge::ensure_valid_threshold(3, 3));
        assert_noop!(
            Bridge::ensure_valid_threshold(4, 3),
            Error::<Test>::InvalidVoteThreshold,
        );
        assert_noop!(
            Bridge::ensure_valid_threshold(4, 10),
            Error::<Test>::InvalidVoteThreshold,
        );
        assert_ok!(Bridge::ensure_valid_threshold(8, 10));
        assert_ok!(Bridge::ensure_valid_threshold(10, 10));
    })
}

#[test]
fn register_unregister_works() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Bridge::register_chain(Origin::root(), ETH),
            Error::<Test>::ChainIdAlreadyRegistered,
        );

        // Register a new chain_id succeed
        Bridge::register_chain(Origin::root(), BNB).unwrap();
        assert_noop!(
            Bridge::register_chain(Origin::root(), BNB),
            Error::<Test>::ChainIdAlreadyRegistered,
        );
        // Teleport succeed when the chain is registered
        Bridge::teleport(Origin::signed(EVE), BNB, EHKO, "TELE".into(), dollar(10)).unwrap();

        // Unregister a exist chain_id succeed
        Bridge::unregister_chain(Origin::root(), ETH).unwrap();
        assert_noop!(
            Bridge::unregister_chain(Origin::root(), ETH),
            Error::<Test>::ChainIdNotRegistered,
        );
        // Teleport fails when the chain is not registered
        assert_noop!(
            Bridge::teleport(Origin::signed(EVE), ETH, EHKO, "TELE".into(), dollar(10)),
            Error::<Test>::ChainIdNotRegistered,
        );
    });
}

#[test]
fn gift_fees_works() {
    new_test_ext().execute_with(|| {
        // A successful case
        assert_eq!(<Test as Config>::Assets::balance(USDT, &DAVE), dollar(0));
        assert_eq!(<Test as Config>::Assets::balance(HKO, &DAVE), dollar(0));

        Bridge::materialize(Origin::signed(ALICE), ETH, 0, EUSDT, DAVE, dollar(10), true).unwrap();
        Bridge::materialize(Origin::signed(BOB), ETH, 0, EUSDT, DAVE, dollar(10), true).unwrap();
        Bridge::materialize(
            Origin::signed(CHARLIE),
            ETH,
            0,
            EUSDT,
            DAVE,
            dollar(10),
            true,
        )
        .unwrap();
        assert_eq!(<Test as Config>::Assets::balance(USDT, &DAVE), dollar(10));
        assert_eq!(
            <Test as Config>::Assets::balance(HKO, &DAVE),
            dollar(25) / 1000 + dollar(1) / 100,
        );

        // A failed case
        // If the bridged amount is less than a certain threshold, no gift will be issued
        assert_eq!(<Test as Config>::Assets::balance(USDT, &BOB), dollar(0));
        assert_eq!(<Test as Config>::Assets::balance(HKO, &BOB), dollar(0));

        Bridge::materialize(Origin::signed(ALICE), ETH, 1, EUSDT, BOB, 299_000_000, true).unwrap();
        Bridge::materialize(Origin::signed(BOB), ETH, 1, EUSDT, BOB, 299_000_000, true).unwrap();
        Bridge::materialize(
            Origin::signed(CHARLIE),
            ETH,
            1,
            EUSDT,
            BOB,
            299_000_000,
            true,
        )
        .unwrap();
        assert_eq!(<Test as Config>::Assets::balance(USDT, &BOB), 299_000_000);
        assert_eq!(<Test as Config>::Assets::balance(HKO, &BOB), 0,);

        // BOB balance = 0.022 HKO
        // gift_fees = 0.025 HKO - (0.022 HKO - 0.01 HKO) = 0.013 HKO
        // final_gift = existential_deposit + 0.013 HKO = 0.023 HKO
        // final_balance = 0.022 HKO + 0.023 HKO = 0.045 HKO
        Balances::set_balance(Origin::root(), BOB, dollar(22) / 1000, dollar(0)).unwrap();

        Bridge::materialize(Origin::signed(ALICE), ETH, 2, EUSDT, BOB, dollar(10), true).unwrap();
        Bridge::materialize(Origin::signed(BOB), ETH, 2, EUSDT, BOB, dollar(10), true).unwrap();
        Bridge::materialize(
            Origin::signed(CHARLIE),
            ETH,
            2,
            EUSDT,
            BOB,
            dollar(10),
            true,
        )
        .unwrap();
        assert_eq!(
            <Test as Config>::Assets::balance(HKO, &BOB),
            dollar(35) / 1000 + dollar(1) / 100,
        );

        // BOB balance = 0.035 HKO
        // gift_fees = 0.025 HKO - (0.035 HKO - 0.01 HKO) = 0 HKO
        // final_gift = 0 HKO
        // final_balance = 0.035 HKO
        Balances::set_balance(Origin::root(), BOB, dollar(35) / 1000, dollar(0)).unwrap();

        Bridge::materialize(Origin::signed(ALICE), ETH, 3, EUSDT, BOB, dollar(10), true).unwrap();
        Bridge::materialize(Origin::signed(BOB), ETH, 3, EUSDT, BOB, dollar(10), true).unwrap();
        Bridge::materialize(
            Origin::signed(CHARLIE),
            ETH,
            3,
            EUSDT,
            BOB,
            dollar(10),
            true,
        )
        .unwrap();
        assert_eq!(
            <Test as Config>::Assets::balance(HKO, &BOB),
            dollar(35) / 1000,
        );
    })
}
#[test]
fn teleport_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(<Test as Config>::Assets::balance(HKO, &EVE), dollar(100));

        Bridge::teleport(Origin::signed(EVE), ETH, EHKO, "TELE".into(), dollar(10)).unwrap();

        assert_eq!(<Test as Config>::Assets::balance(HKO, &EVE), dollar(90));
        assert_eq!(
            <Test as Config>::Assets::balance(HKO, &Bridge::account_id()),
            dollar(10)
        );
    });
}

#[test]
fn materialize_works() {
    new_test_ext().execute_with(|| {
        // EVE has 50 HKO left, and then requests for materializing 20 EHKO
        // Current vote threshold is 2
        Bridge::teleport(Origin::signed(EVE), ETH, EHKO, "TELE".into(), dollar(50)).unwrap();
        Bridge::materialize(Origin::signed(ALICE), ETH, 0, EHKO, EVE, dollar(10), true).unwrap();
        Bridge::materialize(Origin::signed(BOB), ETH, 0, EHKO, EVE, dollar(10), true).unwrap();
        Bridge::materialize(Origin::signed(CHARLIE), ETH, 0, EHKO, EVE, dollar(10), true).unwrap();
        assert_eq!(
            <Test as Config>::Assets::balance(HKO, &Bridge::account_id()),
            dollar(40)
        );
        assert_eq!(<Test as Config>::Assets::balance(HKO, &EVE), dollar(60));

        // The chain_nonce should be unique to avoid comduplicate call
        assert_noop!(
            Bridge::materialize(Origin::signed(ALICE), ETH, 0, EHKO, EVE, dollar(10), true),
            Error::<Test>::ProposalAlreadyComplete,
        );

        // Adjust threshold with 2
        // Vote_for:    [ALICE, CHARLIE]
        // Vote_against [BOB]
        Bridge::materialize(Origin::signed(ALICE), ETH, 1, EHKO, EVE, dollar(10), true).unwrap();
        assert_eq!(<Test as Config>::Assets::balance(HKO, &EVE), dollar(60));
        Bridge::materialize(Origin::signed(BOB), ETH, 1, EHKO, EVE, dollar(10), true).unwrap();
        assert_eq!(<Test as Config>::Assets::balance(HKO, &EVE), dollar(60));
        Bridge::materialize(Origin::signed(CHARLIE), ETH, 1, EHKO, EVE, dollar(10), true).unwrap();
        assert_eq!(<Test as Config>::Assets::balance(HKO, &EVE), dollar(70));
        assert_eq!(
            <Test as Config>::Assets::balance(HKO, &Bridge::account_id()),
            dollar(30)
        );
        // Success in generating `Minted` event
        assert_events(vec![mock::Event::Bridge(Event::MaterializeMinted(
            ETH,
            1,
            EHKO,
            EVE,
            dollar(10),
        ))]);
    })
}

#[test]
fn set_bridge_token_fee_works() {
    new_test_ext().execute_with(|| {
        // Case 1: Bridge toke is HKO
        // Set HKO fee equal to 2 HKO
        Bridge::set_bridge_token_fee(Origin::root(), EHKO, dollar(1)).unwrap();

        // Initial balance of EVE is 100 HKO
        assert_eq!(<Test as Config>::Assets::balance(HKO, &EVE), dollar(100));

        Bridge::teleport(Origin::signed(EVE), ETH, EHKO, "TELE".into(), dollar(10)).unwrap();

        // After teleport 10 HKO, EVE should have 90 HKO
        assert_eq!(<Test as Config>::Assets::balance(HKO, &EVE), dollar(90));
        assert_eq!(
            <Test as Config>::Assets::balance(HKO, &Bridge::account_id()),
            dollar(10)
        );

        // Success in generating `TeleportBurned` event
        // actual amount is 9 HKO
        // fee is 1 HKO
        assert_events(vec![mock::Event::Bridge(Event::TeleportBurned(
            EVE,
            ETH,
            1,
            EHKO,
            "TELE".into(),
            dollar(9),
            dollar(1),
        ))]);

        // Case 2: Bridge toke is EUSDT
        // Set EUSDT fee equal to 1 EUSDT
        Bridge::set_bridge_token_fee(Origin::root(), EUSDT, dollar(1)).unwrap();

        // EVE has 10 USDT initialized
        Assets::mint(Origin::signed(ALICE), USDT, EVE, dollar(10)).unwrap();
        assert_eq!(<Test as Config>::Assets::balance(USDT, &EVE), dollar(10));

        // EVE teleport 10 EUSDT
        Bridge::teleport(Origin::signed(EVE), ETH, EUSDT, "TELE".into(), dollar(10)).unwrap();

        // After teleport 10 EUSDT
        // EVE should have 0 USDT
        // PalletId should receive the fee equal to 1 USDT
        assert_eq!(<Test as Config>::Assets::balance(USDT, &EVE), dollar(0));
        assert_eq!(
            <Test as Config>::Assets::balance(USDT, &Bridge::account_id()),
            dollar(1)
        );

        // Success in generating `TeleportBurned` event
        // actual amount is 9 EUSDT
        // fee is 1 EUSDT
        assert_events(vec![mock::Event::Bridge(Event::TeleportBurned(
            EVE,
            ETH,
            2,
            EUSDT,
            "TELE".into(),
            dollar(9),
            dollar(1),
        ))]);
    });
}

#[test]
fn teleport_external_currency_works() {
    new_test_ext().execute_with(|| {
        // Set EUSDT fee equal to 1 USDT
        Bridge::set_bridge_token_fee(Origin::root(), EUSDT, dollar(1)).unwrap();

        // EVE has 100 USDT initialized
        Assets::mint(Origin::signed(ALICE), USDT, EVE, dollar(100)).unwrap();
        assert_eq!(<Test as Config>::Assets::balance(USDT, &EVE), dollar(100));

        // EVE teleport 10 EUSDT
        Bridge::teleport(Origin::signed(EVE), ETH, EUSDT, "TELE".into(), dollar(10)).unwrap();

        assert_eq!(<Test as Config>::Assets::balance(USDT, &EVE), dollar(90));
        assert_eq!(
            <Test as Config>::Assets::balance(USDT, &Bridge::account_id()),
            dollar(1),
        );

        assert_events(vec![mock::Event::Bridge(Event::TeleportBurned(
            EVE,
            ETH,
            1,
            EUSDT,
            "TELE".into(),
            dollar(9),
            dollar(1),
        ))]);
    });
}

#[test]
fn materialize_external_currency_works() {
    new_test_ext().execute_with(|| {
        // External token use Assets::mint other than Balances::transfer
        assert_eq!(
            <Test as Config>::Assets::balance(USDT, &Bridge::account_id()),
            dollar(0)
        );

        // EVE has 0 USDT, and then requests for materializing 10 USDT
        // Current vote threshold is 3
        Bridge::materialize(Origin::signed(ALICE), ETH, 1, EUSDT, EVE, dollar(10), true).unwrap();
        Bridge::materialize(Origin::signed(BOB), ETH, 1, EUSDT, EVE, dollar(10), true).unwrap();
        Bridge::materialize(
            Origin::signed(CHARLIE),
            ETH,
            1,
            EUSDT,
            EVE,
            dollar(10),
            true,
        )
        .unwrap();
        assert_eq!(<Test as Config>::Assets::balance(USDT, &EVE), dollar(10));

        assert_events(vec![mock::Event::Bridge(Event::MaterializeMinted(
            ETH,
            1,
            EUSDT,
            EVE,
            dollar(10),
        ))]);

        assert_noop!(
            Bridge::teleport(Origin::signed(EVE), ETH, EUSDT, "TELE".into(), dollar(11)),
            pallet_assets::Error::<Test>::BalanceLow,
        );
        Bridge::teleport(Origin::signed(EVE), ETH, EUSDT, "TELE".into(), dollar(10)).unwrap();

        assert_eq!(<Test as Config>::Assets::balance(USDT, &EVE), dollar(0));
        assert_eq!(
            <Test as Config>::Assets::balance(USDT, &Bridge::account_id()),
            dollar(0)
        );
    })
}
#[test]
fn test_merge_overlapping_intervals() {
    // status 0: (1,1), (3,4), (6,6)
    // status 1: push 2 => (1,2), (2,4), (6,6)
    assert_eq!(
        Bridge::merge_overlapping_intervals(vec![(1, 2), (2, 4), (6, 6)]),
        vec![(1, 4), (6, 6)],
    );
    // status 2: push 5 => (1,5), (5,6)
    assert_eq!(
        Bridge::merge_overlapping_intervals(vec![(1, 5), (5, 6)]),
        vec![(1, 6)],
    );

    assert_eq!(
        Bridge::merge_overlapping_intervals(vec![(2, 5), (3, 6)]),
        vec![(2, 6)],
    );

    assert_eq!(
        Bridge::merge_overlapping_intervals(vec![(1, 1), (3, 3), (5, 7)]),
        vec![(1, 1), (3, 3), (5, 7)],
    );
}
