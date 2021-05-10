use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;

#[test]
fn stake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(1), 10));

        // Check storage is correct
        assert_eq!(
            ExchangeRate::<Test>::get(),
            Rate::saturating_from_rational(2, 100)
        );
        assert_eq!(TotalStakingAsset::<Test>::get(), 10);
        assert_eq!(TotalVoucher::<Test>::get(), 500);

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1),
            90
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1),
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
        let _ = LiquidStaking::stake(Origin::signed(1), 10);
        assert_ok!(LiquidStaking::withdraw(Origin::signed(6), 2, 10));

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1),
            90
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &2),
            10
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
        let _ = LiquidStaking::stake(Origin::signed(1), 10);
        assert_noop!(LiquidStaking::withdraw(Origin::signed(1), 2, 11), BadOrigin,);
    })
}

#[test]
fn withdraw_too_much_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1), 10);
        assert_noop!(
            LiquidStaking::withdraw(Origin::signed(6), 2, 11),
            Error::<Test>::ExcessWithdraw,
        );
    })
}

#[test]
fn record_rewards_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1), 10);
        assert_ok!(LiquidStaking::record_rewards(Origin::signed(6), 2, 10));

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
        let _ = LiquidStaking::stake(Origin::signed(1), 10);
        assert_noop!(
            LiquidStaking::record_rewards(Origin::signed(1), 2, 10),
            BadOrigin,
        );
    })
}

#[test]
fn unstake_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1), 10);
        assert_ok!(LiquidStaking::unstake(Origin::signed(1), 500));

        // Check storage is correct
        assert_eq!(
            ExchangeRate::<Test>::get(),
            Rate::saturating_from_rational(2, 100)
        );
        assert_eq!(TotalStakingAsset::<Test>::get(), 0);
        assert_eq!(TotalVoucher::<Test>::get(), 0);
        assert_eq!(
            AccountPendingUnstake::<Test>::get(&1).unwrap(),
            UnstakeInfo {
                amount: 10,
                block_number: frame_system::Pallet::<Test>::block_number()
            }
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1),
            90
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1),
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
        let _ = LiquidStaking::stake(Origin::signed(1), 10);
        assert_noop!(
            LiquidStaking::unstake(Origin::signed(1), 501),
            orml_tokens::Error::<Test>::BalanceTooLow,
        );
    })
}
