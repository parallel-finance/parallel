// Copyright 2021 Parallel Finance Developer.
// This file is part of Parallel Finance.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod edge_cases;
mod interest_rate;
mod liquidate_borrow;
mod market;
mod ptokens;

use super::*;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{
    traits::{CheckedDiv, One, Saturating},
    FixedU128, Permill,
};

use mock::*;

#[test]
fn init_minting_ok() {
    new_test_ext().execute_with(|| {
        assert_eq!(Assets::balance(KSM, ALICE), dollar(1000));
        assert_eq!(Assets::balance(DOT, ALICE), dollar(1000));
        assert_eq!(Assets::balance(USDT, ALICE), dollar(1000));
        assert_eq!(Assets::balance(KSM, BOB), dollar(1000));
        assert_eq!(Assets::balance(DOT, BOB), dollar(1000));
    });
}

#[test]
fn init_markets_ok() {
    new_test_ext().execute_with(|| {
        assert_eq!(Loans::market(KSM).unwrap().state, MarketState::Active);
        assert_eq!(Loans::market(DOT).unwrap().state, MarketState::Active);
        assert_eq!(Loans::market(USDT).unwrap().state, MarketState::Active);
        assert_eq!(BorrowIndex::<Test>::get(HKO), Rate::one());
        assert_eq!(BorrowIndex::<Test>::get(KSM), Rate::one());
        assert_eq!(BorrowIndex::<Test>::get(DOT), Rate::one());
        assert_eq!(BorrowIndex::<Test>::get(USDT), Rate::one());

        assert_eq!(
            ExchangeRate::<Test>::get(KSM),
            Rate::saturating_from_rational(2, 100)
        );
        assert_eq!(
            ExchangeRate::<Test>::get(DOT),
            Rate::saturating_from_rational(2, 100)
        );
        assert_eq!(
            ExchangeRate::<Test>::get(USDT),
            Rate::saturating_from_rational(2, 100)
        );
    });
}

#[test]
fn loans_native_token_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(<Test as Config>::Assets::balance(HKO, &DAVE), dollar(1000));
        assert_eq!(Loans::market(HKO).unwrap().state, MarketState::Active);
        assert_eq!(BorrowIndex::<Test>::get(HKO), Rate::one());
        assert_eq!(
            ExchangeRate::<Test>::get(HKO),
            Rate::saturating_from_rational(2, 100)
        );
        assert_ok!(Loans::mint(Origin::signed(DAVE), HKO, dollar(1000)));

        // Redeem 1001 HKO should cause InsufficientDeposit
        assert_noop!(
            Loans::redeem_allowed(HKO, &DAVE, dollar(50050), &MARKET_MOCK),
            Error::<Test>::InsufficientDeposit
        );
        // Redeem 1000 HKO is ok
        assert_ok!(Loans::redeem_allowed(
            HKO,
            &DAVE,
            dollar(50000),
            &MARKET_MOCK
        ));

        assert_ok!(Loans::collateral_asset(Origin::signed(DAVE), HKO, true));

        // Borrow 500 HKO will reduce 500 HKO liquidity for collateral_factor is 50%
        assert_ok!(Loans::borrow(Origin::signed(DAVE), HKO, dollar(500)));
        // Repay 400 HKO
        assert_ok!(Loans::repay_borrow(Origin::signed(DAVE), HKO, dollar(400)));

        // HKO collateral: deposit = 1000
        // HKO borrow balance: borrow - repay = 500 - 400 = 100
        // HKO: cash - deposit + borrow - repay = 1000 - 1000 + 500 - 400 = 100
        assert_eq!(
            Loans::exchange_rate(HKO)
                .saturating_mul_int(Loans::account_deposits(HKO, DAVE).voucher_balance),
            dollar(1000)
        );
        let borrow_snapshot = Loans::account_borrows(HKO, DAVE);
        assert_eq!(borrow_snapshot.principal, dollar(100));
        assert_eq!(borrow_snapshot.borrow_index, Loans::borrow_index(HKO));
        assert_eq!(<Test as Config>::Assets::balance(HKO, &DAVE), dollar(100),);
    })
}

#[test]
fn mint_works() {
    new_test_ext().execute_with(|| {
        // Deposit 100 DOT
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(100)));

        // DOT collateral: deposit = 100
        // DOT: cash - deposit = 1000 - 100 = 900
        assert_eq!(
            Loans::exchange_rate(DOT)
                .saturating_mul_int(Loans::account_deposits(DOT, ALICE).voucher_balance),
            dollar(100)
        );
        assert_eq!(<Test as Config>::Assets::balance(DOT, &ALICE), dollar(900),);
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &Loans::account_id()),
            dollar(100),
        );
    })
}

#[test]
fn mint_must_return_err_when_overflows_occur() {
    new_test_ext().execute_with(|| {
        Loans::update_market(
            Origin::root(),
            DOT,
            Market {
                cap: u128::MAX,
                ..MARKET_MOCK
            },
        )
        .unwrap();
        // MAX_DEPOSIT = u128::MAX * exchangeRate
        const OVERFLOW_DEPOSIT: u128 = u128::MAX / 50 + 1;

        // Verify token balance first
        assert_noop!(
            Loans::mint(Origin::signed(CHARLIE), DOT, OVERFLOW_DEPOSIT),
            pallet_assets::Error::<Test>::BalanceLow
        );

        // Deposit OVERFLOW_DEPOSIT DOT for CHARLIE
        assert_ok!(Assets::mint(
            Origin::signed(ALICE),
            DOT,
            CHARLIE,
            OVERFLOW_DEPOSIT
        ));

        // Amount is too large, OVERFLOW_DEPOSIT / 0.0X == Overflow
        // Underflow is used here redeem could also be 0
        assert_noop!(
            Loans::mint(Origin::signed(CHARLIE), DOT, OVERFLOW_DEPOSIT),
            ArithmeticError::Underflow
        );

        // Exchange rate must ge greater than zero
        ExchangeRate::<Test>::insert(DOT, Rate::zero());
        assert_noop!(
            Loans::mint(Origin::signed(CHARLIE), DOT, 100),
            ArithmeticError::Underflow
        );
    })
}

#[test]
fn redeem_allowed_works() {
    new_test_ext().execute_with(|| {
        // Prepare: Bob Deposit 200 DOT
        assert_ok!(Loans::mint(Origin::signed(BOB), DOT, 200));

        // Deposit 200 KSM as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), KSM, 200));
        // Redeem 201 KSM should cause InsufficientDeposit
        assert_noop!(
            Loans::redeem_allowed(KSM, &ALICE, 10050, &MARKET_MOCK),
            Error::<Test>::InsufficientDeposit
        );
        // Redeem 1 DOT should cause InsufficientDeposit
        assert_noop!(
            Loans::redeem_allowed(DOT, &ALICE, 50, &MARKET_MOCK),
            Error::<Test>::InsufficientDeposit
        );
        // Redeem 200 KSM is ok
        assert_ok!(Loans::redeem_allowed(KSM, &ALICE, 10000, &MARKET_MOCK));

        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), KSM, true));
        // Borrow 50 DOT will reduce 100 KSM liquidity for collateral_factor is 50%
        assert_ok!(Loans::borrow(Origin::signed(ALICE), DOT, 50));
        // Redeem 101 KSM should cause InsufficientLiquidity
        assert_noop!(
            Loans::redeem_allowed(KSM, &ALICE, 5050, &MARKET_MOCK),
            Error::<Test>::InsufficientLiquidity
        );
        // Redeem 100 KSM is ok
        assert_ok!(Loans::redeem_allowed(KSM, &ALICE, 5000, &MARKET_MOCK));
    })
}

#[test]
fn redeem_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(100)));
        assert_ok!(Loans::redeem(Origin::signed(ALICE), DOT, dollar(20)));

        // DOT collateral: deposit - redeem = 100 - 20 = 80
        // DOT: cash - deposit + redeem = 1000 - 100 + 20 = 920
        assert_eq!(
            Loans::exchange_rate(DOT)
                .saturating_mul_int(Loans::account_deposits(DOT, ALICE).voucher_balance),
            dollar(80)
        );
        assert_eq!(<Test as Config>::Assets::balance(DOT, &ALICE), dollar(920),);
    })
}

#[test]
fn redeem_must_return_err_when_overflows_occur() {
    new_test_ext().execute_with(|| {
        // Amount is too large, max_value / 0.0X == Overflow
        // Underflow is used here redeem could also be 0
        assert_noop!(
            Loans::redeem(Origin::signed(ALICE), DOT, u128::MAX),
            ArithmeticError::Underflow,
        );

        // Exchange rate must ge greater than zero
        ExchangeRate::<Test>::insert(DOT, Rate::zero());
        assert_noop!(
            Loans::redeem(Origin::signed(ALICE), DOT, 100),
            ArithmeticError::Underflow
        );
    })
}

#[test]
fn redeem_all_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(100)));
        assert_ok!(Loans::redeem_all(Origin::signed(ALICE), DOT));

        // DOT: cash - deposit + redeem = 1000 - 100 + 100 = 1000
        // DOT collateral: deposit - redeem = 100 - 100 = 0
        assert_eq!(
            Loans::exchange_rate(DOT)
                .saturating_mul_int(Loans::account_deposits(DOT, ALICE).voucher_balance),
            0,
        );
        assert_eq!(<Test as Config>::Assets::balance(DOT, &ALICE), dollar(1000),);
        assert!(!AccountDeposits::<Test>::contains_key(DOT, &ALICE))
    })
}

#[test]
fn borrow_allowed_works() {
    new_test_ext().execute_with(|| {
        // Deposit 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(BOB), DOT, 200));
        assert_ok!(Loans::mint(Origin::signed(ALICE), KSM, 200));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), KSM, true));
        // Borrow 101 DOT should cause InsufficientLiquidity
        assert_noop!(
            Loans::borrow_allowed(DOT, &ALICE, 101),
            Error::<Test>::InsufficientLiquidity
        );
        // Borrow 100 DOT is ok
        assert_ok!(Loans::borrow_allowed(DOT, &ALICE, 100));
    })
}

#[test]
fn borrow_works() {
    new_test_ext().execute_with(|| {
        // Deposit 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        // Borrow 100 DOT
        assert_ok!(Loans::borrow(Origin::signed(ALICE), DOT, dollar(100)));

        // DOT collateral: deposit = 200
        // DOT borrow balance: borrow = 100
        // DOT: cash - deposit + borrow = 1000 - 200 + 100 = 900
        assert_eq!(
            Loans::exchange_rate(DOT)
                .saturating_mul_int(Loans::account_deposits(DOT, ALICE).voucher_balance),
            dollar(200)
        );
        let borrow_snapshot = Loans::account_borrows(DOT, ALICE);
        assert_eq!(borrow_snapshot.principal, dollar(100));
        assert_eq!(borrow_snapshot.borrow_index, Loans::borrow_index(DOT));
        assert_eq!(<Test as Config>::Assets::balance(DOT, &ALICE), dollar(900),);
    })
}

#[test]
fn repay_borrow_works() {
    new_test_ext().execute_with(|| {
        // Deposit 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        // Borrow 100 DOT
        assert_ok!(Loans::borrow(Origin::signed(ALICE), DOT, dollar(100)));
        // Repay 30 DOT
        assert_ok!(Loans::repay_borrow(Origin::signed(ALICE), DOT, dollar(30)));

        // DOT collateral: deposit = 200
        // DOT borrow balance: borrow - repay = 100 - 30 = 70
        // DOT: cash - deposit + borrow - repay = 1000 - 200 + 100 - 30 = 870
        assert_eq!(
            Loans::exchange_rate(DOT)
                .saturating_mul_int(Loans::account_deposits(DOT, ALICE).voucher_balance),
            dollar(200)
        );
        let borrow_snapshot = Loans::account_borrows(DOT, ALICE);
        assert_eq!(borrow_snapshot.principal, dollar(70));
        assert_eq!(borrow_snapshot.borrow_index, Loans::borrow_index(DOT));
        assert_eq!(<Test as Config>::Assets::balance(DOT, &ALICE), dollar(870),);
    })
}

#[test]
fn repay_borrow_all_works() {
    new_test_ext().execute_with(|| {
        // Bob deposits 200 KSM
        assert_ok!(Loans::mint(Origin::signed(BOB), KSM, dollar(200)));
        // Alice deposit 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        // Alice borrow 50 KSM
        assert_ok!(Loans::borrow(Origin::signed(ALICE), KSM, dollar(50)));

        // Alice repay all borrow balance
        assert_ok!(Loans::repay_borrow_all(Origin::signed(ALICE), KSM));

        // DOT: cash - deposit +  = 1000 - 200 = 800
        // DOT collateral: deposit = 200
        // KSM: cash + borrow - repay = 1000 + 50 - 50 = 1000
        // KSM borrow balance: borrow - repay = 50 - 50 = 0
        assert_eq!(<Test as Config>::Assets::balance(DOT, &ALICE), dollar(800),);
        assert_eq!(
            Loans::exchange_rate(DOT)
                .saturating_mul_int(Loans::account_deposits(DOT, ALICE).voucher_balance),
            dollar(200)
        );
        let borrow_snapshot = Loans::account_borrows(KSM, ALICE);
        assert_eq!(borrow_snapshot.principal, 0);
        assert_eq!(borrow_snapshot.borrow_index, Loans::borrow_index(KSM));
    })
}

#[test]
fn collateral_asset_works() {
    new_test_ext().execute_with(|| {
        // No collateral assets
        assert_noop!(
            Loans::collateral_asset(Origin::signed(ALICE), DOT, true),
            Error::<Test>::NoDeposit
        );
        // Deposit 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, 200));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        assert_eq!(Loans::account_deposits(DOT, ALICE).is_collateral, true);
        assert_noop!(
            Loans::collateral_asset(Origin::signed(ALICE), DOT, true),
            Error::<Test>::DuplicateOperation
        );
        // Borrow 100 DOT base on the collateral of 200 DOT
        assert_ok!(Loans::borrow(Origin::signed(ALICE), DOT, 100));
        assert_noop!(
            Loans::collateral_asset(Origin::signed(ALICE), DOT, false),
            Error::<Test>::InsufficientLiquidity
        );
        // Repay all the borrows
        assert_ok!(Loans::repay_borrow_all(Origin::signed(ALICE), DOT));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, false));
        assert_eq!(Loans::account_deposits(DOT, ALICE).is_collateral, false);
        assert_noop!(
            Loans::collateral_asset(Origin::signed(ALICE), DOT, false),
            Error::<Test>::DuplicateOperation
        );
    })
}

#[test]
fn total_collateral_value_works() {
    new_test_ext().execute_with(|| {
        // Mock the price for DOT = 1, KSM = 1
        let collateral_factor = Rate::saturating_from_rational(50, 100);
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(100)));
        assert_ok!(Loans::mint(Origin::signed(ALICE), KSM, dollar(200)));
        assert_ok!(Loans::mint(Origin::signed(ALICE), USDT, dollar(300)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), KSM, true));
        assert_eq!(
            Loans::total_collateral_value(&ALICE).unwrap(),
            (collateral_factor.saturating_mul(FixedU128::from_inner(dollar(100) + dollar(200))))
        );
    })
}

#[test]
fn add_reserves_works() {
    new_test_ext().execute_with(|| {
        // Add 100 DOT reserves
        assert_ok!(Loans::add_reserves(Origin::root(), ALICE, DOT, dollar(100)));

        assert_eq!(Loans::total_reserves(DOT), dollar(100));
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &Loans::account_id()),
            dollar(100),
        );
        assert_eq!(<Test as Config>::Assets::balance(DOT, &ALICE), dollar(900),);
    })
}

#[test]
fn reduce_reserves_works() {
    new_test_ext().execute_with(|| {
        // Add 100 DOT reserves
        assert_ok!(Loans::add_reserves(Origin::root(), ALICE, DOT, dollar(100)));

        // Reduce 20 DOT reserves
        assert_ok!(Loans::reduce_reserves(
            Origin::root(),
            ALICE,
            DOT,
            dollar(20)
        ));

        assert_eq!(Loans::total_reserves(DOT), dollar(80));
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &Loans::account_id()),
            dollar(80),
        );
        assert_eq!(<Test as Config>::Assets::balance(DOT, &ALICE), dollar(920),);
    })
}

#[test]
fn reduce_reserve_reduce_amount_must_be_less_than_total_reserves() {
    new_test_ext().execute_with(|| {
        assert_ok!(Loans::add_reserves(Origin::root(), ALICE, DOT, dollar(100)));
        assert_noop!(
            Loans::reduce_reserves(Origin::root(), ALICE, DOT, dollar(200)),
            Error::<Test>::InsufficientReserves
        );
    })
}

#[test]
fn ratio_and_rate_works() {
    new_test_ext().execute_with(|| {
        // Permill to FixedU128
        let ratio = Permill::from_percent(50);
        let rate: FixedU128 = ratio.into();
        assert_eq!(rate, FixedU128::saturating_from_rational(1, 2));

        // Permill  (one = 1_000_000)
        let permill = Permill::from_percent(50);
        assert_eq!(permill.mul_floor(100_u128), 50_u128);

        // FixedU128 (one = 1_000_000_000_000_000_000_000)
        let value1 = FixedU128::saturating_from_integer(100);
        let value2 = FixedU128::saturating_from_integer(10);
        assert_eq!(
            value1.checked_mul(&value2),
            Some(FixedU128::saturating_from_integer(1000))
        );
        assert_eq!(
            value1.checked_div(&value2),
            Some(FixedU128::saturating_from_integer(10))
        );
        assert_eq!(
            value1.saturating_mul(permill.into()),
            FixedU128::saturating_from_integer(50)
        );

        let value1 = FixedU128::saturating_from_rational(9, 10);
        let value2 = 10_u128;
        let value3 = FixedU128::saturating_from_integer(10_u128);
        assert_eq!(
            value1.reciprocal(),
            Some(FixedU128::saturating_from_rational(10, 9))
        );
        // u128 div FixedU128
        assert_eq!(
            FixedU128::saturating_from_integer(value2).checked_div(&value1),
            Some(FixedU128::saturating_from_rational(100, 9))
        );

        // FixedU128 div u128
        assert_eq!(
            value1.reciprocal().and_then(|r| r.checked_mul_int(value2)),
            Some(11)
        );
        assert_eq!(
            FixedU128::from_inner(17_777_777_777_777_777_777).checked_div_int(value2),
            Some(1)
        );
        // FixedU128 mul u128
        assert_eq!(
            FixedU128::from_inner(17_777_777_777_777_777_777).checked_mul_int(value2),
            Some(177)
        );

        // reciprocal
        assert_eq!(
            FixedU128::saturating_from_integer(value2).checked_div(&value1),
            Some(FixedU128::saturating_from_rational(100, 9))
        );
        assert_eq!(
            value1
                .reciprocal()
                .and_then(|r| r.checked_mul(&FixedU128::saturating_from_integer(value2))),
            Some(FixedU128::from_inner(11_111_111_111_111_111_110))
        );
        assert_eq!(
            FixedU128::saturating_from_integer(value2)
                .checked_mul(&value3)
                .and_then(|v| v.checked_div(&value1)),
            Some(FixedU128::saturating_from_rational(1000, 9))
        );
        assert_eq!(
            FixedU128::saturating_from_integer(value2)
                .checked_div(&value1)
                .and_then(|v| v.checked_mul(&value3)),
            Some(FixedU128::from_inner(111_111_111_111_111_111_110))
        );

        // FixedU128 div Permill
        let value1 = Permill::from_percent(30);
        let value2 = Permill::from_percent(40);
        let value3 = FixedU128::saturating_from_integer(10);
        assert_eq!(
            value3.checked_div(&value1.into()),
            Some(FixedU128::saturating_from_rational(100, 3)) // 10/0.3
        );

        // u128 div Permill
        assert_eq!(value1.saturating_reciprocal_mul(5_u128), 17); // (1/0.3) * 5 = 16.66666666..
        assert_eq!(value1.saturating_reciprocal_mul_floor(5_u128), 16); // (1/0.3) * 5 = 16.66666666..
        assert_eq!(value2.saturating_reciprocal_mul(5_u128), 12); // (1/0.4) * 5 = 12.5

        // Permill * u128
        let value1 = Permill::from_percent(34);
        let value2 = Permill::from_percent(36);
        let value3 = Permill::from_percent(30);
        let value4 = Permill::from_percent(20);
        assert_eq!(value1 * 10_u64, 3); // 0.34 * 10
        assert_eq!(value2 * 10_u64, 4); // 0.36 * 10
        assert_eq!(value3 * 5_u64, 1); // 0.3 * 5
        assert_eq!(value4 * 8_u64, 2); // 0.2 * 8
        assert_eq!(value4.mul_floor(8_u64), 1); // 0.2 mul_floor 8
    })
}

#[test]
fn update_exchange_rate_works() {
    new_test_ext().execute_with(|| {
        // Initialize value of exchange rate is 0.02
        assert_eq!(
            Loans::exchange_rate(DOT),
            Rate::saturating_from_rational(2, 100)
        );

        // total_supply = 0
        TotalSupply::<Test>::insert(DOT, 0);
        assert_ok!(Loans::update_exchange_rate(DOT));
        assert_eq!(
            Loans::exchange_rate(DOT),
            Rate::saturating_from_rational(2, 100)
        );

        // exchange_rate = total_cash + total_borrows - total_reverse / total_supply
        // total_cash = 10, total_supply = 500
        // exchange_rate = 10 + 5 - 1 / 500
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(10)));
        TotalBorrows::<Test>::insert(DOT, dollar(5));
        TotalReserves::<Test>::insert(DOT, dollar(1));
        assert_ok!(Loans::update_exchange_rate(DOT));
        assert_eq!(
            Loans::exchange_rate(DOT),
            Rate::saturating_from_rational(14, 500)
        );
    })
}

#[test]
fn current_borrow_balance_works() {
    new_test_ext().execute_with(|| {
        // snapshot.principal = 0
        AccountBorrows::<Test>::insert(
            DOT,
            ALICE,
            BorrowSnapshot {
                principal: 0,
                borrow_index: Rate::one(),
            },
        );
        assert_eq!(Loans::current_borrow_balance(&ALICE, DOT).unwrap(), 0);

        // snapshot.borrow_index = 0
        AccountBorrows::<Test>::insert(
            DOT,
            ALICE,
            BorrowSnapshot {
                principal: 100,
                borrow_index: Rate::zero(),
            },
        );
        assert_eq!(Loans::current_borrow_balance(&ALICE, DOT).unwrap(), 0);

        // borrow_index = 1.2, snapshot.borrow_index = 1, snapshot.principal = 100
        BorrowIndex::<Test>::insert(DOT, Rate::saturating_from_rational(12, 10));
        AccountBorrows::<Test>::insert(
            DOT,
            ALICE,
            BorrowSnapshot {
                principal: 100,
                borrow_index: Rate::one(),
            },
        );
        assert_eq!(Loans::current_borrow_balance(&ALICE, DOT).unwrap(), 120);
    })
}

#[test]
fn calc_collateral_amount_works() {
    let exchange_rate = Rate::saturating_from_rational(3, 10);
    assert_eq!(
        Loans::calc_collateral_amount(1000, exchange_rate).unwrap(),
        3333
    );
    assert_eq!(
        Loans::calc_collateral_amount(u128::MAX, exchange_rate),
        Err(DispatchError::Arithmetic(ArithmeticError::Underflow))
    );
}

#[test]
fn get_price_works() {
    MockPriceFeeder::set_price(DOT, 0.into());
    assert!(Loans::get_price(DOT).is_err());

    MockPriceFeeder::set_price(DOT, 2.into());
    assert_eq!(
        Loans::get_price(DOT).unwrap(),
        Price::saturating_from_integer(2)
    );
}

#[test]
fn ensure_enough_cash_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::mint(
            Origin::signed(ALICE),
            KSM,
            Loans::account_id(),
            dollar(1000)
        ));
        assert_ok!(Loans::ensure_enough_cash(KSM, dollar(1000)));
        TotalReserves::<Test>::insert(KSM, dollar(10));
        assert_noop!(
            Loans::ensure_enough_cash(KSM, dollar(1000)),
            Error::<Test>::InsufficientCash,
        );
        assert_ok!(Loans::ensure_enough_cash(KSM, dollar(990)));
    })
}
