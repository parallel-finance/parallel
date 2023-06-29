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

//! Cross-chain transfer tests within Kusama network.

use cumulus_primitives_core::ParaId;
use frame_support::assert_ok;
use heiko_runtime::Assets;
use primitives::{tokens::*, AccountId};
use sp_runtime::traits::AccountIdConversion;
use xcm::{latest::prelude::*, VersionedMultiAssets, VersionedMultiLocation};
use xcm_emulator::TestExt;

use crate::{kusama_test_net::*, setup::*};

#[test]
fn transfer_from_relay_chain() {
    KusamaNet::execute_with(|| {
        assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
            kusama_runtime::RuntimeOrigin::signed(ALICE.into()),
            Box::new(VersionedMultiLocation::V3(X1(Parachain(2085)).into())),
            Box::new(VersionedMultiLocation::V3(
                X1(Junction::AccountId32 {
                    id: BOB,
                    network: None
                })
                .into()
            )),
            Box::new(VersionedMultiAssets::V3((Here, ksm(1f64)).into())),
            0,
        ));
    });

    Heiko::execute_with(|| {
        assert_eq!(Assets::balance(KSM, &AccountId::from(BOB)), 999_860_956_000);
        //ksm fee in heiko is 139_044_000,seems increased 50% in v0.9.24
    });
}

#[test]
fn transfer_to_relay_chain() {
    use heiko_runtime::{RuntimeOrigin, XTokens};
    Heiko::execute_with(|| {
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE.into()),
            KSM,
            ksm(1f64),
            Box::new(xcm::VersionedMultiLocation::V3(MultiLocation::new(
                1,
                X1(Junction::AccountId32 {
                    id: BOB,
                    network: None
                })
            ))),
            WeightLimit::Limited(4_000_000_000.into())
        ));
    });

    KusamaNet::execute_with(|| {
        let para_acc: AccountId = ParaId::from(2085).into_account_truncating();
        println!("heiko para account in relaychain:{:?}", para_acc);
        assert_eq!(
            kusama_runtime::Balances::free_balance(&AccountId::from(BOB)),
            999909712564
        );
    });
}
