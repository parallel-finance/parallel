// Copyright 2021 Parallel Finance name: (), address: (), stakes: (), score: ()  name: (), address: (), stakes: (), score: ()  name: (), address: (), stakes: (), score: ()  Developer.
// This file is  crf: (), nf: (), epf: ()  crf: (), nf: (), epf: () part of Parallel Finance.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Unit tests for the nominee-election pallet.

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_runtime::traits::BadOrigin;

#[test]
fn set_coefficients_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(NomineeElection::coefficients(), MOCK_OLD_COEFFICIENTS);
        assert_noop!(
            NomineeElection::set_coefficients(Origin::signed(2), MOCK_NEW_COEFFICIENTS.clone()),
            BadOrigin
        );
        assert_ok!(NomineeElection::set_coefficients(
            Origin::signed(1),
            MOCK_NEW_COEFFICIENTS
        ));
        assert_eq!(NomineeElection::coefficients(), MOCK_NEW_COEFFICIENTS);
    });
}

#[test]
fn set_validators_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(NomineeElection::validators(), vec![]);
        assert_noop!(
            NomineeElection::set_validators(Origin::signed(1), vec![]),
            Error::<Runtime>::BadValidatorsFeeder
        );
        assert_noop!(
            NomineeElection::set_validators(Origin::signed(6), vec![]),
            Error::<Runtime>::NoEmptyValidators
        );
        assert_ok!(NomineeElection::add_whitelist_validator(
            Origin::signed(2),
            3
        ),);
        assert_ok!(NomineeElection::set_validators(
            Origin::signed(6),
            vec![MOCK_VALIDATOR_THREE]
        ));
        assert_eq!(NomineeElection::validators(), vec![]);
        assert_ok!(NomineeElection::remove_whitelisted_validator(
            Origin::signed(2),
            3
        ),);
        assert_ok!(NomineeElection::set_validators(
            Origin::signed(6),
            vec![MOCK_VALIDATOR_THREE]
        ));
        assert_eq!(NomineeElection::validators(), vec![MOCK_VALIDATOR_THREE]);
        assert_noop!(
            NomineeElection::set_validators(
                Origin::signed(6),
                vec![MOCK_VALIDATOR_THREE, MOCK_VALIDATOR_FOUR]
            ),
            Error::<Runtime>::MaxValidatorsExceeded
        );
    });
}

#[test]
fn add_whitelist_validator_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(NomineeElection::whitelisted_validators(), vec![]);
        assert_noop!(
            NomineeElection::add_whitelist_validator(Origin::signed(1), 1),
            BadOrigin
        );
        assert_ok!(NomineeElection::add_whitelist_validator(
            Origin::signed(2),
            1
        ),);
        assert_eq!(NomineeElection::whitelisted_validators(), vec![1]);
        assert_noop!(
            NomineeElection::add_whitelist_validator(Origin::signed(2), 2),
            Error::<Runtime>::MaxValidatorsExceeded
        );
    });
}

#[test]
fn remove_whitelisted_validator_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(NomineeElection::whitelisted_validators(), vec![]);
        assert_noop!(
            NomineeElection::remove_whitelisted_validator(Origin::signed(1), 1),
            BadOrigin
        );
        assert_ok!(NomineeElection::add_whitelist_validator(
            Origin::signed(2),
            1
        ));
        assert_eq!(NomineeElection::whitelisted_validators(), vec![1]);
        assert_ok!(NomineeElection::remove_whitelisted_validator(
            Origin::signed(2),
            1
        ));
        assert_noop!(
            NomineeElection::remove_whitelisted_validator(Origin::signed(2), 1),
            Error::<Runtime>::ValidatorNotFound
        );
        assert_eq!(NomineeElection::whitelisted_validators(), vec![]);
    });
}

#[test]
fn reset_whitelisted_validators_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(NomineeElection::whitelisted_validators(), vec![]);
        assert_noop!(
            NomineeElection::reset_whitelisted_validators(Origin::signed(1)),
            BadOrigin
        );
        assert_ok!(NomineeElection::add_whitelist_validator(
            Origin::signed(2),
            1
        ));
        assert_eq!(NomineeElection::whitelisted_validators(), vec![1]);
        assert_ok!(NomineeElection::reset_whitelisted_validators(
            Origin::signed(2)
        ));
        assert_eq!(NomineeElection::whitelisted_validators(), vec![]);
    });
}
