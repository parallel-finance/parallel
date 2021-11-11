#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn set_relayer_threshold_works() {
    new_test_ext().execute_with(|| {
        // Get members count works
        assert_eq!(Bridge::get_members_count(), 3);

        assert_noop!(
            Bridge::set_threshold(Origin::signed(FERDIE), 3),
            Error::<Test>::OriginNoPermission,
        );
        // RootOrigin can set threshold
        assert_noop!(
            Bridge::set_threshold(Origin::signed(0u128), 0),
            Error::<Test>::InvalidVoteThreshold,
        );
        assert_ok!(Bridge::set_threshold(Origin::signed(ALICE), 3,));

        BridgeMembership::remove_member(Origin::root(), ALICE).unwrap();
        BridgeMembership::swap_member(Origin::root(), BOB, DAVE).unwrap();
        // After remove and swap, members count should be 2
        // Current members: [CHARLIE , DAVE]
        assert_eq!(Bridge::get_members_count(), 2);
        assert_noop!(
            Bridge::set_threshold(Origin::signed(ALICE), 3),
            Error::<Test>::OriginNoPermission,
        );
        assert_noop!(
            Bridge::set_threshold(Origin::signed(BOB), 3),
            Error::<Test>::OriginNoPermission,
        );
        // When the count of members is 2, the threshold should be less than or equal to 2
        assert_noop!(
            Bridge::set_threshold(Origin::signed(CHARLIE), 3),
            Error::<Test>::InvalidVoteThreshold,
        );
        assert_ok!(Bridge::set_threshold(Origin::signed(DAVE), 2,));
    });
}
