use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{traits::BadOrigin, FixedU128};

#[test]
fn stake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(
            Origin::signed(1.into()),
            10 * DOT_DECIMAL
        ));
        // Check storage is correct
        assert_eq!(
            ExchangeRate::<Test>::get(),
            Rate::saturating_from_rational(2, 100)
        );
        assert_eq!(TotalStakingAsset::<Test>::get(), 99400500000);
        assert_eq!(TotalVoucher::<Test>::get(), 4970025000000);
        // if users stakes 10 DOT, then we charge 0.05995 DOT for xcm fees & slash insurance
        assert_eq!(TotalReserves::<Test>::get(), 599500000);

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            90 * DOT_DECIMAL
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            4970025000000
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            10 * DOT_DECIMAL
        );

        // check StakingPersonTimes works correctly
        assert_eq!(StakingPersonTimes::<Test>::get(), 1);
        assert_noop!(
            LiquidStaking::stake(Origin::signed(1.into()), 100_000_000),
            Error::<Test>::AmountTooSmallToPayCrossChainFees
        );
        assert_ok!(LiquidStaking::stake(Origin::signed(1.into()), 100_000_001));
        assert_eq!(StakingPersonTimes::<Test>::get(), 2);
        StakingPersonTimes::<Test>::mutate(|b| *b = u128::MAX);
        assert_ok!(LiquidStaking::stake(
            Origin::signed(1.into()),
            10 * DOT_DECIMAL
        ));
        assert_eq!(StakingPersonTimes::<Test>::get(), u128::MAX);
    })
}

#[test]
fn withdraw_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10 * DOT_DECIMAL);
        assert_ok!(LiquidStaking::withdraw(
            Origin::signed(6.into()),
            2.into(),
            99400500000
        ));

        // check storage is correct
        assert_eq!(TotalStakingAsset::<Test>::get(), 99400500000);
        assert_eq!(TotalReserves::<Test>::get(), 499500000);

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &AccountId::from(2_u64)),
            // here,
            99500500000
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &AccountId::from(1_u64)),
            90 * DOT_DECIMAL
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            499500000
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
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10 * DOT_DECIMAL);
        assert_noop!(
            LiquidStaking::withdraw(Origin::signed(6.into()), 2.into(), 10 * DOT_DECIMAL + 1),
            Error::<Test>::ExcessWithdrawThreshold,
        );
    })
}

#[test]
fn record_rewards_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10 * DOT_DECIMAL);
        assert_ok!(LiquidStaking::record_rewards(
            Origin::signed(6.into()),
            2.into(),
            10 * DOT_DECIMAL
        ));

        // Check storage is correct
        assert_eq!(
            ExchangeRate::<Test>::get(),
            FixedU128::from_inner(40120623135698512),
        );
        assert_eq!(TotalStakingAsset::<Test>::get(), 199400500000);
        assert_eq!(TotalVoucher::<Test>::get(), 4970025000000);
    })
}

#[test]
fn record_rewards_from_invalid_origin_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10 * DOT_DECIMAL);
        assert_noop!(
            LiquidStaking::record_rewards(Origin::signed(1.into()), 2.into(), 10 * DOT_DECIMAL),
            BadOrigin,
        );
    })
}

#[test]
fn record_slash_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10 * DOT_DECIMAL);
        assert_eq!(TotalReserves::<Test>::get(), 599500000);
        assert_ok!(LiquidStaking::record_slash(
            Origin::signed(6.into()),
            2.into(),
            599500000
        ));

        // Check storage is correct
        assert_eq!(
            ExchangeRate::<Test>::get(),
            Rate::saturating_from_rational(2, 100)
        );
        assert_eq!(TotalStakingAsset::<Test>::get(), 99400500000);
        assert_eq!(TotalVoucher::<Test>::get(), 4970025000000);

        // Record another slash
        assert_ok!(LiquidStaking::record_slash(
            Origin::signed(6.into()),
            2.into(),
            49700250000
        ));

        // Check storage is correct
        assert_eq!(
            ExchangeRate::<Test>::get(),
            Rate::saturating_from_rational(1, 100)
        );
        assert_eq!(TotalStakingAsset::<Test>::get(), 49700250000);
        assert_eq!(TotalVoucher::<Test>::get(), 4970025000000);
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
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10 * DOT_DECIMAL);
        assert_ok!(LiquidStaking::unstake(
            Origin::signed(1.into()),
            4970025000000
        ));

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
                amount: 99400500000,
                block_number: frame_system::Pallet::<Test>::block_number(),
                era_index: None,
            }
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            90 * DOT_DECIMAL
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            0
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            10 * DOT_DECIMAL
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
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10 * DOT_DECIMAL);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 4970025000000);

        assert_ok!(LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            1,
            99400500000
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
        assert_eq!(processing_unstake[0].amount, 99400500000);
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
                1,
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
                1,
                10
            ),
            Error::<Test>::NoPendingUnstake
        );
    })
}

#[test]
fn process_pending_unstake_with_excess_amount_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10 * DOT_DECIMAL);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 4970025000000);

        assert_noop!(
            LiquidStaking::process_pending_unstake(
                Origin::signed(6.into()),
                10000.into(),
                1.into(),
                1,
                20 * DOT_DECIMAL,
            ),
            Error::<Test>::InvalidUnstakeAmount
        );
    })
}

#[test]
fn process_pending_unstake_for_multiple_times_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10 * DOT_DECIMAL);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 4970025000000);

        // The first time
        assert_ok!(LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            1,
            5 * DOT_DECIMAL
        ));

        // Check storage is correct
        assert_eq!(
            AccountPendingUnstake::<Test>::get(&AccountId::from(1_u64)),
            Some(UnstakeInfo {
                amount: 49400500000,
                block_number: frame_system::Pallet::<Test>::block_number(),
                era_index: None,
            }),
        );
        let processing_unstake = AccountProcessingUnstake::<Test>::get(
            &AccountId::from(10000_u64),
            &AccountId::from(1_u64),
        )
        .unwrap();
        assert_eq!(processing_unstake.len(), 1);
        assert_eq!(processing_unstake[0].amount, 5 * DOT_DECIMAL);
        assert_eq!(
            processing_unstake[0].block_number,
            frame_system::Pallet::<Test>::block_number()
        );
        assert_eq!(processing_unstake[0].era_index, Some(1));

        // The second time
        assert_ok!(LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            2,
            4 * DOT_DECIMAL
        ));

        // Check storage is correct
        assert_eq!(
            AccountPendingUnstake::<Test>::get(&AccountId::from(1_u64)),
            Some(UnstakeInfo {
                amount: 9400500000,
                block_number: frame_system::Pallet::<Test>::block_number(),
                era_index: None,
            }),
        );
        let processing_unstake = AccountProcessingUnstake::<Test>::get(
            &AccountId::from(10000_u64),
            &AccountId::from(1_u64),
        )
        .unwrap();
        assert_eq!(processing_unstake.len(), 2);
        assert_eq!(processing_unstake[0].amount, 5 * DOT_DECIMAL);
        assert_eq!(
            processing_unstake[0].block_number,
            frame_system::Pallet::<Test>::block_number()
        );
        assert_eq!(processing_unstake[0].era_index, Some(1));
        assert_eq!(processing_unstake[1].amount, 4 * DOT_DECIMAL);
        assert_eq!(
            processing_unstake[1].block_number,
            frame_system::Pallet::<Test>::block_number()
        );
        assert_eq!(processing_unstake[1].era_index, Some(2));

        // The third time
        assert_ok!(LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            3,
            9400500000
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
        assert_eq!(processing_unstake[0].amount, 5 * DOT_DECIMAL);
        assert_eq!(
            processing_unstake[0].block_number,
            frame_system::Pallet::<Test>::block_number()
        );
        assert_eq!(processing_unstake[0].era_index, Some(1));
        assert_eq!(processing_unstake[1].amount, 4 * DOT_DECIMAL);
        assert_eq!(
            processing_unstake[1].block_number,
            frame_system::Pallet::<Test>::block_number()
        );
        assert_eq!(processing_unstake[1].era_index, Some(2));
        assert_eq!(processing_unstake[2].amount, 9400500000);

        assert_eq!(
            processing_unstake[2].block_number,
            frame_system::Pallet::<Test>::block_number()
        );
        assert_eq!(processing_unstake[2].era_index, Some(3));
    })
}

#[test]
fn finish_processed_unstake_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10 * DOT_DECIMAL);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 4970025000000);
        let _ = LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            1,
            99400500000,
        );

        assert_ok!(LiquidStaking::finish_processed_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            99400500000
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
            999400500000
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            0
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            599500000
        );
    })
}

#[test]
fn finish_processed_unstake_from_invalid_origin_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10 * DOT_DECIMAL);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 4970025000000);
        let _ = LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            1,
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
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10 * DOT_DECIMAL);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 4970025000000);
        let _ = LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            1,
            99400500000,
        );

        assert_noop!(
            LiquidStaking::finish_processed_unstake(
                Origin::signed(6.into()),
                10000.into(),
                1.into(),
                8 * DOT_DECIMAL,
            ),
            Error::<Test>::InvalidProcessedUnstakeAmount
        );
    })
}

#[test]
fn finish_processed_unstake_with_another_incorrect_amount_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10 * DOT_DECIMAL);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 4970025000000);
        let _ = LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            1,
            99400500000,
        );

        assert_noop!(
            LiquidStaking::finish_processed_unstake(
                Origin::signed(6.into()),
                10000.into(),
                1.into(),
                11 * DOT_DECIMAL,
            ),
            Error::<Test>::InvalidProcessedUnstakeAmount
        );
    })
}

#[test]
fn finish_processed_unstake_with_multiple_processing_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1.into()), 10 * DOT_DECIMAL);
        let _ = LiquidStaking::unstake(Origin::signed(1.into()), 4970025000000);
        let _ = LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            3,
            5 * DOT_DECIMAL,
        );
        let _ = LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            3,
            4 * DOT_DECIMAL,
        );
        let _ = LiquidStaking::process_pending_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            3,
            9400500000,
        );

        // The first time
        assert_ok!(LiquidStaking::finish_processed_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            5 * DOT_DECIMAL,
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
                    amount: 4 * DOT_DECIMAL,
                    block_number: frame_system::Pallet::<Test>::block_number(),
                    era_index: Some(3),
                },
                UnstakeInfo {
                    amount: 9400500000,
                    block_number: frame_system::Pallet::<Test>::block_number(),
                    era_index: Some(3),
                },
            ],
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            950000000000
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            0
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            50000000000
        );

        // The second time
        assert_ok!(LiquidStaking::finish_processed_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            4 * DOT_DECIMAL,
        ));

        // Check storage is correct
        assert_eq!(
            AccountProcessingUnstake::<Test>::get(
                &AccountId::from(10000_u64),
                &AccountId::from(1_u64)
            )
            .unwrap(),
            vec![UnstakeInfo {
                amount: 9400500000,
                block_number: frame_system::Pallet::<Test>::block_number(),
                era_index: Some(3),
            },],
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            99 * DOT_DECIMAL
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            0
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            1 * DOT_DECIMAL
        );

        // The third time
        assert_ok!(LiquidStaking::finish_processed_unstake(
            Origin::signed(6.into()),
            10000.into(),
            1.into(),
            9400500000,
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
            999400500000
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            0
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            599500000
        );
    })
}

#[test]
fn process_pending_unstake_for_max_should_fail() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(
            Origin::signed(1.into()),
            10 * DOT_DECIMAL
        ));
        assert_ok!(LiquidStaking::unstake(
            Origin::signed(1.into()),
            4970025000000
        ));
        let max = <mock::Test as Config>::MaxAccountProcessingUnstake::get() as u32;
        // in production, MaxAccountProcessingUnstake should be suitable
        assert_eq!(max, 5);

        for _i in 0..max {
            assert_ok!(LiquidStaking::process_pending_unstake(
                Origin::signed(6.into()),
                10000.into(),
                1.into(),
                1,
                1
            ));
        }

        assert_noop!(
            LiquidStaking::process_pending_unstake(
                Origin::signed(6.into()),
                10000.into(),
                1.into(),
                1,
                1
            ),
            Error::<Test>::MaxAccountProcessingUnstakeExceeded,
        );
    })
}

#[test]
fn illegal_agent_should_fail() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LiquidStaking::withdraw(Origin::signed(6.into()), 11.into(), 1),
            Error::<Test>::IllegalAgent,
        );

        assert_noop!(
            LiquidStaking::record_rewards(Origin::signed(6.into()), 11.into(), 1),
            Error::<Test>::IllegalAgent,
        );

        assert_noop!(
            LiquidStaking::record_slash(Origin::signed(6.into()), 11.into(), 1),
            Error::<Test>::IllegalAgent,
        );

        assert_noop!(
            LiquidStaking::process_pending_unstake(
                Origin::signed(6.into()),
                11.into(),
                1.into(),
                1,
                1
            ),
            Error::<Test>::IllegalAgent,
        );

        assert_noop!(
            LiquidStaking::finish_processed_unstake(
                Origin::signed(6.into()),
                11.into(),
                1.into(),
                1
            ),
            Error::<Test>::IllegalAgent,
        );
    })
}
