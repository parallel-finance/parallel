use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use primitives::tokens;
use std::convert::TryInto;

#[test]
fn create_pool_should_work() {
    new_test_ext().execute_with(|| {
        let initial_balance = Assets::balance(DOT, BOB);

        assert_eq!(Assets::balance(DOT, BOB), initial_balance);
        assert_ok!(Farming::create(
            RawOrigin::Signed(ALICE).into(),
            DOT,
            BOB,
            0,
            3,
            vec![(1, DOT); 1000].try_into().unwrap(),
            SAMPLE_LP_TOKEN,
        ));

        assert!(Pools::<Test>::contains_key(DOT));

        assert_eq!(Assets::balance(DOT, BOB), initial_balance - 3000);
    })
}

#[test]
fn create_pool_should_not_work_if_pool_already_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(Farming::create(
            RawOrigin::Signed(ALICE).into(),
            DOT,
            BOB,
            0,
            3,
            vec![(1, DOT); 1000].try_into().unwrap(),
            SAMPLE_LP_TOKEN,
        ));

        assert_noop!(
            Farming::create(
                RawOrigin::Signed(ALICE).into(),
                DOT,
                BOB,
                0,
                3,
                vec![(1, DOT); 1000].try_into().unwrap(),
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
            Farming::create(
                RawOrigin::Signed(ALICE).into(),
                DOT,
                BOB,
                0,
                3,
                vec![(1, DOT); 1000].try_into().unwrap(),
                DOT,
            ),
            Error::<Test>::NotANewlyCreatedAsset
        );
    })
}

#[test]
fn create_pool_should_not_work_if_endblock_smaller_than_startblock() {
    new_test_ext().execute_with(|| {
        let (start_block, end_block) = (6, 2);

        assert_noop!(
            Farming::create(
                RawOrigin::Signed(ALICE).into(),
                DOT,
                BOB,
                start_block,
                end_block,
                vec![(1, DOT); 1000].try_into().unwrap(),
                DOT,
            ),
            Error::<Test>::SmallerThanEndBlock
        );
    })
}

#[test]
fn deposit_should_work() {
    new_test_ext().execute_with(|| {
        let initial_balance = Assets::balance(DOT, BOB);

        assert_eq!(Assets::balance(DOT, BOB), initial_balance);
        assert_ok!(Farming::create(
            RawOrigin::Signed(ALICE).into(),
            DOT,
            BOB,
            0,
            3,
            vec![(1, DOT); 1000].try_into().unwrap(),
            SAMPLE_LP_TOKEN,
        ));

        assert_eq!(Assets::balance(DOT, BOB), initial_balance - 3000);

        assert_ok!(Farming::deposit(RawOrigin::Signed(BOB).into(), DOT, 1000));

        assert_eq!(Assets::balance(DOT, BOB), initial_balance - 3000 - 1000);
    })
}

#[test]
fn deposit_should_not_work_if_amount_is_zero() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Farming::deposit(RawOrigin::Signed(BOB).into(), DOT, 0),
            Error::<Test>::NotAValidAmount
        );
    })
}

#[test]
fn deposit_should_not_work_if_pool_does_not_exist() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Farming::deposit(RawOrigin::Signed(BOB).into(), DOT, 1000),
            Error::<Test>::PoolDoesNotExist
        );
    })
}

#[test]
fn deposit_should_not_work_if_not_a_valid_duration() {
    new_test_ext().execute_with(|| {
        let (start_block, end_block) = (3, 100);
        assert_ok!(Farming::create(
            RawOrigin::Signed(ALICE).into(),
            DOT,
            BOB,
            start_block,
            end_block,
            vec![(1, DOT); 1000].try_into().unwrap(),
            SAMPLE_LP_TOKEN,
        ));

        // current block number is smaller than pool.start
        System::set_block_number(start_block - 1);
        assert_noop!(
            Farming::deposit(RawOrigin::Signed(BOB).into(), DOT, 1000),
            Error::<Test>::NotAValidDuration
        );

        // current block number is bigger than pool.end
        System::set_block_number(end_block + 1);
        assert_noop!(
            Farming::deposit(RawOrigin::Signed(BOB).into(), DOT, 1000),
            Error::<Test>::NotAValidDuration
        );
    })
}

#[test]
fn withdraw_should_work() {
    new_test_ext().execute_with(|| {
        let initial_balance = Assets::balance(DOT, BOB);

        assert_eq!(Assets::balance(DOT, BOB), initial_balance);
        assert_ok!(Farming::create(
            RawOrigin::Signed(ALICE).into(),
            DOT,
            BOB,
            0,
            3,
            vec![(1, DOT); 1000].try_into().unwrap(),
            SAMPLE_LP_TOKEN,
        ));

        assert_eq!(Assets::balance(DOT, BOB), initial_balance - 3000);

        assert_ok!(Farming::deposit(RawOrigin::Signed(BOB).into(), DOT, 1000));

        assert_ok!(Farming::withdraw(RawOrigin::Signed(BOB).into(), DOT, 1000));

        assert_eq!(Assets::balance(DOT, BOB), initial_balance - 3000);
    })
}

#[test]
fn withdraw_should_not_work_if_amount_is_zero() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Farming::withdraw(RawOrigin::Signed(BOB).into(), DOT, 0),
            Error::<Test>::NotAValidAmount
        );
    })
}

#[test]
fn withdraw_should_not_work_if_pool_does_not_exist() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Farming::withdraw(RawOrigin::Signed(BOB).into(), DOT, 1000),
            Error::<Test>::PoolDoesNotExist
        );
    })
}

#[test]
fn create_pool_account_id_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            Farming::pool_account_id(tokens::DOT).unwrap(),
            AccountId(5650623433380315385)
        );
        assert_eq!(
            Farming::pool_account_id(tokens::XDOT).unwrap(),
            AccountId(17971758411142122835)
        );
        assert_eq!(
            Farming::pool_account_id(tokens::PARA).unwrap(),
            AccountId(12297710138430822110)
        );
    })
}
