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

const PRICE_ONE: u128 = 1_000_000_000_000_000_000;

#[test]
fn get_price_from_oracle() {
    new_test_ext().execute_with(|| {
        // currency exist
        assert_eq!(
            Doracle::get_price(&DOT),
            Some((Price::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );

        // currency not exist
        assert_eq!(Doracle::get_price(&SDOT), None);
    });
}

#[test]
fn set_price_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            Doracle::get_price(&DOT),
            Some((Price::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );
        // set DOT price
        assert_ok!(Doracle::set_price(
            Origin::signed(1),
            DOT,
            Price::saturating_from_integer(99)
        ));
        assert_eq!(
            Doracle::get_price(&DOT),
            Some((Price::from_inner(9_900_000_000 * PRICE_ONE), 0))
        );
        assert_ok!(Doracle::set_price(
            Origin::signed(1),
            KSM,
            Price::saturating_from_integer(1)
        ));
        assert_eq!(
            Doracle::get_emergency_price(&KSM),
            Some((1_000_000.into(), 0))
        );
    });
}

#[test]
fn reset_price_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            Doracle::get_price(&DOT),
            Some((Price::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );
        // set DOT price
        EmergencyPrice::<Test>::insert(DOT, Price::saturating_from_integer(99));
        assert_eq!(
            Doracle::get_price(&DOT),
            Some((Price::from_inner(9_900_000_000 * PRICE_ONE), 0))
        );

        // reset DOT price
        EmergencyPrice::<Test>::remove(DOT);
        assert_eq!(
            Doracle::get_price(&DOT),
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
            Doracle::get_price(&DOT),
            Some((FixedU128::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );
        assert_noop!(
            Doracle::set_price(Origin::signed(2), DOT, Price::saturating_from_integer(100),),
            BadOrigin
        );
        assert_ok!(Doracle::set_price(
            Origin::signed(1),
            DOT,
            Price::saturating_from_integer(90),
        ));
        assert_eq!(
            Doracle::get_price(&DOT),
            Some((FixedU128::from_inner(9_000_000_000 * PRICE_ONE), 0))
        );

        // check the event
        let set_price_event = Event::Doracle(crate::Event::SetPrice(
            DOT,
            Price::saturating_from_integer(90),
        ));
        assert!(System::events()
            .iter()
            .any(|record| record.event == set_price_event));
        assert_eq!(
            Doracle::set_price(Origin::signed(1), DOT, Price::saturating_from_integer(90),),
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
            Doracle::get_price(&DOT),
            Some((FixedU128::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );
        assert_ok!(Doracle::set_price(
            Origin::signed(1),
            DOT,
            Price::saturating_from_integer(90),
        ));
        assert_eq!(
            Doracle::get_price(&DOT),
            Some((FixedU128::from_inner(9_000_000_000 * PRICE_ONE), 0))
        );

        // try reset price
        assert_noop!(Doracle::reset_price(Origin::signed(2), DOT), BadOrigin);
        assert_ok!(Doracle::reset_price(Origin::signed(1), DOT));

        // price need to be 100 after reset_price
        assert_eq!(
            Doracle::get_price(&DOT),
            Some((FixedU128::from_inner(10_000_000_000 * PRICE_ONE), 0))
        );

        // check the event
        let reset_price_event = Event::Doracle(crate::Event::ResetPrice(DOT));
        assert!(System::events()
            .iter()
            .any(|record| record.event == reset_price_event));
        assert_eq!(Doracle::reset_price(Origin::signed(1), DOT), Ok(().into()));
    });
}

#[test]
fn get_liquid_price_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            Doracle::get_price(&KSM),
            Some((Price::from_inner(500 * 1_000_000 * PRICE_ONE), 0))
        );

        assert_eq!(
            Doracle::get_price(&SKSM),
            LiquidStakingExchangeRateProvider::get_exchange_rate()
                .checked_mul_int(500 * 1_000_000 * PRICE_ONE)
                .map(|i| (Price::from_inner(i), 0))
        );
    });
}
