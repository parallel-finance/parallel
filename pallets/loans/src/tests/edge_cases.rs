use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_err, assert_ok};
use sp_runtime::FixedPointNumber;

#[test]
fn exceeded_supply_cap() {
    new_test_ext().execute_with(|| {
        Assets::mint(Origin::signed(ALICE), DOT, ALICE, million_dollar(1001)).unwrap();
        let amount = million_dollar(501);
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, amount));
        // Exceed upper bound.
        assert_err!(
            Loans::mint(Origin::signed(ALICE), DOT, amount),
            Error::<Test>::SupplyCapacityExceeded
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

        // Alice borrow only 1/1e5 KSM which is hard to accure total borrows interest in 100 seconds
        assert_ok!(Loans::borrow(Origin::signed(ALICE), KSM, 10_u128.pow(7)));

        accrue_interest_per_block(KSM, 100, 9);

        assert_eq!(Loans::current_borrow_balance(&ALICE, KSM), Ok(10000005));
        // FIXME since total_borrows is too small and we accure internal on it every 100 seconds
        // accure_interest fails every time
        // as you can see the current borrow balance is not equal to total_borrows anymore
        assert_eq!(Loans::total_borrows(KSM), 10000000);

        // Alice repay all borrow balance. total_borrows = total_borrows.saturating_sub(10000005) = 0.
        assert_ok!(Loans::repay_borrow_all(Origin::signed(ALICE), KSM));

        assert_eq!(Assets::balance(KSM, &ALICE), dollar(800) - 5);

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
            Loans::ensure_under_supply_cap(SDOT, dollar(100)),
            Error::<Test>::MarketDoesNotExist
        );
    });
}

#[test]
fn redeem_all_should_be_accurate() {
    new_test_ext().execute_with(|| {
        assert_ok!(Loans::mint(Origin::signed(ALICE), KSM, dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), KSM, true));
        assert_ok!(Loans::borrow(Origin::signed(ALICE), KSM, dollar(50)));

        // let exchange_rate greater than 0.02
        accrue_interest_per_block(KSM, 6, 2);
        assert_eq!(
            Loans::exchange_rate(KSM),
            Rate::from_inner(20000000036387000)
        );

        assert_ok!(Loans::repay_borrow_all(Origin::signed(ALICE), KSM));
        // It failed with InsufficientLiquidity before #839
        assert_ok!(Loans::redeem_all(Origin::signed(ALICE), KSM));
    })
}

#[test]
fn prevent_the_exchange_rate_attack() {
    new_test_ext().execute_with(|| {
        // Initialize Eve's balance
        assert_ok!(<Test as Config>::Assets::transfer(
            DOT,
            &ALICE,
            &EVE,
            dollar(200),
            false
        ));
        // Eve deposits a small amount
        assert_ok!(Loans::mint(Origin::signed(EVE), DOT, 20));
        // !!! Eve transfer a big amount to Loans::account_id
        assert_ok!(<Test as Config>::Assets::transfer(
            DOT,
            &EVE,
            &Loans::account_id(),
            dollar(100),
            false
        ));
        assert_eq!(<Test as Config>::Assets::balance(DOT, &EVE), 99999999999980);
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &Loans::account_id()),
            100000000000020
        );
        assert_eq!(
            Loans::total_supply(DOT),
            20 * 50, // 20 / 0.02
        );
        TimestampPallet::set_timestamp(12000);
        // Eve can not let the exchage rate greater than 1
        assert!(Loans::accrue_interest(DOT).is_err());

        // Mock a BIG exchange_rate: 100000000000.02
        ExchangeRate::<Test>::insert(
            DOT,
            Rate::saturating_from_rational(100000000000020u128, 20 * 50),
        );
        // Bob can not deposit 0.1 DOT because the voucher_balance can not be 0.
        assert_noop!(
            Loans::mint(Origin::signed(BOB), DOT, 100000000000),
            Error::<Test>::InvalidExchangeRate
        );
    })
}
