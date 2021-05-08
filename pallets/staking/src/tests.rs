use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

#[test]
fn stake_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(1), 10));

        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::saturating_from_rational(2, 100));
        assert_eq!(TotalStakingAsset::<Test>::get(), 10);
        assert_eq!(TotalVoucher::<Test>::get(), 500);

        // Check balance is correct
        assert_eq!(<Test as Config>::Currency::free_balance(CurrencyId::DOT, &1), 90);
        assert_eq!(<Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1), 500);
        assert_eq!(<Test as Config>::Currency::free_balance(CurrencyId::DOT, &LiquidStaking::account_id()), 10);
    })
}