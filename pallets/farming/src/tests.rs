use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;

#[test]
fn pool_create_work() {
    new_test_ext().execute_with(|| {
        // 1, create pool already exists
        assert_noop!(
            Farming::create(Origin::root(), STAKE_TOKEN, REWARD_TOKEN, 50,),
            Error::<Test>::PoolAlreadyExists,
        );

        // 2, create pool with a invalid lock duration
        assert_noop!(
            Farming::create(Origin::root(), EHKO, REWARD_TOKEN, 60000,),
            Error::<Test>::ExcessMaxLockDuration,
        );
    })
}

#[test]
fn pool_status_work() {
    new_test_ext().execute_with(|| {
        // 1, deposit when status is active
        assert_ok!(Farming::deposit(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            100_000_000,
        ));

        assert_ok!(Farming::set_pool_status(
            Origin::root(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            false,
        ));

        // 2, deposit when status is not active
        assert_noop!(
            Farming::deposit(
                RawOrigin::Signed(BOB).into(),
                STAKE_TOKEN,
                REWARD_TOKEN,
                100_000_000,
            ),
            Error::<Test>::PoolIsNotActive,
        );

        // 3, can not set status for a pool which not exists
        assert_noop!(
            Farming::set_pool_status(Origin::root(), EHKO, REWARD_TOKEN, false,),
            Error::<Test>::PoolDoesNotExist,
        );

        // 4, can not set status with current status
        assert_noop!(
            Farming::set_pool_status(Origin::root(), STAKE_TOKEN, REWARD_TOKEN, false,),
            Error::<Test>::PoolNewActiveStatusWrong,
        );
    })
}

#[test]
fn pool_lock_duration_work() {
    new_test_ext().execute_with(|| {
        // 1, can not set lock duration for a pool which not exists
        assert_noop!(
            Farming::set_pool_lock_duration(Origin::root(), EHKO, REWARD_TOKEN, 60,),
            Error::<Test>::PoolDoesNotExist,
        );

        // 2, can not set lock duration with current lock duration
        assert_noop!(
            Farming::set_pool_lock_duration(Origin::root(), STAKE_TOKEN, REWARD_TOKEN, 100,),
            Error::<Test>::PoolIsInTargetLockDuration,
        );

        // 3, can not set lock duration with a invalid lock duration
        assert_noop!(
            Farming::set_pool_lock_duration(Origin::root(), STAKE_TOKEN, REWARD_TOKEN, 60000,),
            Error::<Test>::ExcessMaxLockDuration,
        );

        assert_ok!(Farming::deposit(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            100_000_000,
        ));

        assert_ok!(Farming::deposit(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            100_000_000,
        ));

        assert_eq!(
            <Test as Config>::Assets::balance(STAKE_TOKEN, &BOB),
            400_000_000
        );

        // 4,withdraw when lock_duration = 50 and then check balance
        assert_ok!(Farming::withdraw(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            100_000_000,
        ));

        assert_eq!(
            <Test as Config>::Assets::balance(STAKE_TOKEN, &BOB),
            400_000_000
        );

        assert_ok!(Farming::set_pool_lock_duration(
            Origin::root(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            0,
        ));

        assert_ok!(Farming::deposit(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            100_000_000,
        ));

        assert_eq!(
            <Test as Config>::Assets::balance(STAKE_TOKEN, &BOB),
            300_000_000
        );

        // 5,withdraw when lock_duration = 0 and then check balance
        assert_ok!(Farming::withdraw(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            100_000_000,
        ));

        assert_eq!(
            <Test as Config>::Assets::balance(STAKE_TOKEN, &BOB),
            400_000_000
        );

        assert_ok!(Farming::redeem(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
        ));

        assert_eq!(
            <Test as Config>::Assets::balance(STAKE_TOKEN, &BOB),
            500_000_000
        );
    })
}

#[test]
fn pool_deposit_work() {
    new_test_ext().execute_with(|| {
        // 1, can not deposit to a pool which is not exists
        assert_noop!(
            Farming::deposit(
                RawOrigin::Signed(ALICE).into(),
                EHKO,
                REWARD_TOKEN,
                100_000_000,
            ),
            Error::<Test>::PoolDoesNotExist,
        );

        // 2, can not deposit 0
        assert_noop!(
            Farming::deposit(
                RawOrigin::Signed(ALICE).into(),
                STAKE_TOKEN,
                REWARD_TOKEN,
                0,
            ),
            Error::<Test>::NotAValidAmount,
        );
    })
}

#[test]
fn pool_withdraw_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Farming::deposit(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            100_000_000,
        ));

        let user_position = Farming::positions((STAKE_TOKEN, REWARD_TOKEN, ALICE));
        assert_eq!(user_position.deposit_balance, 100_000_000);

        // 1, can not withdraw from a pool which is not exists
        assert_noop!(
            Farming::withdraw(
                RawOrigin::Signed(ALICE).into(),
                EHKO,
                REWARD_TOKEN,
                100_000_000,
            ),
            Error::<Test>::PoolDoesNotExist,
        );

        // 2, can not withdraw 0
        assert_noop!(
            Farming::withdraw(
                RawOrigin::Signed(ALICE).into(),
                STAKE_TOKEN,
                REWARD_TOKEN,
                0,
            ),
            Error::<Test>::NotAValidAmount,
        );

        // 3, can not withdraw more than deposit.
        assert_noop!(
            Farming::withdraw(
                RawOrigin::Signed(ALICE).into(),
                STAKE_TOKEN,
                REWARD_TOKEN,
                200_000_000,
            ),
            ArithmeticError::Overflow,
        );

        assert_ok!(Farming::withdraw(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            10_000_000,
        ));
        let user_position = Farming::positions((STAKE_TOKEN, REWARD_TOKEN, ALICE));
        assert_eq!(user_position.deposit_balance, 90_000_000);

        assert_ok!(Farming::withdraw(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            10_000_000,
        ));
        let user_position = Farming::positions((STAKE_TOKEN, REWARD_TOKEN, ALICE));
        assert_eq!(user_position.deposit_balance, 80_000_000);

        assert_ok!(Farming::withdraw(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            10_000_000,
        ));
        let user_position = Farming::positions((STAKE_TOKEN, REWARD_TOKEN, ALICE));
        assert_eq!(user_position.deposit_balance, 70_000_000);

        // 4, withdraw excess user max lock item count
        assert_noop!(
            Farming::withdraw(
                RawOrigin::Signed(ALICE).into(),
                STAKE_TOKEN,
                REWARD_TOKEN,
                10_000_000,
            ),
            Error::<Test>::ExcessMaxUserLockItemsCount,
        );
        let user_position = Farming::positions((STAKE_TOKEN, REWARD_TOKEN, ALICE));
        assert_eq!(user_position.deposit_balance, 70_000_000);
    })
}

#[test]
fn pool_redeem_work() {
    new_test_ext().execute_with(|| {
        // 1, can not redeem from a pool which is not exists
        assert_noop!(
            Farming::redeem(RawOrigin::Signed(ALICE).into(), EHKO, REWARD_TOKEN,),
            Error::<Test>::PoolDoesNotExist,
        );

        assert_ok!(Farming::deposit(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            100_000_000,
        ));
        assert_eq!(
            <Test as Config>::Assets::balance(STAKE_TOKEN, &BOB),
            400_000_000
        );

        run_to_block(10);
        assert_ok!(Farming::withdraw(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            20_000_000,
        ));
        assert_ok!(Farming::withdraw(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            20_000_000,
        ));
        assert_ok!(Farming::withdraw(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            20_000_000,
        ));

        assert_ok!(Farming::redeem(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
        ));

        // 2, redeem under lock height should not work.
        run_to_block(30);
        let user_position = Farming::positions((STAKE_TOKEN, REWARD_TOKEN, BOB));
        assert_eq!(user_position.lock_balance_items.len(), 3);
        assert_eq!(
            <Test as Config>::Assets::balance(STAKE_TOKEN, &BOB),
            400_000_000
        );

        run_to_block(110);
        assert_ok!(Farming::redeem(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
        ));

        // 3, redeem under lock height should work.
        let user_position = Farming::positions((STAKE_TOKEN, REWARD_TOKEN, BOB));
        assert_eq!(user_position.lock_balance_items.len(), 0);
        assert_eq!(
            <Test as Config>::Assets::balance(STAKE_TOKEN, &BOB),
            460_000_000
        );
    })
}

#[test]
fn pool_dispatch_work() {
    new_test_ext().execute_with(|| {
        // 1, can not dispatch reward for a pool which is not exists
        assert_noop!(
            Farming::dispatch_reward(
                Origin::root(),
                EHKO,
                REWARD_TOKEN,
                REWARD_TOKEN_PAYER,
                1_000_000_000_000_000,
                100,
            ),
            Error::<Test>::PoolDoesNotExist,
        );

        // 2, can not dispatch reward for zero block
        assert_noop!(
            Farming::dispatch_reward(
                Origin::root(),
                STAKE_TOKEN,
                REWARD_TOKEN,
                REWARD_TOKEN_PAYER,
                1_000_000_000_000_000,
                0,
            ),
            Error::<Test>::NotAValidDuration,
        );

        run_to_block(10);
        assert_ok!(Farming::dispatch_reward(
            Origin::root(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            REWARD_TOKEN_PAYER,
            1_000_000_000_000_000,
            100,
        ));

        assert_eq!(
            <Test as Config>::Assets::balance(REWARD_TOKEN, &REWARD_TOKEN_PAYER),
            2_000_000_000_000_000
        );
        let pool_info = Farming::pools(STAKE_TOKEN, REWARD_TOKEN).unwrap();
        assert_eq!(pool_info.duration, 100);
        assert_eq!(pool_info.period_finish, 110);
        assert_eq!(pool_info.last_update_block, 10);
        assert_eq!(pool_info.reward_rate, 10_000_000_000_000);

        run_to_block(60);
        assert_ok!(Farming::dispatch_reward(
            Origin::root(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            REWARD_TOKEN_PAYER,
            0,
            100,
        ));

        assert_eq!(
            <Test as Config>::Assets::balance(REWARD_TOKEN, &REWARD_TOKEN_PAYER),
            2_000_000_000_000_000
        );
        let pool_info = Farming::pools(STAKE_TOKEN, REWARD_TOKEN).unwrap();
        assert_eq!(pool_info.duration, 100);
        assert_eq!(pool_info.period_finish, 160);
        assert_eq!(pool_info.last_update_block, 60);
        assert_eq!(pool_info.reward_rate, 5_000_000_000_000);
    })
}

#[test]
fn pool_claim_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Farming::deposit(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            100_000_000,
        ));

        // 1, can not claim from a pool which is not exists
        assert_noop!(
            Farming::claim(RawOrigin::Signed(ALICE).into(), EHKO, REWARD_TOKEN,),
            Error::<Test>::PoolDoesNotExist,
        );

        // 2, can claim 0 reward from pool
        assert_ok!(Farming::claim(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
        ));
        assert_eq!(<Test as Config>::Assets::balance(REWARD_TOKEN, &ALICE), 0);

        run_to_block(10);
        assert_ok!(Farming::dispatch_reward(
            Origin::root(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            REWARD_TOKEN_PAYER,
            1_000_000_000_000_000,
            100,
        ));

        run_to_block(60);
        assert_ok!(Farming::claim(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
        ));
        assert_eq!(
            <Test as Config>::Assets::balance(REWARD_TOKEN, &ALICE),
            500_000_000_000_000
        );
    })
}

#[test]
fn pool_claim_precision_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Farming::deposit(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            100_000_000,
        ));

        assert_ok!(Farming::deposit(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            200_000_000,
        ));

        run_to_block(10);
        assert_ok!(Farming::dispatch_reward(
            Origin::root(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            REWARD_TOKEN_PAYER,
            1_000_000_000_000_000,
            100,
        ));
        run_to_block(20);
        assert_ok!(Farming::claim(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
        ));
        assert_eq!(
            <Test as Config>::Assets::balance(REWARD_TOKEN, &ALICE),
            33_333_333_333_333
        );
    })
}

#[test]
fn pool_complicated_scene0_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Farming::deposit(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            100_000_000,
        ));

        run_to_block(10);
        assert_ok!(Farming::dispatch_reward(
            Origin::root(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            REWARD_TOKEN_PAYER,
            1_000_000_000_000_000,
            100,
        ));

        run_to_block(20);
        assert_ok!(Farming::deposit(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            400_000_000,
        ));

        run_to_block(30);
        assert_ok!(Farming::claim(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
        ));
        assert_eq!(
            <Test as Config>::Assets::balance(REWARD_TOKEN, &ALICE),
            120_000_000_000_000
        );

        run_to_block(40);
        assert_ok!(Farming::withdraw(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            100_000_000,
        ));

        run_to_block(50);
        assert_ok!(Farming::claim(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
        ));
        assert_eq!(
            <Test as Config>::Assets::balance(REWARD_TOKEN, &ALICE),
            165_000_000_000_000
        ); //120+45

        run_to_block(60);
        assert_ok!(Farming::withdraw(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            300_000_000,
        ));

        run_to_block(110);
        assert_ok!(Farming::claim(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
        ));
        assert_eq!(
            <Test as Config>::Assets::balance(REWARD_TOKEN, &ALICE),
            690_000_000_000_000
        ); //165+525

        run_to_block(140);
        assert_ok!(Farming::dispatch_reward(
            Origin::root(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            REWARD_TOKEN_PAYER,
            1_000_000_000_000_000,
            100,
        ));
        assert_ok!(Farming::redeem(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
        ));
        assert_eq!(
            <Test as Config>::Assets::balance(STAKE_TOKEN, &BOB),
            200_000_000
        ); //500-400+100

        run_to_block(160);
        assert_ok!(Farming::redeem(
            RawOrigin::Signed(BOB).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
        ));
        assert_eq!(
            <Test as Config>::Assets::balance(STAKE_TOKEN, &BOB),
            500_000_000
        ); //500-400+100+300

        run_to_block(190);
        assert_ok!(Farming::dispatch_reward(
            Origin::root(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            REWARD_TOKEN_PAYER,
            0,
            100,
        ));
        assert_ok!(Farming::claim(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
        ));
        assert_eq!(
            <Test as Config>::Assets::balance(REWARD_TOKEN, &ALICE),
            1_190_000_000_000_000
        ); //690+500

        run_to_block(290);
        assert_ok!(Farming::claim(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
        ));
        assert_eq!(
            <Test as Config>::Assets::balance(REWARD_TOKEN, &ALICE),
            1_690_000_000_000_000
        ); //1190+500

        run_to_block(300);
        assert_ok!(Farming::claim(
            RawOrigin::Signed(ALICE).into(),
            STAKE_TOKEN,
            REWARD_TOKEN,
        ));
        assert_eq!(
            <Test as Config>::Assets::balance(REWARD_TOKEN, &ALICE),
            1_690_000_000_000_000
        ); //1690+0
    })
}
