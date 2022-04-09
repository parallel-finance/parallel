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
use primitives::{tokens::*, AccountId};
use vanilla_runtime::Assets;
use xcm::{latest::prelude::*, VersionedMultiAssets, VersionedMultiLocation};
use xcm_emulator::TestExt;

use crate::{kusama_test_net::*, setup::*};

#[test]
fn transfer_from_relay_chain() {
    KusamaNet::execute_with(|| {
        assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
            kusama_runtime::Origin::signed(ALICE.into()),
            Box::new(VersionedMultiLocation::V1(X1(Parachain(2085)).into())),
            Box::new(VersionedMultiLocation::V1(
                X1(Junction::AccountId32 {
                    id: BOB,
                    network: NetworkId::Any
                })
                .into()
            )),
            Box::new(VersionedMultiAssets::V1((Here, ksm(1f64)).into())),
            0,
        ));
    });

    Vanilla::execute_with(|| {
        assert_eq!(Assets::balance(KSM, &AccountId::from(BOB)), 999_904_000_000);
        //ksm fee in heiko is 96_000_000
    });
}

#[test]
fn transfer_to_relay_chain() {
    use vanilla_runtime::{Origin, XTokens};
    Vanilla::execute_with(|| {
        assert_ok!(XTokens::transfer(
            Origin::signed(ALICE.into()),
            KSM,
            ksm(1f64),
            Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::new(
                1,
                X1(Junction::AccountId32 {
                    id: BOB,
                    network: NetworkId::Any
                })
            ))),
            4_000_000_000
        ));
    });

    KusamaNet::execute_with(|| {
        let para_acc: AccountId = ParaId::from(2085).into_account();
        println!("heiko para account in relaychain:{:?}", para_acc);
        assert_eq!(
            kusama_runtime::Balances::free_balance(&AccountId::from(BOB)),
            999_893_333_340 //xcm fee in kusama is 106_666_660~=0.015$
        );
    });
}
