#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn set_relayer_threshold_works() {
    new_test_ext().execute_with(|| {
        // General Account cannot set threshold
        assert_noop!(
            Bridge::set_threshold(Origin::signed(FERDIE), 3),
            Error::<Test>::OriginNoPermission,
        );

        // RootOrigin can set threshold
        // [ZeroAccount]
        assert_noop!(
            Bridge::set_threshold(Origin::signed(0u128), 0),
            Error::<Test>::InvalidVoteThreshold,
        );

        // BridgeMembers can set threshold
        // [ALICE, BOB, CHARLIE]
        assert_ok!(Bridge::set_threshold(Origin::signed(ALICE), 3,));
        assert_ok!(Bridge::set_threshold(Origin::signed(BOB), 3,));
        // When the count of members is 3, the threshold should be less than or equal to 3
        assert_noop!(
            Bridge::set_threshold(Origin::signed(CHARLIE), 4),
            Error::<Test>::InvalidVoteThreshold,
        );
    });
}

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
        assert_ok!(Bridge::set_threshold(Origin::signed(CHARLIE), 2,));
        assert_ok!(Bridge::set_threshold(Origin::signed(DAVE), 2,));
        assert_noop!(
            Bridge::set_threshold(Origin::signed(ALICE), 3),
            Error::<Test>::OriginNoPermission,
        );
        assert_noop!(
            Bridge::set_threshold(Origin::signed(BOB), 3),
            Error::<Test>::OriginNoPermission,
        );
    });
}
