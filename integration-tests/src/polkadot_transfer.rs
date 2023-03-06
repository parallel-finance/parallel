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
use frame_support::{assert_ok, traits::ConstU32, WeakBoundedVec};
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
            polkadot_runtime::RuntimeOrigin::signed(ALICE.into()),
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
        assert_eq!(Assets::balance(DOT, &AccountId::from(BOB)), 9_860_956_000);
        //dot fee in parallel is 139_860_000
    });
}

#[test]
fn transfer_to_relay_chain() {
    use parallel_runtime::{RuntimeOrigin, XTokens};
    Parallel::execute_with(|| {
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE.into()),
            DOT,
            dot(10f64),
            Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::new(
                1,
                X1(Junction::AccountId32 {
                    id: BOB,
                    network: NetworkId::Any
                })
            ))),
            WeightLimit::Limited(4_000_000_000)
        ));
    });

    PolkadotNet::execute_with(|| {
        let para_acc: AccountId = ParaId::from(2012).into_account_truncating();
        println!("parallel para account in relaychain:{:?}", para_acc);
        assert_eq!(
            polkadot_runtime::Balances::free_balance(&AccountId::from(BOB)),
            99_591_353_032
        );
    });
}

#[test]
fn transfer_sibling_chain_asset() {
    let _ = env_logger::builder().is_test(true).try_init();
    TestNet::reset();

    //since not easy to introduce runtime from other chain,just use heiko's
    use parallel_runtime::{Assets, Balances, PolkadotXcm, RuntimeOrigin, XTokens};

    MockSibling::execute_with(|| {
        assert_ok!(PolkadotXcm::reserve_transfer_assets(
            RuntimeOrigin::signed(ALICE.into()).clone(),
            Box::new(MultiLocation::new(1, X1(Parachain(2012))).into()),
            Box::new(
                Junction::AccountId32 {
                    id: BOB,
                    network: NetworkId::Any
                }
                .into()
                .into()
            ),
            Box::new(
                (
                    X1(GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
                        b"CLV".to_vec(),
                        None
                    ))),
                    para(10)
                )
                    .into()
            ),
            0
        ));
    });

    // Rerun execute to actually send the egress message via XCM
    MockSibling::execute_with(|| {});

    Parallel::execute_with(|| {
        assert_eq!(Assets::balance(CLV, &AccountId::from(BOB)), 9400000000000);
    });

    Parallel::execute_with(|| {
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE.into()),
            PARA,
            10_000_000_000_000,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(2002), //Sibling chain
                        Junction::AccountId32 {
                            network: NetworkId::Any,
                            id: BOB.into(),
                        }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000),
        ));

        assert_eq!(
            Balances::free_balance(&AccountId::from(ALICE)),
            90_000_000_000_000
        );
    });
}
