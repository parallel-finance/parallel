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

use crate::{kusama_test_net::*, setup::*};

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{assert_ok, traits::ConstU32, WeakBoundedVec};
use primitives::{AccountId, BlockNumber, KAR};
use scale_info::TypeInfo;
use sp_core::{hexdisplay::HexDisplay, RuntimeDebug};
use xcm::latest::prelude::*;
use xcm_emulator::TestExt;

pub type Lease = BlockNumber;
#[derive(
    Encode,
    Decode,
    Eq,
    PartialEq,
    Copy,
    Clone,
    RuntimeDebug,
    PartialOrd,
    Ord,
    TypeInfo,
    MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum CurrencyId {
    Token(),
    DexShare(),
    Erc20(),
    StableAssetPoolToken(),
    LiquidCrowdloan(Lease),
    ForeignAsset(),
}

#[test]
fn transfer_sibling_chain_asset() {
    TestNet::reset();

    //since not easy to introduce runtime from other chain,just use heiko's
    use heiko_runtime::{Assets, Balances, PolkadotXcm, RuntimeOrigin, XTokens};

    MockSibling::execute_with(|| {
        let mut general_key =
            WeakBoundedVec::<u8, ConstU32<32>>::force_from([4, 13].to_vec(), None).to_vec();
        assert_eq!(general_key, vec![4, 13]);
        let mut general_key_hex = format!("0x{:?}", HexDisplay::from(&general_key));
        assert_eq!(general_key_hex, "0x040d");

        general_key = WeakBoundedVec::<u8, ConstU32<32>>::force_from(
            CurrencyId::LiquidCrowdloan(13).encode(),
            None,
        )
        .to_vec();
        assert_eq!(general_key, vec![4, 13, 0, 0, 0]);
        general_key_hex = format!("0x{:?}", HexDisplay::from(&general_key));
        assert_eq!(general_key_hex, "0x040d000000");

        assert_ok!(PolkadotXcm::reserve_transfer_assets(
            RuntimeOrigin::signed(ALICE.into()).clone(),
            Box::new(MultiLocation::new(1, X1(Parachain(2085))).into()),
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
                        b"KAR".to_vec(),
                        None
                    ))),
                    heiko(10)
                )
                    .into()
            ),
            0
        ));
    });

    // Rerun execute to actually send the egress message via XCM
    MockSibling::execute_with(|| {});

    Heiko::execute_with(|| {
        assert_eq!(Assets::balance(KAR, &AccountId::from(BOB)), 9999982000000);
    });

    Heiko::execute_with(|| {
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE.into()),
            0,
            10_000_000_000_000,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(2000), //Sibling chain
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
