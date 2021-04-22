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
use primitives::{CurrencyId, BLOCK_PER_YEAR, RATE_DECIMAL, TOKEN_DECIMAL};
use sp_runtime::{traits::Saturating, Perbill};

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
fn update_jump_rate_model_works() {
    ExtBuilder::default().build().execute_with(|| {
        let base_rate_per_year = Rate::saturating_from_rational(2, 100);
        let multiplier_per_year = Rate::saturating_from_rational(1, 10);
        let jump_multiplier_per_year = Rate::saturating_from_rational(11, 10);
        assert_ok!(Loans::init_jump_rate_model(
            base_rate_per_year,
            multiplier_per_year,
            jump_multiplier_per_year,
        ));
        assert_eq!(
            BaseRatePerBlock::<Runtime>::get(),
            base_rate_per_year.saturating_mul(Perbill::from_rational(1, BLOCK_PER_YEAR).into())
        );
        assert_eq!(
            MultiplierPerBlock::<Runtime>::get(),
            multiplier_per_year.saturating_mul(Perbill::from_rational(1, BLOCK_PER_YEAR).into())
        );
        assert_eq!(
            JumpMultiplierPerBlock::<Runtime>::get(),
            jump_multiplier_per_year
                .saturating_mul(Perbill::from_rational(1, BLOCK_PER_YEAR).into())
        );
    });
}

#[test]
fn update_borrow_rate_works() {
    ExtBuilder::default().build().execute_with(|| {
        // normal rate
        let mut cash: u128 = dollar(5);
        let borrows: u128 = dollar(10);
        let reserves: u128 = 0;
        assert_ok!(Loans::update_borrow_rate(DOT, cash, borrows, reserves));
        let util = Loans::calc_utilization_ratio(cash, borrows, reserves).unwrap();
        let multiplier_per_block = MultiplierPerBlock::<Runtime>::get();
        let base_rate_per_block = BaseRatePerBlock::<Runtime>::get();
        let kink = Kink::<Runtime>::get();
        let jump_multiplier_per_block = JumpMultiplierPerBlock::<Runtime>::get();
        assert_eq!(
            multiplier_per_block.saturating_mul(util.into()) + base_rate_per_block,
            BorrowRate::<Runtime>::get(DOT),
        );

        // jump rate
        cash = dollar(1);
        assert_ok!(Loans::update_borrow_rate(DOT, cash, borrows, reserves));
        let util = Loans::calc_utilization_ratio(cash, borrows, reserves).unwrap();
        let normal_rate = multiplier_per_block.saturating_mul(kink.into()) + base_rate_per_block;
        let excess_util = util.saturating_sub(kink);
        assert_eq!(
            jump_multiplier_per_block.saturating_mul(excess_util.into()) + normal_rate,
            BorrowRate::<Runtime>::get(DOT),
        );
    });
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
        // Alice borrow 100 KSM
        assert_ok!(Loans::borrow(Origin::signed(ALICE), KSM, dollar(100)));
        // Alice repay all borrow balance
        assert_ok!(Loans::repay_borrow_all(Origin::signed(ALICE), KSM));

        // DOT: cash - deposit +  = 1000 - 200 = 800
        // DOT collateral: deposit = 200
        // KSM: cash + borrow - repay = 1000 + 100 - 100 = 1000
        // KSM borrow balance: borrow - repay = 100 - 100 = 0
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
        let multiplier_per_block =
            multiplier_per_year.saturating_mul(Perbill::from_rational(1, BLOCK_PER_YEAR).into());
        assert_eq!(multiplier_per_block, Loans::multipler_per_block());

        let borrow_snapshot = Loans::account_borrows(DOT, ALICE);
        assert_eq!(borrow_snapshot.principal, dollar(100));
        assert_eq!(borrow_snapshot.borrow_index, Rate::one());

        let base_rate_per_year = Rate::saturating_from_rational(2, 100);
        let base_rate_per_block =
            base_rate_per_year.saturating_mul(Perbill::from_rational(1, BLOCK_PER_YEAR).into());
        let mut borrow_index = Rate::one();
        let mut total_borrows = borrow_snapshot.principal;
        let total_reserves = 0;

        // Finalized block from 1 to 49
        for i in 2..50 {
            run_to_block(i);
            // utilizationRatio = totalBorrows / (totalCash + totalBorrows)
            let util_ratio = Ratio::from_rational(total_borrows, total_cash + total_borrows);
            assert_eq!(Loans::utilization_ratio(DOT), util_ratio);

            let borrow_rate_per_block =
                multiplier_per_block.saturating_mul(util_ratio.into()) + base_rate_per_block;
            total_borrows = borrow_rate_per_block.saturating_mul_int(total_borrows) + total_borrows;
            assert_eq!(Loans::total_borrows(DOT), total_borrows);

            // exchangeRate = (totalCash + totalBorrows - totalReserves) / totalSupply
            assert_eq!(
                Loans::exchange_rate(DOT).into_inner(),
                (total_cash + total_borrows - total_reserves) * RATE_DECIMAL / total_supply
            );

            borrow_index = borrow_index * borrow_rate_per_block + borrow_index;
            assert_eq!(Loans::borrow_index(DOT), borrow_index);
        }

        // Calculate borrow accrued interest
        let borrow_principal = (borrow_index / borrow_snapshot.borrow_index)
            .saturating_mul_int(borrow_snapshot.principal);
        let supply_interest =
            Loans::exchange_rate(DOT).saturating_mul_int(total_supply) - dollar(200);
        assert_eq!(total_borrows / 10000, borrow_principal / 10000);
        assert_eq!(
            (total_borrows - dollar(100)) / 10000,
            supply_interest / 10000
        );
    })
}
