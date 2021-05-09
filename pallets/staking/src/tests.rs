use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

#[test]
fn stake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(1), 10));

        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::saturating_from_rational(2, 100));
        assert_eq!(TotalStakingAsset::<Test>::get(), 10);
        assert_eq!(TotalVoucher::<Test>::get(), 500);

        // Check balance is correct
        assert_eq!(<Test as Config>::Currency::free_balance(CurrencyId::DOT, &1), 90);
        assert_eq!(<Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1), 500);
        assert_eq!(
            <Test as Config>::Currency::free_balance(
                CurrencyId::DOT, &LiquidStaking::account_id()
            ),
            10
        );
    })
}

#[test]
fn unstake_should_work() {
    new_test_ext().execute_with(|| {
        let _ = LiquidStaking::stake(Origin::signed(1), 10);
        assert_ok!(LiquidStaking::unstake(Origin::signed(1), 500));

        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::saturating_from_rational(2, 100));
        assert_eq!(TotalStakingAsset::<Test>::get(), 0);
        assert_eq!(TotalVoucher::<Test>::get(), 0);
        assert_eq!(
            AccountPendingUnstake::<Test>::get(&1).unwrap(),
            UnstakeInfo { amount: 10, block_number: frame_system::Pallet::<Test>::block_number()}
        );

        // Check balance is correct
        assert_eq!(<Test as Config>::Currency::free_balance(CurrencyId::DOT, &1), 90);
        assert_eq!(<Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1), 0);
        assert_eq!(
            <Test as Config>::Currency::free_balance(
                CurrencyId::DOT, &LiquidStaking::account_id()
            ),
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
