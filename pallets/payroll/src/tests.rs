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

#[test]
fn init_minting_ok() {
    new_test_ext().execute_with(|| {
        assert_eq!(Assets::balance(KSM, ALICE), dollar(1000));
        assert_eq!(Assets::balance(DOT, ALICE), dollar(1000));
        assert_eq!(Assets::balance(USDT, ALICE), dollar(1000));
        assert_eq!(Assets::balance(KSM, BOB), dollar(1000));
        assert_eq!(Assets::balance(DOT, BOB), dollar(1000));
    });
}
