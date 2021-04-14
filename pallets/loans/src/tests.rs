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

//! Unit tests for the loans module.
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use primitives::{CurrencyId, BLOCK_PER_YEAR, RATE_DECIMAL, TOKEN_DECIMAL};

use super::*;

use mock::*;

#[test]
fn mock_genesis_ok() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(TotalBorrows::<Runtime>::get(DOT), 0 * TOKEN_DECIMAL);
        assert_eq!(TotalSupply::<Runtime>::get(BTC), 0 * TOKEN_DECIMAL);
        assert_eq!(BorrowIndex::<Runtime>::get(USDT), RATE_DECIMAL);
        assert_eq!(CollateralRate::<Runtime>::get(KSM), 5 * RATE_DECIMAL / 10);
    });
}

// Test rate module
#[test]
fn utilization_rate_works() {
    // 50% borrow
    assert_eq!(
        Loans::utilization_rate(1, 1, 0).unwrap(),
        5 * RATE_DECIMAL / 10
    );
    assert_eq!(
        Loans::utilization_rate(100, 100, 0).unwrap(),
        5 * RATE_DECIMAL / 10
    );
    // no borrow
    assert_eq!(
        Loans::utilization_rate(1, 0, 0).unwrap(),
        0 * RATE_DECIMAL / 10
    );
    // full borrow
    assert_eq!(Loans::utilization_rate(0, 1, 0).unwrap(), 1 * RATE_DECIMAL);
}

#[test]
fn update_jump_rate_model_works() {
    ExtBuilder::default().build().execute_with(|| {
        let base_rate_per_year: u128 = 2 * RATE_DECIMAL / 100;
        let multiplier_per_year: u128 = RATE_DECIMAL / 10;
        let jump_multiplier_per_year: u128 = 11 * RATE_DECIMAL / 10;
        let kink: u128 = 8 * RATE_DECIMAL / 10;
        assert_ok!(Loans::init_jump_rate_model(
            base_rate_per_year,
            multiplier_per_year,
            jump_multiplier_per_year,
            kink,
        ));
        assert_eq!(
            BaseRatePerBlock::<Runtime>::get(),
            Some(base_rate_per_year / BLOCK_PER_YEAR)
        );
        assert_eq!(
            MultiplierPerBlock::<Runtime>::get(),
            Some(multiplier_per_year * RATE_DECIMAL / (BLOCK_PER_YEAR * kink))
        );
        assert_eq!(
            JumpMultiplierPerBlock::<Runtime>::get(),
            Some(jump_multiplier_per_year / BLOCK_PER_YEAR)
        );
        assert_eq!(Kink::<Runtime>::get(), Some(kink));
    });
}

#[test]
fn update_borrow_rate_works() {
    ExtBuilder::default().build().execute_with(|| {
        // normal rate
        let mut cash: u128 = 5 * TOKEN_DECIMAL;
        let borrows: u128 = 10 * TOKEN_DECIMAL;
        let reserves: u128 = 0;
        assert_ok!(Loans::update_borrow_rate(DOT, cash, borrows, reserves));
        let util = Loans::utilization_rate(cash, borrows, reserves).unwrap();
        let multiplier_per_block = MultiplierPerBlock::<Runtime>::get().unwrap();
        let base_rate_per_block = BaseRatePerBlock::<Runtime>::get().unwrap();
        let kink = Kink::<Runtime>::get().unwrap();
        let jump_multiplier_per_block = JumpMultiplierPerBlock::<Runtime>::get().unwrap();
        assert_eq!(
            util * multiplier_per_block / RATE_DECIMAL + base_rate_per_block,
            BorrowRate::<Runtime>::get(DOT),
        );

        // jump rate
        cash = 1 * TOKEN_DECIMAL;
        assert_ok!(Loans::update_borrow_rate(KSM, cash, borrows, reserves));
        let normal_rate = kink * multiplier_per_block / RATE_DECIMAL + base_rate_per_block;
        let excess_util = util.saturating_sub(kink);
        assert_eq!(
            excess_util * (jump_multiplier_per_block / RATE_DECIMAL) + normal_rate,
            BorrowRate::<Runtime>::get(KSM),
        );
    });
}

#[test]
fn calc_exchange_rate_works() {}

#[test]
fn mint_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 100 DOT
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, 100));

        // DOT collateral: deposit = 100
        // DOT: cash - deposit = 1000 - 100 = 900
        assert_eq!(
            Loans::account_collateral(DOT, ALICE) * Loans::exchange_rate(DOT) / RATE_DECIMAL,
            100
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            900,
        );
    })
}

#[test]
fn mint_failed() {
    ExtBuilder::default().build().execute_with(|| {
        // calculate collateral amount failed
        ExchangeRate::<Runtime>::insert(DOT, 0);
        assert_noop!(
            Loans::mint(Origin::signed(ALICE), DOT, 100),
            Error::<Runtime>::CalcCollateralFailed,
        );
    })
}

#[test]
fn redeem_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 100 DOT
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, 100));
        // Redeem 20 DOT
        assert_ok!(Loans::redeem(Origin::signed(ALICE), DOT, 20));

        // DOT collateral: deposit - redeem = 100 - 20 = 80
        // DOT: cash - deposit + redeem = 1000 - 100 + 20 = 920
        assert_eq!(
            Loans::account_collateral(DOT, ALICE) * Loans::exchange_rate(DOT) / RATE_DECIMAL,
            80
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            920,
        );
    })
}

#[test]
fn redeem_all_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 100 DOT
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, 100));
        // Redeem all DOT
        assert_ok!(Loans::redeem_all(Origin::signed(ALICE), DOT));

        // DOT: cash - deposit + redeem = 1000 - 100 + 100 = 1000
        // DOT collateral: deposit - redeem = 100 - 100 = 0
        assert_eq!(
            Loans::account_collateral(DOT, ALICE) * Loans::exchange_rate(DOT) / RATE_DECIMAL,
            0,
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            1000,
        );
    })
}

#[test]
fn borrow_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, 200));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        // Borrow 100 DOT
        assert_ok!(Loans::borrow(Origin::signed(ALICE), DOT, 100));

        // DOT collateral: deposit = 200
        // DOT borrow balance: borrow = 100
        // DOT: cash - deposit + borrow = 1000 - 200 + 100 = 900
        assert_eq!(
            Loans::account_collateral(DOT, ALICE) * Loans::exchange_rate(DOT) / RATE_DECIMAL,
            200
        );
        let borrow_snapshot = Loans::account_borrows(DOT, ALICE);
        assert_eq!(borrow_snapshot.principal, 100);
        assert_eq!(borrow_snapshot.interest_index, Loans::borrow_index(DOT));
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            900,
        );
    })
}

#[test]
fn repay_borrow_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, 200));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        // Borrow 100 DOT
        assert_ok!(Loans::borrow(Origin::signed(ALICE), DOT, 100));
        // Repay 30 DOT
        assert_ok!(Loans::repay_borrow(Origin::signed(ALICE), DOT, 30));

        // DOT collateral: deposit = 200
        // DOT borrow balance: borrow - repay = 100 - 30 = 70
        // DOT: cash - deposit + borrow - repay = 1000 - 200 + 100 - 30 = 870
        assert_eq!(
            Loans::account_collateral(DOT, ALICE) * Loans::exchange_rate(DOT) / RATE_DECIMAL,
            200
        );
        let borrow_snapshot = Loans::account_borrows(DOT, ALICE);
        assert_eq!(borrow_snapshot.principal, 70);
        assert_eq!(borrow_snapshot.interest_index, Loans::borrow_index(DOT));
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            870,
        );
    })
}

#[test]
fn repay_borrow_all_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Bob deposits 200 KSM
        assert_ok!(Loans::mint(Origin::signed(BOB), KSM, 200));
        // Alice deposit 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, 200));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        // Alice borrow 100 KSM
        assert_ok!(Loans::borrow(Origin::signed(ALICE), KSM, 100));
        // Alice repay all borrow balance
        assert_ok!(Loans::repay_borrow_all(Origin::signed(ALICE), KSM));

        // DOT: cash - deposit +  = 1000 - 200 = 800
        // DOT collateral: deposit = 200
        // KSM: cash + borrow - repay = 1000 + 100 - 100 = 1000
        // KSM borrow balance: borrow - repay = 100 - 100 = 0
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            800,
        );
        assert_eq!(
            Loans::account_collateral(DOT, ALICE) * Loans::exchange_rate(DOT) / RATE_DECIMAL,
            200,
        );
        let borrow_snapshot = Loans::account_borrows(KSM, ALICE);
        assert_eq!(borrow_snapshot.principal, 0);
        assert_eq!(borrow_snapshot.interest_index, Loans::borrow_index(KSM));
    })
}

#[test]
fn liquidate_borrow_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Bob deposits 200 KSM
        assert_ok!(Loans::mint(Origin::signed(BOB), KSM, 200));
        // Alice deposits 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, 200));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        // Alice borrows 100 KSM
        assert_ok!(Loans::borrow(Origin::signed(ALICE), KSM, 100));
        // adjust KSM price to make ALICE generate shortfall
        MOCK_PRICE_FEEDER::set_price(KSM, 2);
        // BOB repay the KSM borrow balance and get DOT from ALICE
        assert_ok!(Loans::liquidate_borrow(
            Origin::signed(BOB),
            ALICE,
            KSM,
            50,
            DOT
        ));

        // incentive = repay KSM value / 0.9
        // Alice DOT: cash - deposit = 1000 - 200 = 800
        // Alice DOT collateral: deposit - incentive = 200 - (50 * 2 / 0.9) = 89
        // Alice KSM: cash + borrow = 1000 + 100 = 1100
        // Alice KSM borrow balance: origin borrow balance - repay = 100 - 50 = 50
        // Bob KSM: cash - deposit -repay = 1000 - 200 - 50 = 750
        // Bob DOT collateral: incentive = 50 * 2 / 0.9 = 111
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            800,
        );
        assert_eq!(
            Loans::account_collateral(DOT, ALICE) * Loans::exchange_rate(DOT) / RATE_DECIMAL,
            89,
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(KSM, &ALICE),
            1100,
        );
        assert_eq!(Loans::account_borrows(KSM, ALICE).principal, 50);
        assert_eq!(<Runtime as Config>::Currency::free_balance(KSM, &BOB), 750);
        assert_eq!(
            Loans::account_collateral(DOT, BOB) * Loans::exchange_rate(DOT) / RATE_DECIMAL,
            111,
        );
    })
}

#[test]
fn collateral_asset_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, 200));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        assert_eq!(Loans::account_collateral_assets(ALICE), vec![DOT]);
    })
}
