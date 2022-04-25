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

use super::*;
use mock::*;

use frame_support::assert_ok;

#[test]
fn test_works() {
    new_test_ext().execute_with(|| {
        // Alice creates stream 100 DOT to Bob
        assert_ok!(Doracle::create_something(Origin::signed(ALICE),));

        assert_eq!(0, 1);
    });
}
