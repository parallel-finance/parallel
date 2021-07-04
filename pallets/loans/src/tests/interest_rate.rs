use crate::{
    mock::{Loans, Origin, Runtime, ALICE, DOT},
    tests::{million_dollar, process_block, run_to_block, ExtBuilder},
    InterestRateModel, Markets,
};
use frame_support::assert_ok;

use primitives::{Rate, Ratio, SECONDS_PER_YEAR};
use sp_runtime::{
    traits::{CheckedDiv, One, Saturating, Zero},
    FixedPointNumber,
};

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

        let borrow_snapshot = Loans::account_borrows(DOT, ALICE);
        assert_eq!(borrow_snapshot.principal, million_dollar(100));
        assert_eq!(borrow_snapshot.borrow_index, Rate::one());

        let base_rate = Rate::saturating_from_rational(2, 100);
        let jump_rate = Rate::saturating_from_rational(10, 100);
        // let full_rate = Rate::saturating_from_rational(32, 100);
        let jump_utilization = Ratio::from_percent(80);

        let mut borrow_index = Rate::one();
        let mut total_borrows = borrow_snapshot.principal;
        let mut total_reserves: u128 = 0;

        // Finalized block from 1 to 50
        process_block(1);
        for i in 2..50 {
            process_block(i);
            // utilizationRatio = totalBorrows / (totalCash + totalBorrows)
            let util_ratio = Ratio::from_rational(total_borrows, total_cash + total_borrows);
            assert_eq!(Loans::utilization_ratio(DOT), util_ratio);

            let delta_time = 6;
            let borrow_rate =
                (jump_rate - base_rate) * util_ratio.into() / jump_utilization.into() + base_rate;
            let interest_accumulated: u128 = borrow_rate
                .saturating_mul_int(total_borrows)
                .saturating_mul(delta_time)
                .checked_div(SECONDS_PER_YEAR.into())
                .unwrap();
            total_borrows = interest_accumulated + total_borrows;
            assert_eq!(Loans::total_borrows(DOT), total_borrows);
            total_reserves = Markets::<Runtime>::get(&DOT)
                .unwrap()
                .reserve_factor
                .mul_floor(interest_accumulated)
                + total_reserves;
            assert_eq!(Loans::total_reserves(DOT), total_reserves);

            // exchangeRate = (totalCash + totalBorrows - totalReserves) / totalSupply
            assert_eq!(
                Loans::exchange_rate(DOT).into_inner(),
                (total_cash + total_borrows - total_reserves) * rate_decimal / total_supply
            );
            let numerator = borrow_index
                .saturating_mul(borrow_rate)
                .saturating_mul(delta_time.into())
                .checked_div(&Rate::saturating_from_integer(SECONDS_PER_YEAR))
                .unwrap();
            borrow_index = numerator + borrow_index;
            assert_eq!(Loans::borrow_index(DOT), borrow_index);
        }
        assert_eq!(total_borrows, 100000063926960646826);
        assert_eq!(total_reserves, 9589044097001);
        assert_eq!(borrow_index, Rate::from_inner(1000000639269606444));
        assert_eq!(
            Loans::exchange_rate(DOT),
            Rate::from_inner(20000005433791654)
        );

        // Calculate borrow accrued interest
        let borrow_principal = (borrow_index / borrow_snapshot.borrow_index)
            .saturating_mul_int(borrow_snapshot.principal);
        let supply_interest =
            Loans::exchange_rate(DOT).saturating_mul_int(total_supply) - million_dollar(200);
        assert_eq!(supply_interest, 54337916540000);
        assert_eq!(borrow_principal, 100000063926960644400);
        assert_eq!(total_borrows / 10000, borrow_principal / 10000);
        assert_eq!(
            (total_borrows - million_dollar(100) - total_reserves) / 10000,
            supply_interest / 10000
        );
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

        run_to_block(3);

        // block 3
        assert_eq!(Loans::utilization_ratio(DOT), Ratio::from_percent(50));
        assert_eq!(Loans::total_borrows(DOT), 100000001331811263318);
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
        let error_model = InterestRateModel::new_jump_model(
            Rate::zero(),
            Rate::one(),
            Rate::zero(),
            Ratio::from_percent(0),
        );

        Loans::mutate_market(&DOT, |market| {
            market.rate_model = error_model;
        })
        .unwrap();
        run_to_block(3);

        // block 3
        // No storage has been changed
        assert_eq!(Loans::utilization_ratio(DOT), Ratio::from_percent(0));
        assert_eq!(Loans::total_borrows(DOT), million_dollar(100));
        assert_eq!(Loans::total_reserves(DOT), 0);
        assert_eq!(Loans::exchange_rate(DOT).into_inner(), 20000000000000000);
        assert_eq!(Loans::borrow_index(DOT), Rate::one());
    })
}
