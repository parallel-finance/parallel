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

//! Unit tests for the prices pallet.

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::{traits::BadOrigin, FixedPointNumber};

#[test]
fn get_price_from_oracle() {
    new_test_ext().execute_with(|| {
        // currency exist
        assert_eq!(
            Prices::get_price(&DOT),
            Some((Price::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );

        // currency not exist
        assert_eq!(Prices::get_price(&SKSM), None);
    });
}

#[test]
fn set_price_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            Prices::get_price(&DOT),
            Some((Price::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );
        // set DOT price
        assert_ok!(Prices::set_price(
            Origin::signed(1),
            DOT,
            Price::saturating_from_integer(99)
        ));
        assert_eq!(
            Prices::get_price(&DOT),
            Some((Price::from_inner(9_900_000_000 * PRICE_ONE), 0))
        );
        assert_ok!(Prices::set_price(
            Origin::signed(1),
            KSM,
            Price::saturating_from_integer(1)
        ));
        assert_eq!(
            Prices::get_emergency_price(&KSM),
            Some((1_000_000.into(), 0))
        );
    });
}

#[test]
fn reset_price_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            Prices::get_price(&DOT),
            Some((Price::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );
        // set DOT price
        EmergencyPrice::<Test>::insert(DOT, Price::saturating_from_integer(99));
        assert_eq!(
            Prices::get_price(&DOT),
            Some((Price::from_inner(9_900_000_000 * PRICE_ONE), 0))
        );

        // reset DOT price
        EmergencyPrice::<Test>::remove(DOT);
        assert_eq!(
            Prices::get_price(&DOT),
            Some((Price::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );
    });
}

#[test]
fn set_price_call_work() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // set emergency price from 100 to 90
        assert_eq!(
            Prices::get_price(&DOT),
            Some((FixedU128::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );
        assert_noop!(
            Prices::set_price(Origin::signed(2), DOT, Price::saturating_from_integer(100),),
            BadOrigin
        );
        assert_ok!(Prices::set_price(
            Origin::signed(1),
            DOT,
            Price::saturating_from_integer(90),
        ));
        assert_eq!(
            Prices::get_price(&DOT),
            Some((FixedU128::from_inner(9_000_000_000 * PRICE_ONE), 0))
        );

        // check the event
        let set_price_event = Event::Prices(crate::Event::SetPrice(
            DOT,
            Price::saturating_from_integer(90),
        ));
        assert!(System::events()
            .iter()
            .any(|record| record.event == set_price_event));
        assert_eq!(
            Prices::set_price(Origin::signed(1), DOT, Price::saturating_from_integer(90),),
            Ok(().into())
        );
    });
}

#[test]
fn reset_price_call_work() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // set emergency price from 100 to 90
        assert_eq!(
            Prices::get_price(&DOT),
            Some((FixedU128::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );
        assert_ok!(Prices::set_price(
            Origin::signed(1),
            DOT,
            Price::saturating_from_integer(90),
        ));
        assert_eq!(
            Prices::get_price(&DOT),
            Some((FixedU128::from_inner(9_000_000_000 * PRICE_ONE), 0))
        );

        // try reset price
        assert_noop!(Prices::reset_price(Origin::signed(2), DOT), BadOrigin);
        assert_ok!(Prices::reset_price(Origin::signed(1), DOT));

        // price need to be 100 after reset_price
        assert_eq!(
            Prices::get_price(&DOT),
            Some((FixedU128::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );

        // check the event
        let reset_price_event = Event::Prices(crate::Event::ResetPrice(DOT));
        assert!(System::events()
            .iter()
            .any(|record| record.event == reset_price_event));
        assert_eq!(Prices::reset_price(Origin::signed(1), DOT), Ok(().into()));
    });
}

#[test]
fn get_liquid_price_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            Prices::get_price(&DOT),
            Some((Price::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );

        assert_eq!(
            Prices::get_price(&SDOT),
            LiquidStakingExchangeRateProvider::get_exchange_rate(&SDOT)
                .unwrap()
                .checked_mul_int(10_000_000_000 * PRICE_ONE)
                .map(|i| (Price::from_inner(i), 0))
        );
    });
}

#[test]
fn get_ctoken_price_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            Prices::get_price(&DOT),
            Some((Price::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );

        assert_eq!(
            Prices::get_price(&CDOT_7_14),
            Some((Price::from_inner(6666666666_666666660000000000), 0))
        );
    });
}

#[test]
fn get_lp_ctoken_price_work() {
    new_test_ext().execute_with(|| {
        DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (CDOT_7_14, DOT),
            (300 * PRICE_ONE, 100 * PRICE_ONE), //3:1
            ALICE,
            LP_DOT_CDOT_7_14,
        )
        .unwrap();

        assert_eq!(
            Prices::get_price(&LP_DOT_CDOT_7_14),
            Some((Price::from_inner(23094010767_585030803573000000), 0)) //23094010767
        );
    });
}

#[test]
fn get_lp_ctoken_price_with_another_exchange_rate_work() {
    new_test_ext().execute_with(|| {
        DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (CDOT_7_14, DOT),
            (200 * PRICE_ONE, 100 * PRICE_ONE), //2:1
            ALICE,
            LP_DOT_CDOT_7_14,
        )
        .unwrap();

        assert_eq!(
            Prices::get_price(&LP_DOT_CDOT_7_14),
            Some((Price::from_inner(18856180831_641268887810000000), 0)) //18856180831
        );
    });
}

#[test]
fn get_lp_ctoken_no_op_price_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            Prices::get_no_op(&DOT),
            Some(primitives::TimeStampedPrice {
                value: Price::saturating_from_integer(100),
                timestamp: 0
            })
        );

        DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (CDOT_7_14, DOT),
            (300 * PRICE_ONE, 100 * PRICE_ONE),
            ALICE,
            LP_DOT_CDOT_7_14,
        )
        .unwrap();

        assert_eq!(
            Prices::get_no_op(&LP_DOT_CDOT_7_14),
            Some(primitives::TimeStampedPrice {
                value: Price::from_inner(23094_010767585030803573), //230.94*(2^(12-10) since lp decimal is 12
                timestamp: 0
            })
        );
    });
}
