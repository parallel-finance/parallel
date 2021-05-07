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

use frame_support::{assert_noop, assert_ok};
use primitives::{BLOCK_PER_YEAR, RATE_DECIMAL};
use sp_runtime::traits::{CheckedDiv, One, Saturating};

use super::*;

use crate::loan::calc_collateral_amount;
use mock::*;

#[test]
fn mock_genesis_ok() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(BorrowIndex::<Runtime>::get(USDT), Rate::one());
        assert_eq!(
            CollateralFactor::<Runtime>::get(KSM),
            Ratio::from_percent(50)
        );
    });
}

// Test rate module
#[test]
fn utilization_rate_works() {
    // 50% borrow
    assert_eq!(
        Loans::calc_utilization_ratio(1, 1, 0).unwrap(),
        Ratio::from_percent(50)
    );
    assert_eq!(
        Loans::calc_utilization_ratio(100, 100, 0).unwrap(),
        Ratio::from_percent(50)
    );
    // no borrow
    assert_eq!(
        Loans::calc_utilization_ratio(1, 0, 0).unwrap(),
        Ratio::zero()
    );
    // full borrow
    assert_eq!(
        Loans::calc_utilization_ratio(0, 1, 0).unwrap(),
        Ratio::from_percent(100)
    );
}

#[test]
fn mint_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 100 DOT
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(100)));

        // DOT collateral: deposit = 100
        // DOT: cash - deposit = 1000 - 100 = 900
        assert_eq!(
            Loans::exchange_rate(DOT).saturating_mul_int(Loans::account_collateral(DOT, ALICE)),
            dollar(100)
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            dollar(900),
        );
    })
}

#[test]
fn mint_failed() {
    ExtBuilder::default().build().execute_with(|| {
        // calculate collateral amount failed
        ExchangeRate::<Runtime>::insert(DOT, Rate::zero());
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
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(100)));
        // Redeem 20 DOT
        assert_ok!(Loans::redeem(Origin::signed(ALICE), DOT, dollar(20)));

        // DOT collateral: deposit - redeem = 100 - 20 = 80
        // DOT: cash - deposit + redeem = 1000 - 100 + 20 = 920
        assert_eq!(
            Loans::exchange_rate(DOT).saturating_mul_int(Loans::account_collateral(DOT, ALICE)),
            dollar(80)
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            dollar(920),
        );
    })
}

#[test]
fn redeem_all_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 100 DOT
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(100)));
        // Redeem all DOT
        assert_ok!(Loans::redeem_all(Origin::signed(ALICE), DOT));

        // DOT: cash - deposit + redeem = 1000 - 100 + 100 = 1000
        // DOT collateral: deposit - redeem = 100 - 100 = 0
        assert_eq!(
            Loans::exchange_rate(DOT).saturating_mul_int(Loans::account_collateral(DOT, ALICE)),
            0,
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            dollar(1000),
        );
    })
}

#[test]
fn borrow_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        // Borrow 100 DOT
        assert_ok!(Loans::borrow(Origin::signed(ALICE), DOT, dollar(100)));

        // DOT collateral: deposit = 200
        // DOT borrow balance: borrow = 100
        // DOT: cash - deposit + borrow = 1000 - 200 + 100 = 900
        assert_eq!(
            Loans::exchange_rate(DOT).saturating_mul_int(Loans::account_collateral(DOT, ALICE)),
            dollar(200)
        );
        let borrow_snapshot = Loans::account_borrows(DOT, ALICE);
        assert_eq!(borrow_snapshot.principal, dollar(100));
        assert_eq!(borrow_snapshot.borrow_index, Loans::borrow_index(DOT));
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            dollar(900),
        );
    })
}

#[test]
fn repay_borrow_works() {
    ExtBuilder::default().build().execute_with(|| {
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
            Loans::exchange_rate(DOT).saturating_mul_int(Loans::account_collateral(DOT, ALICE)),
            dollar(200)
        );
        let borrow_snapshot = Loans::account_borrows(DOT, ALICE);
        assert_eq!(borrow_snapshot.principal, dollar(70));
        assert_eq!(borrow_snapshot.borrow_index, Loans::borrow_index(DOT));
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            dollar(870),
        );
    })
}

#[test]
fn repay_borrow_all_works() {
    ExtBuilder::default().build().execute_with(|| {
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
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            dollar(800),
        );
        assert_eq!(
            Loans::exchange_rate(DOT).saturating_mul_int(Loans::account_collateral(DOT, ALICE)),
            dollar(200),
        );
        let borrow_snapshot = Loans::account_borrows(KSM, ALICE);
        assert_eq!(borrow_snapshot.principal, 0);
        assert_eq!(borrow_snapshot.borrow_index, Loans::borrow_index(KSM));
    })
}

#[test]
fn liquidate_borrow_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Bob deposits 200 KSM
        assert_ok!(Loans::mint(Origin::signed(BOB), KSM, dollar(200)));
        // Alice deposits 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        // Alice borrows 100 KSM
        assert_ok!(Loans::borrow(Origin::signed(ALICE), KSM, dollar(100)));
        // adjust KSM price to make ALICE generate shortfall
        MOCK_PRICE_FEEDER::set_price(KSM, 2);
        // BOB repay the KSM borrow balance and get DOT from ALICE
        assert_ok!(Loans::liquidate_borrow(
            Origin::signed(BOB),
            ALICE,
            KSM,
            dollar(50),
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
            dollar(800),
        );
        assert_eq!(
            Loans::exchange_rate(DOT).saturating_mul_int(Loans::account_collateral(DOT, ALICE)),
            88888888888888888889,
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(KSM, &ALICE),
            dollar(1100),
        );
        assert_eq!(Loans::account_borrows(KSM, ALICE).principal, dollar(50));
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(KSM, &BOB),
            dollar(750)
        );
        assert_eq!(
            Loans::exchange_rate(DOT).saturating_mul_int(Loans::account_collateral(DOT, BOB)),
            111111111111111111111,
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

#[test]
fn interest_rate_model_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 200 DOT and borrow 100 DOT
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        assert_ok!(Loans::borrow(Origin::signed(ALICE), DOT, dollar(100)));

        let total_cash = dollar(200) - dollar(100);
        let total_supply = calc_collateral_amount(dollar(200), Loans::exchange_rate(DOT)).unwrap();
        assert_eq!(Loans::total_supply(DOT), total_supply);

        let multiplier_per_year = Multiplier::saturating_from_rational(1, 10);
        let multiplier_per_block = multiplier_per_year
            .checked_div(&Rate::saturating_from_integer(BLOCK_PER_YEAR))
            .unwrap();
        assert_eq!(
            multiplier_per_block,
            Loans::currency_interest_model(DOT).multiplier_per_block
        );
        assert_eq!(multiplier_per_block, Rate::from_inner(19025875190));

        let borrow_snapshot = Loans::account_borrows(DOT, ALICE);
        assert_eq!(borrow_snapshot.principal, dollar(100));
        assert_eq!(borrow_snapshot.borrow_index, Rate::one());

        let base_rate_per_year = Rate::saturating_from_rational(2, 100);
        let base_rate_per_block = base_rate_per_year
            .checked_div(&Rate::saturating_from_integer(BLOCK_PER_YEAR))
            .unwrap();
        assert_eq!(base_rate_per_block, Rate::from_inner(3805175038));

        let mut borrow_index = Rate::one();
        let mut total_borrows = borrow_snapshot.principal;
        let mut total_reserves: u128 = 0;

        // Finalized block from 1 to 49
        for i in 2..50 {
            run_to_block(i);
            // utilizationRatio = totalBorrows / (totalCash + totalBorrows)
            let util_ratio = Ratio::from_rational(total_borrows, total_cash + total_borrows);
            assert_eq!(Loans::utilization_ratio(DOT), util_ratio);

            let borrow_rate_per_block =
                multiplier_per_block.saturating_mul(util_ratio.into()) + base_rate_per_block;
            assert_eq!(borrow_rate_per_block, Rate::from_inner(13318112633));
            let interest_accumulated = borrow_rate_per_block.saturating_mul_int(total_borrows);
            total_borrows = interest_accumulated + total_borrows;
            assert_eq!(Loans::total_borrows(DOT), total_borrows);
            total_reserves =
                Loans::reserve_factor(DOT).mul_floor(interest_accumulated) + total_reserves;
            assert_eq!(Loans::total_reserves(DOT), total_reserves);

            // exchangeRate = (totalCash + totalBorrows - totalReserves) / totalSupply
            assert_eq!(
                Loans::exchange_rate(DOT).into_inner(),
                (total_cash + total_borrows - total_reserves) * RATE_DECIMAL / total_supply
            );

            borrow_index = borrow_index * borrow_rate_per_block + borrow_index;
            assert_eq!(Loans::borrow_index(DOT), borrow_index);
        }
        assert_eq!(total_borrows, 100000063926960645957);
        assert_eq!(total_reserves, 9589044096872);
        assert_eq!(borrow_index, Rate::from_inner(1000000639269606437));
        assert_eq!(
            Loans::exchange_rate(DOT),
            // Rate::from_inner(20000006392696064) // before reserve
            Rate::from_inner(20000005433791654)
        );

        // Calculate borrow accrued interest
        let borrow_principal = (borrow_index / borrow_snapshot.borrow_index)
            .saturating_mul_int(borrow_snapshot.principal);
        let supply_interest =
            Loans::exchange_rate(DOT).saturating_mul_int(total_supply) - dollar(200);
        // assert_eq!(supply_interest, 63926960640000); // before reserve
        assert_eq!(supply_interest, 54337916540000);
        assert_eq!(borrow_principal, 100000063926960643700);
        assert_eq!(total_borrows / 10000, borrow_principal / 10000);
        assert_eq!(
            (total_borrows - dollar(100) - total_reserves) / 10000,
            supply_interest / 10000
        );
    })
}

#[test]
fn add_reserves_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Add 100 DOT reserves
        assert_ok!(Loans::add_reserves(Origin::root(), ALICE, DOT, dollar(100)));

        assert_eq!(Loans::total_reserves(DOT), dollar(100));
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &Loans::account_id()),
            dollar(100),
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            dollar(900),
        );
    })
}

#[test]
fn reduce_reserves_works() {
    ExtBuilder::default().build().execute_with(|| {
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
            <Runtime as Config>::Currency::free_balance(DOT, &Loans::account_id()),
            dollar(80),
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            dollar(920),
        );
    })
}
