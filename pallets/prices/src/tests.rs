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

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::{traits::BadOrigin, FixedPointNumber};

#[test]
fn get_price_from_oracle() {
    ExtBuilder::default().build().execute_with(|| {
        // currency exist
        assert_eq!(
            PricesPallet::get_price(&DOT),
            Some((OraclePrice::saturating_from_integer(100).into_inner(), 0))
        );

        // currency not exist
        assert_eq!(PricesPallet::get_price(&KSM), None);
    });
}

#[test]
fn set_price_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(
            PricesPallet::get_price(&DOT),
            Some((OraclePrice::saturating_from_integer(100).into_inner(), 0))
        );
        // set DOT price
        EmergencyPrice::<Runtime>::insert(DOT, OraclePrice::saturating_from_integer(99));
        assert_eq!(
            PricesPallet::get_price(&DOT),
            Some((OraclePrice::saturating_from_integer(99).into_inner(), 0))
        );
    });
}

#[test]
fn reset_price_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(
            PricesPallet::get_price(&DOT),
            Some((OraclePrice::saturating_from_integer(100).into_inner(), 0))
        );
        // set DOT price
        EmergencyPrice::<Runtime>::insert(DOT, OraclePrice::saturating_from_integer(99));
        assert_eq!(
            PricesPallet::get_price(&DOT),
            Some((OraclePrice::saturating_from_integer(99).into_inner(), 0))
        );

        // reset DOT price
        EmergencyPrice::<Runtime>::remove(DOT);
        assert_eq!(
            PricesPallet::get_price(&DOT),
            Some((OraclePrice::saturating_from_integer(100).into_inner(), 0))
        );
    });
}

#[test]
fn set_price_call_work() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        // set emergency price from 100 to 90
        assert_eq!(
            PricesPallet::get_price(&DOT),
            Some((OraclePrice::saturating_from_integer(100).into_inner(), 0))
        );
        assert_noop!(
            PricesPallet::set_price(
                Origin::signed(2),
                DOT,
                OraclePrice::saturating_from_integer(100)
            ),
            BadOrigin
        );
        assert_ok!(PricesPallet::set_price(
            Origin::signed(1),
            DOT,
            OraclePrice::saturating_from_integer(90)
        ));
        assert_eq!(
            PricesPallet::get_price(&DOT),
            Some((OraclePrice::saturating_from_integer(90).into_inner(), 0))
        );

        // check the event
        let set_price_event = Event::prices(crate::Event::SetPrice(
            DOT,
            OraclePrice::saturating_from_integer(90),
        ));
        assert!(System::events()
            .iter()
            .any(|record| record.event == set_price_event));
        assert_eq!(
            PricesPallet::set_price(
                Origin::signed(1),
                DOT,
                OraclePrice::saturating_from_integer(90)
            ),
            Ok(().into())
        );
    });
}

#[test]
fn reset_price_call_work() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        // set emergency price from 100 to 90
        assert_eq!(
            PricesPallet::get_price(&DOT),
            Some((OraclePrice::saturating_from_integer(100).into_inner(), 0))
        );
        assert_ok!(PricesPallet::set_price(
            Origin::signed(1),
            DOT,
            OraclePrice::saturating_from_integer(90)
        ));
        assert_eq!(
            PricesPallet::get_price(&DOT),
            Some((OraclePrice::saturating_from_integer(90).into_inner(), 0))
        );

        // try reset price
        assert_noop!(PricesPallet::reset_price(Origin::signed(2), DOT), BadOrigin);
        assert_ok!(PricesPallet::reset_price(Origin::signed(1), DOT));

        // price need to be 100 after reset_price
        assert_eq!(
            PricesPallet::get_price(&DOT),
            Some((OraclePrice::saturating_from_integer(100).into_inner(), 0))
        );

        // check the event
        let reset_price_event = Event::prices(crate::Event::ResetPrice(DOT));
        assert!(System::events()
            .iter()
            .any(|record| record.event == reset_price_event));
        assert_eq!(
            PricesPallet::reset_price(Origin::signed(1), DOT),
            Ok(().into())
        );
    });
}
