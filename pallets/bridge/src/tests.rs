#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use primitives::tokens::HKO;

#[test]
fn change_bridge_members_works() {
    new_test_ext().execute_with(|| {
        // Get members count works
        assert_eq!(Bridge::get_members_count(), 3);

        // After remove and swap, members count should be 2
        BridgeMembership::remove_member(Origin::root(), ALICE).unwrap();
        BridgeMembership::swap_member(Origin::root(), BOB, DAVE).unwrap();
        assert_eq!(Bridge::get_members_count(), 2);

        // Current members: [CHARLIE , DAVE]
        assert_ok!(Bridge::set_vote_threshold(Origin::signed(CHARLIE), 2,));
        assert_ok!(Bridge::set_vote_threshold(Origin::signed(DAVE), 2,));
        assert_noop!(
            Bridge::set_vote_threshold(Origin::signed(ALICE), 3),
            Error::<Test>::OriginNoPermission,
        );
        assert_noop!(
            Bridge::set_vote_threshold(Origin::signed(BOB), 3),
            Error::<Test>::OriginNoPermission,
        );
    });
}

#[test]
fn set_relayer_threshold_works() {
    new_test_ext().execute_with(|| {
        // General Account cannot set threshold
        assert_noop!(
            Bridge::set_vote_threshold(Origin::signed(FERDIE), 3),
            Error::<Test>::OriginNoPermission,
        );

        // RootOrigin can set threshold
        // [ZeroAccount]
        assert_noop!(
            Bridge::set_vote_threshold(Origin::signed(0u128), 0),
            Error::<Test>::InvalidVoteThreshold,
        );

        // BridgeMembers can set threshold
        // [ALICE, BOB, CHARLIE]
        assert_ok!(Bridge::set_vote_threshold(Origin::signed(ALICE), 3,));
        assert_ok!(Bridge::set_vote_threshold(Origin::signed(BOB), 3,));
        // When the count of members is 3, the threshold should be less than or equal to 3
        assert_noop!(
            Bridge::set_vote_threshold(Origin::signed(CHARLIE), 4),
            Error::<Test>::InvalidVoteThreshold,
        );
    });
}

#[test]
fn register_unregister_works() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Bridge::register_chain(Origin::signed(ALICE), ETH),
            Error::<Test>::ChainIdAlreadyRegistered,
        );

        // Register a new chain_id succeed
        Bridge::register_chain(Origin::signed(ALICE), BNB).unwrap();
        assert_noop!(
            Bridge::register_chain(Origin::signed(ALICE), BNB),
            Error::<Test>::ChainIdAlreadyRegistered,
        );
        // Teleport succeed when the chain is registered
        Bridge::teleport(Origin::signed(EVE), BNB, EHKO, "TELE".into(), dollar(10)).unwrap();

        // Unregister a exist chain_id succeed
        Bridge::unregister_chain(Origin::signed(ALICE), ETH).unwrap();
        assert_noop!(
            Bridge::unregister_chain(Origin::signed(ALICE), ETH),
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
