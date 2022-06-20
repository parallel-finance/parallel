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

//! Cross-chain transfer tests within Polkadot network.

use cumulus_primitives_core::ParaId;
use frame_support::assert_ok;
use parallel_runtime::Assets;
use primitives::{tokens::*, AccountId};
use sp_runtime::traits::AccountIdConversion;
use xcm::{latest::prelude::*, VersionedMultiAssets, VersionedMultiLocation};
use xcm_emulator::TestExt;

use crate::{polkadot_test_net::*, setup::*};

#[test]
fn transfer_from_relay_chain() {
    PolkadotNet::execute_with(|| {
        assert_ok!(polkadot_runtime::XcmPallet::reserve_transfer_assets(
            polkadot_runtime::Origin::signed(ALICE.into()),
            Box::new(VersionedMultiLocation::V1(X1(Parachain(2012)).into())),
            Box::new(VersionedMultiLocation::V1(
                X1(Junction::AccountId32 {
                    id: BOB,
                    network: NetworkId::Any
                })
                .into()
            )),
            Box::new(VersionedMultiAssets::V1((Here, dot(1f64)).into())),
            0,
        ));
    });

    Parallel::execute_with(|| {
        assert_eq!(Assets::balance(DOT, &AccountId::from(BOB)), 9_860_140_000);
        //dot fee in parallel is 139_860_000
    });
}

#[test]
fn transfer_to_relay_chain() {
    use parallel_runtime::{Origin, XTokens};
    Parallel::execute_with(|| {
        assert_ok!(XTokens::transfer(
            Origin::signed(ALICE.into()),
            DOT,
            dot(10f64),
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

    PolkadotNet::execute_with(|| {
        let para_acc: AccountId = ParaId::from(2012).into_account_truncating();
        println!("parallel para account in relaychain:{:?}", para_acc);
        assert_eq!(
            polkadot_runtime::Balances::free_balance(&AccountId::from(BOB)),
            995_305_825_48
        );
    });
}
