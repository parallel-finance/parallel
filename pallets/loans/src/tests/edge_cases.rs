use crate::{
    mock::{new_test_ext, Loans, Origin, Test, ALICE, DOT, KSM},
    tests::{dollar, run_to_block},
    Config,
};
use frame_support::assert_ok;
use sp_runtime::FixedPointNumber;

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

        assert_eq!(
            <Test as Config>::Assets::balance(KSM, &ALICE),
            dollar(800) - 56,
        );

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
