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

//! add tests for relay-chain transact here
//! for unit tests covering internal logic we may still maintain in pallet
//!
use crate::{kusama_test_net::*, setup::*};
use frame_support::{assert_ok, storage::with_transaction};
use primitives::AccountId;
use sp_runtime::{DispatchResult, TransactionOutcome};
use xcm_emulator::TestExt;

#[test]
/// Test liquidate_staking stake.
fn liquidate_staking_call_should_work() {
    let mut amount = ksm(10f64);
    let sovereign_sub_account: AccountId =
        hex_literal::hex!["5d199b535508990c59f411757617904ce65c905fced6878bacfbf26d3b4a1e97"]
            .into();
    Heiko::execute_with(|| {
        use heiko_runtime::{LiquidStaking, Origin};
        assert_ok!(LiquidStaking::stake(
            RuntimeOrigin::signed(AccountId::from(ALICE)),
            amount
        ));
        assert_ok!(with_transaction(
            || -> TransactionOutcome<DispatchResult> {
                assert_ok!(LiquidStaking::do_advance_era(1));
                assert_ok!(LiquidStaking::do_matching());
                TransactionOutcome::Commit(Ok(()))
            }
        ));
        let reserved_factor = LiquidStaking::reserve_factor();
        let reserved = reserved_factor.mul_floor(amount);
        let xcm_fee = 5_000_000_000 as u128;
        amount = amount - (reserved + xcm_fee);
    });

    KusamaNet::execute_with(|| {
        use kusama_runtime::Staking;
        assert_eq!(
            Staking::ledger(&sovereign_sub_account.clone()),
            Some(pallet_staking::StakingLedger {
                stash: sovereign_sub_account.clone(),
                total: amount,
                active: amount,
                unlocking: Default::default(),
                claimed_rewards: vec![]
            })
        );
    })
}
