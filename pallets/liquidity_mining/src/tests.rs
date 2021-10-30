use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use std::convert::TryInto;

#[test]
fn create_pool_should_work() {
    new_test_ext().execute_with(|| {
        let initial_balance = Assets::balance(DOT, BOB);

        assert_eq!(Assets::balance(DOT, BOB), initial_balance);
        assert_ok!(LiquidityMining::create(
            RawOrigin::Signed(ALICE).into(),
            DOT,
            BOB,
            0,
            3,
            vec![1; 1000].try_into().unwrap(),
            vec![DOT; 1000].try_into().unwrap(),
            SAMPLE_LP_TOKEN,
        ));

        assert!(Pools::<Test>::contains_key(DOT));

        assert_eq!(Assets::balance(DOT, BOB), initial_balance - 3000);
    })
}

#[test]
fn create_pool_should_not_work_if_block_and_rewards_not_same_size() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LiquidityMining::create(
                RawOrigin::Signed(ALICE).into(),
                DOT,
                BOB,
                0,
                3,
                vec![1; 999].try_into().unwrap(),
                vec![DOT; 1000].try_into().unwrap(),
                SAMPLE_LP_TOKEN,
            ),
            Error::<Test>::PerBlockAndRewardsAreNotSameSize
        );
    })
}

#[test]
fn create_pool_should_not_work_if_pool_already_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidityMining::create(
            RawOrigin::Signed(ALICE).into(),
            DOT,
            BOB,
            0,
            3,
            vec![1; 1000].try_into().unwrap(),
            vec![DOT; 1000].try_into().unwrap(),
            SAMPLE_LP_TOKEN,
        ));

        assert_noop!(
            LiquidityMining::create(
                RawOrigin::Signed(ALICE).into(),
                DOT,
                BOB,
                0,
                3,
                vec![1; 1000].try_into().unwrap(),
                vec![DOT; 1000].try_into().unwrap(),
                SAMPLE_LP_TOKEN,
            ),
            Error::<Test>::PoolAlreadyExists
        );
    })
}

#[test]
fn create_pool_should_not_work_if_not_a_newly_asset() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LiquidityMining::create(
                RawOrigin::Signed(ALICE).into(),
                DOT,
                BOB,
                0,
                3,
                vec![1; 1000].try_into().unwrap(),
                vec![DOT; 1000].try_into().unwrap(),
                DOT,
            ),
            Error::<Test>::NotANewlyCreatedAsset
        );
    })
}
