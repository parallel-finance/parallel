use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;

#[test]
fn stake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(1.into()), 10));

        // Check storage is correct
        assert_eq!(
            ExchangeRate::<Test>::get(),
            Rate::saturating_from_rational(2, 100)
        );
        assert_eq!(TotalStakingAsset::<Test>::get(), 10);
        assert_eq!(TotalVoucher::<Test>::get(), 500);

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            90
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            500
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            10
        );
    })
}

#[test]
fn withdraw_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        assert_ok!(LiquidStaking::withdraw(
            Origin::signed(6.into()),
            2.into(),
            10
        ));

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &AccountId::from(2_u64)),
            10
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &AccountId::from(1_u64)),
            90
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            0
        );
    })
}

#[test]
fn withdraw_from_invalid_origin_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        assert_noop!(
            LiquidStaking::withdraw(Origin::signed(1.into()), 2.into(), 11),
            BadOrigin,
        );
    })
}

#[test]
fn withdraw_too_much_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        assert_noop!(
            LiquidStaking::withdraw(Origin::signed(6.into()), 2.into(), 11),
            Error::<Test>::ExcessWithdrawThreshold,
        );
    })
}

#[test]
fn record_rewards_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        assert_ok!(LiquidStaking::record_rewards(
            Origin::signed(6.into()),
            2.into(),
            10
        ));

        // Check storage is correct
        assert_eq!(
            ExchangeRate::<Test>::get(),
            Rate::saturating_from_rational(4, 100)
        );
        assert_eq!(TotalStakingAsset::<Test>::get(), 20);
        assert_eq!(TotalVoucher::<Test>::get(), 500);
    })
}

#[test]
fn record_rewards_from_invalid_origin_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        assert_noop!(
            LiquidStaking::record_rewards(Origin::signed(1.into()), 2.into(), 10),
            BadOrigin,
        );
    })
}

#[test]
fn record_slash_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        assert_ok!(LiquidStaking::record_slash(
            Origin::signed(6.into()),
            2.into(),
            5
        ));

        // Check storage is correct
        assert_eq!(
            ExchangeRate::<Test>::get(),
            Rate::saturating_from_rational(1, 100)
        );
        assert_eq!(TotalStakingAsset::<Test>::get(), 5);
        assert_eq!(TotalVoucher::<Test>::get(), 500);
    })
}

#[test]
fn record_slash_from_invalid_origin_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        assert_noop!(
            LiquidStaking::record_slash(Origin::signed(1.into()), 2.into(), 5),
            BadOrigin,
        );
    })
}

#[test]
fn unstake_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        assert_ok!(LiquidStaking::unstake(Origin::signed(1.into()), 500));

        // Check storage is correct
        assert_eq!(
            ExchangeRate::<Test>::get(),
            Rate::saturating_from_rational(2, 100)
        );
        assert_eq!(TotalStakingAsset::<Test>::get(), 0);
        assert_eq!(TotalVoucher::<Test>::get(), 0);
        assert_eq!(
            AccountPendingUnstake::<Test>::get(&AccountId::from(1_u64)).unwrap(),
            UnstakeInfo {
                amount: 10,
                block_number: frame_system::Pallet::<Test>::block_number()
            }
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            90
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            0
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            10
        );
    })
}

#[test]
fn unstake_amount_should_not_exceed_balance() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        assert_noop!(
            LiquidStaking::unstake(Origin::signed(1.into()), 501),
            orml_tokens::Error::<Test>::BalanceTooLow,
        );
    })
}

#[test]
fn process_pending_unstake_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 500);

        assert_ok!(LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            10
        ));

        // Check storage is correct
        assert_eq!(
            AccountPendingUnstake::<Test>::get(&AccountId::from(1_u64)),
            None,
        );
        let processing_unstake =
            AccountProcessingUnstake::<Test>::get(&AccountId::from(10000_u64), &AccountId::from(1))
                .unwrap();
        assert_eq!(processing_unstake.len(), 1);
        assert_eq!(processing_unstake[0].amount, 10);
        assert_eq!(
            processing_unstake[0].block_number,
            frame_system::Pallet::<Test>::block_number()
        );
    })
}

#[test]
fn process_pending_unstake_from_invalid_origin_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 500);

        assert_noop!(
            LiquidStaking::process_pending_unstake(
                Origin::signed(1.into()),
                10000.into(),
                1.into(),
                10
            ),
            BadOrigin
        );
    })
}

#[test]
fn process_pending_unstake_with_empty_unstake_request_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);

        assert_noop!(
            LiquidStaking::process_pending_unstake(
                Origin::signed(6.into()),
                10000.into(),
                1.into(),
                10
            ),
            Error::<Test>::NoPendingUnstake
        );
    })
}

#[test]
fn process_pending_unstake_with_excess_amount_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 500);

        assert_noop!(
            LiquidStaking::process_pending_unstake(
                Origin::signed(6.into()),
                10000.into(),
                1.into(),
                20,
            ),
            Error::<Test>::InvalidUnstakeAmount
        );
    })
}

#[test]
fn process_pending_unstake_for_multiple_times_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 500);

        // The first time
        assert_ok!(LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            5
        ));

        // Check storage is correct
        assert_eq!(
            AccountPendingUnstake::<Test>::get(&AccountId::from(1_u64)),
            Some(UnstakeInfo {
                amount: 5,
                block_number: frame_system::Pallet::<Test>::block_number(),
            }),
        );
        let processing_unstake = AccountProcessingUnstake::<Test>::get(
            &AccountId::from(10000_u64),
            &AccountId::from(1_u64),
        )
        .unwrap();
        assert_eq!(processing_unstake.len(), 1);
        assert_eq!(processing_unstake[0].amount, 5);
        assert_eq!(
            processing_unstake[0].block_number,
            frame_system::Pallet::<Test>::block_number()
        );

        // The second time
        assert_ok!(LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            4
        ));

        // Check storage is correct
        assert_eq!(
            AccountPendingUnstake::<Test>::get(&AccountId::from(1_u64)),
            Some(UnstakeInfo {
                amount: 1,
                block_number: frame_system::Pallet::<Test>::block_number(),
            }),
        );
        let processing_unstake = AccountProcessingUnstake::<Test>::get(
            &AccountId::from(10000_u64),
            &AccountId::from(1_u64),
        )
        .unwrap();
        assert_eq!(processing_unstake.len(), 2);
        assert_eq!(processing_unstake[0].amount, 5);
        assert_eq!(
            processing_unstake[0].block_number,
            frame_system::Pallet::<Test>::block_number()
        );
        assert_eq!(processing_unstake[1].amount, 4);
        assert_eq!(
            processing_unstake[1].block_number,
            frame_system::Pallet::<Test>::block_number()
        );

        // The third time
        assert_ok!(LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            1
        ));

        // Check storage is correct
        assert_eq!(
            AccountPendingUnstake::<Test>::get(&AccountId::from(1_u64)),
            None,
        );
        let processing_unstake = AccountProcessingUnstake::<Test>::get(
            &AccountId::from(10000_u64),
            &AccountId::from(1_u64),
        )
        .unwrap();
        assert_eq!(processing_unstake.len(), 3);
        assert_eq!(processing_unstake[0].amount, 5);
        assert_eq!(
            processing_unstake[0].block_number,
            frame_system::Pallet::<Test>::block_number()
        );
        assert_eq!(processing_unstake[1].amount, 4);
        assert_eq!(
            processing_unstake[1].block_number,
            frame_system::Pallet::<Test>::block_number()
        );
        assert_eq!(processing_unstake[2].amount, 1);
        assert_eq!(
            processing_unstake[2].block_number,
            frame_system::Pallet::<Test>::block_number()
        );
    })
}

#[test]
fn finish_processed_unstake_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 500);
        let _ = LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            10,
        );

        assert_ok!(LiquidStaking::finish_processed_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            10
        ));

        // Check storage is correct
        assert_eq!(
            AccountProcessingUnstake::<Test>::get(
                &AccountId::from(10000_u64),
                &AccountId::from(1_u64)
            ),
            None,
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            100
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            0
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            0
        );
    })
}

#[test]
fn finish_processed_unstake_from_invalid_origin_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 500);
        let _ = LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            10,
        );

        assert_noop!(
            LiquidStaking::finish_processed_unstake(
                Origin::signed(1.into()),
                10000.into(),
                1.into(),
                10
            ),
            BadOrigin
        );
    })
}

#[test]
fn finish_processed_unstake_without_processing_first_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 500);

        assert_noop!(
            LiquidStaking::finish_processed_unstake(
                Origin::signed(6.into()),
                10000.into(),
                1.into(),
                10
            ),
            Error::<Test>::NoProcessingUnstake
        );
    })
}

#[test]
fn finish_processed_unstake_with_incorrect_amount_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 500);
        let _ = LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            10,
        );

        assert_noop!(
            LiquidStaking::finish_processed_unstake(
                Origin::signed(6.into()),
                10000.into(),
                1.into(),
                8,
            ),
            Error::<Test>::InvalidProcessedUnstakeAmount
        );
    })
}

#[test]
fn finish_processed_unstake_with_another_incorrect_amount_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 500);
        let _ = LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            10,
        );

        assert_noop!(
            LiquidStaking::finish_processed_unstake(
                Origin::signed(6.into()),
                10000.into(),
                1.into(),
                11,
            ),
            Error::<Test>::InvalidProcessedUnstakeAmount
        );
    })
}

#[test]
fn finish_processed_unstake_with_multiple_processing_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 500);
        let _ = LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            5,
        );
        let _ = LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            4,
        );
        let _ = LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            1,
        );

        // The first time
        assert_ok!(LiquidStaking::finish_processed_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            5,
        ));

        // Check storage is correct
        assert_eq!(
            AccountProcessingUnstake::<Test>::get(
                &AccountId::from(10000_u64),
                &AccountId::from(1_u64)
            )
            .unwrap(),
            vec![
                UnstakeInfo {
                    amount: 4,
                    block_number: frame_system::Pallet::<Test>::block_number()
                },
                UnstakeInfo {
                    amount: 1,
                    block_number: frame_system::Pallet::<Test>::block_number()
                },
            ],
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            95
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            0
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            5
        );

        // The second time
        assert_ok!(LiquidStaking::finish_processed_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            4,
        ));

        // Check storage is correct
        assert_eq!(
            AccountProcessingUnstake::<Test>::get(
                &AccountId::from(10000_u64),
                &AccountId::from(1_u64)
            )
            .unwrap(),
            vec![UnstakeInfo {
                amount: 1,
                block_number: frame_system::Pallet::<Test>::block_number()
            },],
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            99
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            0
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            1
        );

        // The third time
        assert_ok!(LiquidStaking::finish_processed_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            1,
        ));

        // Check storage is correct
        assert_eq!(
            AccountProcessingUnstake::<Test>::get(
                &AccountId::from(10000_u64),
                &AccountId::from(1_u64)
            ),
            None,
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            100
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            0
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            0
        );
    })
}

#[test]
fn record_rewards_deduct_reserve_should_work() {
    new_test_ext().execute_with(|| {
        let stake_amount = 10 * DOT_DECIMAL;
        assert_ok!(LiquidStaking::stake(
            Origin::signed(11.into()),
            stake_amount
        ));

        let reward_amount = 1 * DOT_DECIMAL;
        assert_ok!(LiquidStaking::record_rewards(
            Origin::signed(6.into()),
            2.into(),
            reward_amount
        ));

        let reserve_factor = ReserveFactor::<Test>::get();
        let total_staking = stake_amount + reward_amount - reserve_factor.mul_floor(reward_amount);
        assert_eq!(TotalStakingAsset::<Test>::get(), total_staking);
        let total_voucher = 500 * DOT_DECIMAL;
        assert_eq!(TotalVoucher::<Test>::get(), total_voucher);
        assert_eq!(TotalReserve::<Test>::get(), 5 * 10u128.pow(7));
        assert_eq!(
            ExchangeRate::<Test>::get(),
            Rate::saturating_from_rational(total_staking, total_voucher)
        );
    })
}

#[test]
fn process_pending_unstake_for_max_should_fail() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(1.into()), 10));
        assert_ok!(LiquidStaking::unstake(Origin::signed(1.into()), 500));
        let max = <mock::Test as Config>::MaxAccountProcessingUnstake::get() as u32;
        // in production, MaxAccountProcessingUnstake should be suitable
        assert_eq!(max, 5);

        for _i in 0..max {
            assert_ok!(LiquidStaking::process_pending_unstake(
                Origin::signed(6.into()),
                10000.into(),
                1.into(),
                1
            ));
        }

        assert_noop!(
            LiquidStaking::process_pending_unstake(
                Origin::signed(6.into()),
                10000.into(),
                1.into(),
                1
            ),
            Error::<Test>::MaxAccountProcessingUnstakeExceeded,
        );
    })
}
