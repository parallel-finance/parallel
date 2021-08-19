use crate::types::{
    MatchingLedger, OperationSatus, StakingOperationType, StakingSettlementKind, UnstakeMisc,
};
use crate::{mock::*, *};
use frame_support::{assert_err, assert_ok};
use orml_traits::MultiCurrency;
use primitives::{CurrencyId, EraIndex, Rate};
use sp_runtime::traits::One;

use crate::types::*;

fn t_insert_pending_op(era_index: EraIndex) {
    let block_number = System::block_number();
    StakingOperationHistory::<Test>::insert(
        era_index,
        StakingOperationType::WithdrawUnbonded,
        Operation {
            amount: 1u64.into(),
            block_number,
            status: OperationSatus::Pending,
        },
    )
}

#[test]
fn stake_should_work() {
    new_test_ext().execute_with(|| {
        let currency_era: EraIndex = 100;
        CurrentEra::<Test>::put(currency_era);

        assert_ok!(LiquidStaking::stake(Origin::signed(Alice), 10));
        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::one());
        assert_eq!(StakingPool::<Test>::get(), 10);
        assert_eq!(
            MatchingPool::<Test>::get(currency_era),
            MatchingLedger {
                total_stake_amount: 10,
                total_unstake_amount: 0,
            }
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &Alice),
            90
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &Alice),
            110
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            10
        );
    })
}

#[test]
fn unstake_should_work() {
    new_test_ext().execute_with(|| {
        let currency_era: EraIndex = 100;
        CurrentEra::<Test>::put(currency_era);

        assert_ok!(LiquidStaking::stake(Origin::signed(Alice), 10));
        assert_ok!(LiquidStaking::unstake(Origin::signed(Alice), 6));

        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::one());
        assert_eq!(StakingPool::<Test>::get(), 4);
        assert_eq!(
            AccountUnstake::<Test>::get(Alice, currency_era),
            UnstakeMisc {
                total_amount: 6,
                claimed_amount: 0,
            }
        );
        assert_eq!(
            MatchingPool::<Test>::get(currency_era),
            MatchingLedger {
                total_stake_amount: 10,
                total_unstake_amount: 6,
            }
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &Alice),
            90
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &Alice),
            104
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()),
            10
        );
    })
}

#[test]
fn test_record_staking_settlement_ok() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(Alice), 100));
        assert_eq!(
            <Test as Config>::Currency::total_issuance(CurrencyId::xDOT,),
            200
        );
        assert_eq!(LiquidStaking::exchange_rate(), Rate::from(1));
        assert_ok!(LiquidStaking::record_staking_settlement(
            Origin::signed(Alice),
            1,
            300,
            StakingSettlementKind::Reward
        ));

        assert_eq!(LiquidStaking::exchange_rate(), Rate::from(2));
    })
}

#[test]
fn test_duplicated_record_staking_settlement() {
    new_test_ext().execute_with(|| {
        LiquidStaking::record_staking_settlement(
            Origin::signed(Alice),
            1,
            100,
            StakingSettlementKind::Reward,
        )
        .unwrap();

        assert_err!(
            LiquidStaking::record_staking_settlement(
                Origin::signed(Alice),
                1,
                100,
                StakingSettlementKind::Reward
            ),
            Error::<Test>::StakingSettlementAlreadyRecorded
        )
    })
}

#[test]
fn test_trigger_new_era() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::trigger_new_era(Origin::signed(Alice), 1));
        assert_eq!(LiquidStaking::previous_era(), 0u32);
        assert_eq!(LiquidStaking::current_era(), 1u32);
        assert_err!(
            LiquidStaking::trigger_new_era(Origin::signed(Alice), 1),
            Error::<Test>::EraAlreadyPushed
        );
    })
}

#[test]
fn test_record_withdrawal_response() {
    new_test_ext().execute_with(|| {
        assert_err!(
            LiquidStaking::record_withdrawal_unbond_response(Origin::signed(Alice), 1u32),
            Error::<Test>::OperationNotPending
        );

        t_insert_pending_op(1u32);
        assert_ok!(LiquidStaking::record_withdrawal_unbond_response(
            Origin::signed(Alice),
            1u32
        ));

        assert_err!(
            LiquidStaking::record_withdrawal_unbond_response(Origin::signed(Alice), 1u32),
            Error::<Test>::OperationNotPending
        );
    });
}

#[test]
fn test_matching_pool_summary() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(Alice), 10));
        assert_ok!(LiquidStaking::unstake(Origin::signed(Alice), 5));

        let current_era = LiquidStaking::current_era();

        assert_eq!(
            MatchingPool::<Test>::get(current_era),
            MatchingLedger {
                total_stake_amount: 10,
                total_unstake_amount: 5,
            }
        );

        assert_ok!(LiquidStaking::trigger_new_era(Origin::signed(Alice), 1));

        assert_eq!(
            StakingOperationHistory::<Test>::get(current_era, StakingOperationType::Bond),
            Some(Operation {
                status: OperationSatus::Pending,
                block_number: 0_u64,
                amount: 5
            })
        );

        System::set_block_number(1);

        assert_ok!(LiquidStaking::unstake(Origin::signed(Alice), 5));

        assert_ok!(LiquidStaking::trigger_new_era(
            Origin::signed(Alice),
            current_era + 2
        ));

        assert_eq!(
            StakingOperationHistory::<Test>::get(current_era + 1, StakingOperationType::Unbond),
            Some(Operation {
                status: OperationSatus::Pending,
                block_number: 1_u64,
                amount: 5
            })
        );
    })
}
