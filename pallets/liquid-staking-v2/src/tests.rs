use frame_support::{assert_err, assert_ok};

use crate::types::{MatchingLedger, StakeingSettlementKind, UnstakeMisc};
use crate::{mock::*, *};
use orml_traits::MultiCurrency;
use primitives::{CurrencyId, EraIndex, Rate};
use sp_runtime::{traits::One};

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
            EraMatchingPool::<Test>::get(currency_era),
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
            EraMatchingPool::<Test>::get(currency_era),
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
        assert_ok!(LiquidStaking::record_staking_settlement(
            Origin::signed(Alice),
            1,
            100,
            StakeingSettlementKind::Reward
        ));

        assert_eq!(LiquidStaking::exchange_rate(), Rate::from(1));
    })
}

#[test]
fn test_duplicated_record_staking_settlement() {
    new_test_ext().execute_with(|| {
        LiquidStaking::record_staking_settlement(
            Origin::signed(Alice),
            1,
            100,
            StakeingSettlementKind::Reward,
        )
        .unwrap();

        assert_err!(
            LiquidStaking::record_staking_settlement(
                Origin::signed(Alice),
                1,
                100,
                StakeingSettlementKind::Reward
            ),
            Error::<Test>::StakeingSettlementAlreadyRecorded
        )
    })
}

#[test]
fn test_set_era_index() {
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
