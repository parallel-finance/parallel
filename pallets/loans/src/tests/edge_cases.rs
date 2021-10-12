use crate::{
    mock::{million_dollar, new_test_ext, Assets, Loans, Origin, Test, ALICE, DOT, KSM, XDOT},
    tests::{dollar, run_to_block},
    Error,
};
use frame_support::{assert_err, assert_ok};
use sp_runtime::FixedPointNumber;

#[test]
fn exceeded_market_capacity() {
    new_test_ext().execute_with(|| {
        Assets::mint(Origin::signed(ALICE), DOT, ALICE, million_dollar(1001)).unwrap();
        let amount = million_dollar(501);
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, amount));
        // Exceed upper bound.
        assert_err!(
            Loans::mint(Origin::signed(ALICE), DOT, amount),
            Error::<Test>::ExceededMarketCapacity
        );

        Loans::redeem(Origin::signed(ALICE), DOT, amount).unwrap();
        // Here should work, cause we redeemed already.
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, amount));
    })
}

#[test]
fn repay_borrow_all_no_underflow() {
    new_test_ext().execute_with(|| {
        // Alice deposits 200 KSM as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), KSM, dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), KSM, true));

        // Alice borrow only 1/1e6 KSM which is hard to accure total borrows interest in 6 seconds
        assert_ok!(Loans::borrow(Origin::signed(ALICE), KSM, 10_u128.pow(8)));

        run_to_block(150);

        assert_eq!(Loans::current_borrow_balance(&ALICE, KSM), Ok(100000056));
        // FIXME since total_borrows is too small and we accure internal on it every 6 seconds
        // accure_interest fails every time
        // as you can see the current borrow balance is not equal to total_borrows anymore
        assert_eq!(Loans::total_borrows(KSM), 10_u128.pow(8));

        // Alice repay all borrow balance
        assert_ok!(Loans::repay_borrow_all(Origin::signed(ALICE), KSM));

        assert_eq!(Assets::balance(KSM, &ALICE), dollar(800) - 56,);

        assert_eq!(
            Loans::exchange_rate(DOT)
                .saturating_mul_int(Loans::account_deposits(KSM, ALICE).voucher_balance),
            dollar(200)
        );

        let borrow_snapshot = Loans::account_borrows(KSM, ALICE);
        assert_eq!(borrow_snapshot.principal, 0);
        assert_eq!(borrow_snapshot.borrow_index, Loans::borrow_index(KSM));
    })
}

#[test]
fn ensure_capacity_fails_when_market_not_existed() {
    new_test_ext().execute_with(|| {
        assert_err!(
            Loans::ensure_capacity(XDOT, dollar(100)),
            Error::<Test>::MarketDoesNotExist
        );
    });
}
