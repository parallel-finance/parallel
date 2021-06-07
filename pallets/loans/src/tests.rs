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
use primitives::SECONDS_PER_YEAR;
use sp_runtime::traits::{CheckedDiv, One, Saturating};
use sp_runtime::{FixedU128, Permill};

use super::*;

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
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, million_dollar(100)));

        // DOT collateral: deposit = 100
        // DOT: cash - deposit = 1000 - 100 = 900
        assert_eq!(
            Loans::exchange_rate(DOT).saturating_mul_int(Loans::account_collateral(DOT, ALICE)),
            million_dollar(100)
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            million_dollar(900),
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
            Error::<Runtime>::Underflow,
        );
    })
}

#[test]
fn redeem_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 100 DOT
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, million_dollar(100)));
        // Redeem 20 DOT
        assert_ok!(Loans::redeem(
            Origin::signed(ALICE),
            DOT,
            million_dollar(20)
        ));

        // DOT collateral: deposit - redeem = 100 - 20 = 80
        // DOT: cash - deposit + redeem = 1000 - 100 + 20 = 920
        assert_eq!(
            Loans::exchange_rate(DOT).saturating_mul_int(Loans::account_collateral(DOT, ALICE)),
            million_dollar(80)
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            million_dollar(920),
        );
    })
}

#[test]
fn redeem_all_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 100 DOT
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, million_dollar(100)));
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
            million_dollar(1000),
        );
    })
}

#[test]
fn borrow_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, million_dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        // Borrow 100 DOT
        assert_ok!(Loans::borrow(
            Origin::signed(ALICE),
            DOT,
            million_dollar(100)
        ));

        // DOT collateral: deposit = 200
        // DOT borrow balance: borrow = 100
        // DOT: cash - deposit + borrow = 1000 - 200 + 100 = 900
        assert_eq!(
            Loans::exchange_rate(DOT).saturating_mul_int(Loans::account_collateral(DOT, ALICE)),
            million_dollar(200)
        );
        let borrow_snapshot = Loans::account_borrows(DOT, ALICE);
        assert_eq!(borrow_snapshot.principal, million_dollar(100));
        assert_eq!(borrow_snapshot.borrow_index, Loans::borrow_index(DOT));
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            million_dollar(900),
        );
    })
}

#[test]
fn repay_borrow_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, million_dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        // Borrow 100 DOT
        assert_ok!(Loans::borrow(
            Origin::signed(ALICE),
            DOT,
            million_dollar(100)
        ));
        // Repay 30 DOT
        assert_ok!(Loans::repay_borrow(
            Origin::signed(ALICE),
            DOT,
            million_dollar(30)
        ));

        // DOT collateral: deposit = 200
        // DOT borrow balance: borrow - repay = 100 - 30 = 70
        // DOT: cash - deposit + borrow - repay = 1000 - 200 + 100 - 30 = 870
        assert_eq!(
            Loans::exchange_rate(DOT).saturating_mul_int(Loans::account_collateral(DOT, ALICE)),
            million_dollar(200)
        );
        let borrow_snapshot = Loans::account_borrows(DOT, ALICE);
        assert_eq!(borrow_snapshot.principal, million_dollar(70));
        assert_eq!(borrow_snapshot.borrow_index, Loans::borrow_index(DOT));
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            million_dollar(870),
        );
    })
}

#[test]
fn repay_borrow_all_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Bob deposits 200 KSM
        assert_ok!(Loans::mint(Origin::signed(BOB), KSM, million_dollar(200)));
        // Alice deposit 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, million_dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        // Alice borrow 50 KSM
        assert_ok!(Loans::borrow(
            Origin::signed(ALICE),
            KSM,
            million_dollar(50)
        ));
        // Alice repay all borrow balance
        assert_ok!(Loans::repay_borrow_all(Origin::signed(ALICE), KSM));

        // DOT: cash - deposit +  = 1000 - 200 = 800
        // DOT collateral: deposit = 200
        // KSM: cash + borrow - repay = 1000 + 50 - 50 = 1000
        // KSM borrow balance: borrow - repay = 50 - 50 = 0
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            million_dollar(800),
        );
        assert_eq!(
            Loans::exchange_rate(DOT).saturating_mul_int(Loans::account_collateral(DOT, ALICE)),
            million_dollar(200),
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
        assert_ok!(Loans::mint(Origin::signed(BOB), KSM, million_dollar(200)));
        // Alice deposits 200 DOT as collateral
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, million_dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        // Alice borrows 100 KSM
        assert_ok!(Loans::borrow(
            Origin::signed(ALICE),
            KSM,
            million_dollar(100)
        ));
        // adjust KSM price to make ALICE generate shortfall
        MOCK_PRICE_FEEDER::set_price(KSM, 2.into());
        // BOB repay the KSM borrow balance and get DOT from ALICE
        assert_ok!(Loans::liquidate_borrow(
            Origin::signed(BOB),
            ALICE,
            KSM,
            million_dollar(50),
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
            million_dollar(800),
        );
        assert_eq!(
            Loans::exchange_rate(DOT).saturating_mul_int(Loans::account_collateral(DOT, ALICE)),
            88888888888888888889,
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(KSM, &ALICE),
            million_dollar(1100),
        );
        assert_eq!(
            Loans::account_borrows(KSM, ALICE).principal,
            million_dollar(50)
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(KSM, &BOB),
            million_dollar(750)
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
        let rate_decimal: u128 = 1_000_000_000_000_000_000;
        // Deposit 200 DOT and borrow 100 DOT
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, million_dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        assert_ok!(Loans::borrow(
            Origin::signed(ALICE),
            DOT,
            million_dollar(100)
        ));

        let total_cash = million_dollar(200) - million_dollar(100);
        let total_supply =
            Loans::calc_collateral_amount(million_dollar(200), Loans::exchange_rate(DOT)).unwrap();
        assert_eq!(Loans::total_supply(DOT), total_supply);

        let multiplier_per_year = Multiplier::saturating_from_rational(1, 10);
        let block_fraction = Rate::saturating_from_rational(6, SECONDS_PER_YEAR);
        let multiplier_per_block = multiplier_per_year.checked_mul(&block_fraction).unwrap();
        assert_eq!(multiplier_per_block, Rate::from_inner(19025875190));

        let borrow_snapshot = Loans::account_borrows(DOT, ALICE);
        assert_eq!(borrow_snapshot.principal, million_dollar(100));
        assert_eq!(borrow_snapshot.borrow_index, Rate::one());

        let base_rate_per_year = Rate::saturating_from_rational(2, 100);
        let base_rate_per_block = base_rate_per_year.checked_mul(&block_fraction).unwrap();
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

            let borrow_rate_apr =
                multiplier_per_year.saturating_mul(util_ratio.into()) + base_rate_per_year;
            let delta_time =
                <Runtime as Config>::UnixTime::get() / 1000 - Loans::last_block_timestamp();
            let fraction = Rate::saturating_from_rational(delta_time, SECONDS_PER_YEAR);
            let interest_accumulated =
                fraction.saturating_mul_int(borrow_rate_apr.saturating_mul_int(total_borrows));
            total_borrows = interest_accumulated + total_borrows;
            assert_eq!(Loans::total_borrows(DOT), total_borrows);
            total_reserves =
                Loans::reserve_factor(DOT).mul_floor(interest_accumulated) + total_reserves;
            assert_eq!(Loans::total_reserves(DOT), total_reserves);

            // exchangeRate = (totalCash + totalBorrows - totalReserves) / totalSupply
            assert_eq!(
                Loans::exchange_rate(DOT).into_inner(),
                (total_cash + total_borrows - total_reserves) * rate_decimal / total_supply
            );
            borrow_index = borrow_index
                .saturating_mul(borrow_rate_apr)
                .saturating_mul(fraction)
                + borrow_index;
            assert_eq!(Loans::borrow_index(DOT), borrow_index);
        }
        assert_eq!(total_borrows, 100000063926960646629);
        assert_eq!(total_reserves, 9589044096972);
        assert_eq!(borrow_index, Rate::from_inner(1000000639269606443));
        assert_eq!(
            Loans::exchange_rate(DOT),
            // Rate::from_inner(20000006392696064) // before reserve
            Rate::from_inner(20000005433791654)
        );

        // Calculate borrow accrued interest
        let borrow_principal = (borrow_index / borrow_snapshot.borrow_index)
            .saturating_mul_int(borrow_snapshot.principal);
        let supply_interest =
            Loans::exchange_rate(DOT).saturating_mul_int(total_supply) - million_dollar(200);
        // assert_eq!(supply_interest, 63926960640000); // before reserve
        assert_eq!(supply_interest, 54337916540000);
        assert_eq!(borrow_principal, 100000063926960644300);
        assert_eq!(total_borrows / 10000, borrow_principal / 10000);
        assert_eq!(
            (total_borrows - million_dollar(100) - total_reserves) / 10000,
            supply_interest / 10000
        );
    })
}

#[test]
fn add_reserves_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Add 100 DOT reserves
        assert_ok!(Loans::add_reserves(
            Origin::root(),
            ALICE,
            DOT,
            million_dollar(100)
        ));

        assert_eq!(Loans::total_reserves(DOT), million_dollar(100));
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &Loans::account_id()),
            million_dollar(100),
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            million_dollar(900),
        );
    })
}

#[test]
fn reduce_reserves_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Add 100 DOT reserves
        assert_ok!(Loans::add_reserves(
            Origin::root(),
            ALICE,
            DOT,
            million_dollar(100)
        ));

        // Reduce 20 DOT reserves
        assert_ok!(Loans::reduce_reserves(
            Origin::root(),
            ALICE,
            DOT,
            million_dollar(20)
        ));

        assert_eq!(Loans::total_reserves(DOT), million_dollar(80));
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &Loans::account_id()),
            million_dollar(80),
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            million_dollar(920),
        );
    })
}

#[test]
fn ratio_and_rate_works() {
    ExtBuilder::default().build().execute_with(|| {
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
fn set_rate_model_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Check genesis rate model
        assert_eq!(
            Loans::currency_interest_model(DOT),
            rate::InterestRateModel {
                base_rate: Rate::saturating_from_rational(2, 100).into(),
                kink_rate: Rate::saturating_from_rational(10, 100).into(),
                full_rate: Rate::saturating_from_rational(32, 100).into(),
                kink_utilization: Ratio::from_percent(80).into(),
            }
        );
        // Set new rate model
        assert_ok!(Loans::set_rate_model(
            Origin::root(),
            DOT,
            rate::InterestRateModel {
                base_rate: Rate::saturating_from_rational(5, 100).into(),
                kink_rate: Rate::saturating_from_rational(15, 100).into(),
                full_rate: Rate::saturating_from_rational(35, 100).into(),
                kink_utilization: Ratio::from_percent(80).into(),
            }
        ));
        assert_eq!(
            Loans::currency_interest_model(DOT),
            rate::InterestRateModel {
                base_rate: Rate::saturating_from_rational(5, 100).into(),
                kink_rate: Rate::saturating_from_rational(15, 100).into(),
                full_rate: Rate::saturating_from_rational(35, 100).into(),
                kink_utilization: Ratio::from_percent(80).into(),
            }
        );
    })
}

#[test]
fn set_rate_model_failed_by_error_param() {
    ExtBuilder::default().build().execute_with(|| {
        // Invalid base_rate
        assert_noop!(
            Loans::set_rate_model(
                Origin::root(),
                DOT,
                rate::InterestRateModel {
                    base_rate: Rate::saturating_from_rational(36, 100).into(),
                    kink_rate: Rate::saturating_from_rational(15, 100).into(),
                    full_rate: Rate::saturating_from_rational(35, 100).into(),
                    kink_utilization: Ratio::from_percent(80).into(),
                }
            ),
            Error::<Runtime>::InvalidRateModelParam
        );
        // Invalid kink_rate
        assert_noop!(
            Loans::set_rate_model(
                Origin::root(),
                DOT,
                rate::InterestRateModel {
                    base_rate: Rate::saturating_from_rational(5, 100).into(),
                    kink_rate: Rate::saturating_from_rational(36, 100).into(),
                    full_rate: Rate::saturating_from_rational(37, 100).into(),
                    kink_utilization: Ratio::from_percent(80).into(),
                }
            ),
            Error::<Runtime>::InvalidRateModelParam
        );
        // Invalid full_rate
        assert_noop!(
            Loans::set_rate_model(
                Origin::root(),
                DOT,
                rate::InterestRateModel {
                    base_rate: Rate::saturating_from_rational(5, 100).into(),
                    kink_rate: Rate::saturating_from_rational(15, 100).into(),
                    full_rate: Rate::saturating_from_rational(37, 100).into(),
                    kink_utilization: Ratio::from_percent(80).into(),
                }
            ),
            Error::<Runtime>::InvalidRateModelParam
        );
        // base_rate greater than kink_rate
        assert_noop!(
            Loans::set_rate_model(
                Origin::root(),
                DOT,
                rate::InterestRateModel {
                    base_rate: Rate::saturating_from_rational(10, 100).into(),
                    kink_rate: Rate::saturating_from_rational(9, 100).into(),
                    full_rate: Rate::saturating_from_rational(14, 100).into(),
                    kink_utilization: Ratio::from_percent(80).into(),
                }
            ),
            Error::<Runtime>::InvalidRateModelParam
        );
        // kink_rate greater than full_rate
        assert_noop!(
            Loans::set_rate_model(
                Origin::root(),
                DOT,
                rate::InterestRateModel {
                    base_rate: Rate::saturating_from_rational(5, 100).into(),
                    kink_rate: Rate::saturating_from_rational(15, 100).into(),
                    full_rate: Rate::saturating_from_rational(14, 100).into(),
                    kink_utilization: Ratio::from_percent(80).into(),
                }
            ),
            Error::<Runtime>::InvalidRateModelParam
        );
    })
}

#[test]
fn update_exchange_rate_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Initialize value of exchange rate is 0.02
        assert_eq!(
            Loans::exchange_rate(DOT),
            Rate::saturating_from_rational(2, 100)
        );

        // total_supply = 0
        TotalSupply::<Runtime>::insert(DOT, 0);
        assert_ok!(Loans::update_exchange_rate(DOT));
        assert_eq!(
            Loans::exchange_rate(DOT),
            Rate::saturating_from_rational(2, 100)
        );

        // total_cash + total_borrows - total_reverse / total_supply
        // 10 + 5 - 1 / 500
        // total_cash = 10, total_supply = 500
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, million_dollar(10)));
        TotalBorrows::<Runtime>::insert(DOT, million_dollar(5));
        TotalReserves::<Runtime>::insert(DOT, million_dollar(1));
        assert_ok!(Loans::update_exchange_rate(DOT));
        assert_eq!(
            Loans::exchange_rate(DOT),
            Rate::saturating_from_rational(14, 500)
        );
    })
}

#[test]
fn borrow_balance_stored_works() {
    ExtBuilder::default().build().execute_with(|| {
        // snapshot.principal = 0
        AccountBorrows::<Runtime>::insert(
            DOT,
            ALICE,
            BorrowSnapshot {
                principal: 0,
                borrow_index: Rate::one(),
            },
        );
        assert_eq!(Loans::borrow_balance_stored(&ALICE, &DOT).unwrap(), 0);

        // snapshot.borrow_index = 0
        AccountBorrows::<Runtime>::insert(
            DOT,
            ALICE,
            BorrowSnapshot {
                principal: 100,
                borrow_index: Rate::zero(),
            },
        );
        assert_eq!(Loans::borrow_balance_stored(&ALICE, &DOT).unwrap(), 0);

        // borrow_index = 1.2, snapshot.borrow_index = 1, snapshot.principal = 100
        BorrowIndex::<Runtime>::insert(DOT, Rate::saturating_from_rational(12, 10));
        AccountBorrows::<Runtime>::insert(
            DOT,
            ALICE,
            BorrowSnapshot {
                principal: 100,
                borrow_index: Rate::one(),
            },
        );
        assert_eq!(Loans::borrow_balance_stored(&ALICE, &DOT).unwrap(), 120);
    })
}

#[test]
fn with_transaction_commit_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 200 DOT and borrow 100 DOT
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, million_dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        assert_ok!(Loans::borrow(
            Origin::signed(ALICE),
            DOT,
            million_dollar(100)
        ));

        // let total_cash = dollar(200) - dollar(100);
        let total_supply =
            Loans::calc_collateral_amount(million_dollar(200), Loans::exchange_rate(DOT)).unwrap();
        assert_eq!(Loans::total_supply(DOT), total_supply);

        let borrow_snapshot = Loans::account_borrows(DOT, ALICE);
        assert_eq!(borrow_snapshot.principal, million_dollar(100));
        assert_eq!(borrow_snapshot.borrow_index, Rate::one());

        // block 1
        assert_eq!(Loans::utilization_ratio(DOT), Ratio::from_percent(0));
        assert_eq!(Loans::total_borrows(DOT), million_dollar(100));
        assert_eq!(Loans::total_reserves(DOT), 0);
        assert_eq!(Loans::exchange_rate(DOT).into_inner(), 20000000000000000);
        assert_eq!(Loans::borrow_index(DOT), Rate::one());

        run_to_block(2);

        // block 2
        assert_eq!(Loans::utilization_ratio(DOT), Ratio::from_percent(50));
        assert_eq!(Loans::total_borrows(DOT), 100000001331811263314);
        assert_eq!(Loans::total_reserves(DOT), 199771689497);
        assert_eq!(Loans::exchange_rate(DOT).into_inner(), 20000000113203957);
        assert_eq!(
            Loans::borrow_index(DOT),
            Rate::from_inner(1000000013318112633)
        );
    })
}

#[test]
fn with_transaction_rollback_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Deposit 200 DOT and borrow 100 DOT
        assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, million_dollar(200)));
        assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
        assert_ok!(Loans::borrow(
            Origin::signed(ALICE),
            DOT,
            million_dollar(100)
        ));

        // let total_cash = dollar(200) - dollar(100);
        let total_supply =
            Loans::calc_collateral_amount(million_dollar(200), Loans::exchange_rate(DOT)).unwrap();
        assert_eq!(Loans::total_supply(DOT), total_supply);

        let borrow_snapshot = Loans::account_borrows(DOT, ALICE);
        assert_eq!(borrow_snapshot.principal, million_dollar(100));
        assert_eq!(borrow_snapshot.borrow_index, Rate::one());

        // block 1
        assert_eq!(Loans::utilization_ratio(DOT), Ratio::from_percent(0));
        assert_eq!(Loans::total_borrows(DOT), million_dollar(100));
        assert_eq!(Loans::total_reserves(DOT), 0);
        assert_eq!(Loans::exchange_rate(DOT).into_inner(), 20000000000000000);
        assert_eq!(Loans::borrow_index(DOT), Rate::one());

        // Set an error rate model to trigger an Error Result when accruing interest.
        let error_model = InterestRateModel {
            base_rate: Rate::zero().into(),
            kink_rate: Rate::one().into(),
            full_rate: Rate::zero().into(),
            kink_utilization: Ratio::from_percent(0),
        };
        CurrencyInterestModel::<Runtime>::insert(DOT, error_model);
        run_to_block(2);

        // block 2
        // No storage has been changed
        assert_eq!(Loans::utilization_ratio(DOT), Ratio::from_percent(0));
        assert_eq!(Loans::total_borrows(DOT), million_dollar(100));
        assert_eq!(Loans::total_reserves(DOT), 0);
        assert_eq!(Loans::exchange_rate(DOT).into_inner(), 20000000000000000);
        assert_eq!(Loans::borrow_index(DOT), Rate::one());
    })
}

#[test]
fn calc_collateral_amount_works() {
    let amount: u128 = 1000;
    let exchange_rate = Rate::saturating_from_rational(3, 10);
    assert_eq!(
        Loans::calc_collateral_amount(amount, exchange_rate).unwrap(),
        3333
    );
}

#[test]
fn get_price_works() {
    MOCK_PRICE_FEEDER::set_price(DOT, 0.into());
    assert!(Loans::get_price(&DOT).is_err());

    MOCK_PRICE_FEEDER::set_price(DOT, 2.into());
    assert_eq!(
        Loans::get_price(&DOT).unwrap(),
        Price::saturating_from_integer(2)
    );
}
