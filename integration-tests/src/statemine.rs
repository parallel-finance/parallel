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

use cumulus_primitives_core::ParaId;
use frame_support::{assert_ok, traits::Currency};
use pallet_traits::ump::{XcmCall, XcmWeightFeeMisc};
use polkadot_parachain::primitives::Sibling;
use primitives::{tokens::*, AccountId, Balance, CurrencyId};
use sp_runtime::{traits::AccountIdConversion, MultiAddress};
use xcm::latest::prelude::*;
use xcm_emulator::TestExt;

pub const RMRK_ASSET_ID: u32 = 8;
pub const RMRK_DECIMAL: u8 = 10;
pub const RMRK_WEIGHT_PER_SEC: u128 = 100000000000;
pub const HEIKO_RMRK_ASSET_ID: u32 = 4187061565;
pub const STATEMINE_TOTAL_FEE_AMOUNT: u128 = 1_000_000_000; //still can be decreased further but we add some margin here
pub const FEE_IN_KUSAMA: u128 = 165_940_672;
pub const FEE_IN_STATEMINE: u128 = 10_666_664;
pub const WEIGHT_IN_STATEMINE: u64 = 4_000_000_000;

pub fn rmrk(n: f64) -> Balance {
    (n as u128) * 10u128.pow(RMRK_DECIMAL.into())
}

#[test]
fn statemine() {
    use pallet_traits::xcm::AssetType;
    let statemine_rmrk_asset_location =
        MultiLocation::new(1, X3(Parachain(1000), PalletInstance(50), GeneralIndex(8)));
    let statemine_rmrk_asset_type = AssetType::Xcm(statemine_rmrk_asset_location);
    let statemine_rmrk_asset_id: CurrencyId = statemine_rmrk_asset_type.clone().into();
    Heiko::execute_with(|| {
        use heiko_runtime::{AssetRegistry, Assets, Origin};
        assert_eq!(statemine_rmrk_asset_id, HEIKO_RMRK_ASSET_ID);
        Assets::force_create(
            Origin::root(),
            HEIKO_RMRK_ASSET_ID,
            MultiAddress::Id(AccountId::from(ALICE)),
            true,
            1,
        )
        .unwrap();
        Assets::force_set_metadata(
            Origin::root(),
            HEIKO_RMRK_ASSET_ID,
            b"RMRK".to_vec(),
            b"RMRK".to_vec(),
            RMRK_DECIMAL,
            false,
        )
        .unwrap();
        assert_ok!(AssetRegistry::register_asset(
            Origin::root(),
            statemine_rmrk_asset_type.clone().into(),
            statemine_rmrk_asset_type.clone(),
        ));
        assert_ok!(AssetRegistry::update_asset_units_per_second(
            Origin::root(),
            statemine_rmrk_asset_type,
            RMRK_WEIGHT_PER_SEC,
        ));
    });
    Statemine::execute_with(|| {
        use statemine_runtime::{Assets, Balances, Origin, PolkadotXcm, System};

        Balances::make_free_balance_be(&ALICE.into(), ksm(1f64));

        // need to have some KSM to be able to receive user assets
        let para_acc: AccountId = Sibling::from(2085).into_account();
        println!("heiko para account in sibling chain:{:?}", para_acc);
        Balances::make_free_balance_be(&para_acc, ksm(1f64));

        // create assets and set metadata
        Assets::force_create(
            Origin::root(),
            RMRK_ASSET_ID,
            MultiAddress::Id(AccountId::from(ALICE)),
            true,
            1,
        )
        .unwrap();
        Assets::force_set_metadata(
            Origin::root(),
            RMRK_ASSET_ID,
            b"RMRK".to_vec(),
            b"RMRK".to_vec(),
            RMRK_DECIMAL,
            false,
        )
        .unwrap();

        //mint 10 rmrk to alice
        Assets::mint(
            Origin::signed(AccountId::from(ALICE)),
            RMRK_ASSET_ID,
            MultiAddress::Id(AccountId::from(ALICE)),
            rmrk(10f64),
        )
        .unwrap();

        System::reset_events();

        //reserve transfer rmrk from statemine to heiko
        assert_ok!(PolkadotXcm::reserve_transfer_assets(
            Origin::signed(ALICE.into()).clone(),
            Box::new(MultiLocation::new(1, X1(Parachain(2085))).into()),
            Box::new(
                Junction::AccountId32 {
                    id: BOB,
                    network: NetworkId::Any
                }
                .into()
                .into()
            ),
            Box::new((X2(PalletInstance(50), GeneralIndex(8)), rmrk(2f64)).into()),
            0
        ));
    });

    // Rerun the Statemine::execute to actually send the egress message via XCM
    Statemine::execute_with(|| {});

    Heiko::execute_with(|| {
        use heiko_runtime::{Assets, Origin, XTokens, XcmHelper};
        assert_eq!(
            Assets::balance(statemine_rmrk_asset_id, &AccountId::from(BOB)),
            19940000000
        ); //with RMRK_WEIGHT_PER_SEC set in heiko fee is 60_000_000 which is 0.006rmrk~=0.09$
        assert_ok!(Assets::mint(
            Origin::signed(AccountId::from(ALICE)),
            KSM,
            MultiAddress::Id(AccountId::from(BOB)),
            ksm(1f64),
        )); //mint some ksm to BOB to pay for the xcm fee
        assert_ok!(XcmHelper::update_xcm_weight_fee(
            Origin::root(),
            XcmCall::TransferToSiblingchain(Box::new((1, Parachain(1000)).into())),
            XcmWeightFeeMisc {
                weight: WEIGHT_IN_STATEMINE,
                fee: FEE_IN_STATEMINE,
            }
        )); // set TransferToSiblingchain storage
        assert_ok!(XTokens::transfer_multicurrencies(
            Origin::signed(BOB.into()),
            vec![
                (KSM, STATEMINE_TOTAL_FEE_AMOUNT),
                (HEIKO_RMRK_ASSET_ID, rmrk(1f64)),
            ],
            0,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(1000),
                        Junction::AccountId32 {
                            network: NetworkId::Any,
                            id: BOB.into(),
                        }
                    )
                )
                .into()
            ),
            WEIGHT_IN_STATEMINE as u64
        )); //transfer rmrk back to statemine with ksm as fee
    });
    KusamaNet::execute_with(|| {
        let heiko_sovereign: AccountId = ParaId::from(2085u32).into_account();
        let statemine_sovereign: AccountId = ParaId::from(1000u32).into_account();
        assert_eq!(
            ksm(100f64) - (STATEMINE_TOTAL_FEE_AMOUNT - FEE_IN_STATEMINE),
            kusama_runtime::Balances::free_balance(&heiko_sovereign)
        ); //fee deducted from heiko_sovereign
        assert_eq!(
            STATEMINE_TOTAL_FEE_AMOUNT - FEE_IN_STATEMINE - FEE_IN_KUSAMA,
            kusama_runtime::Balances::free_balance(&statemine_sovereign)
        ); // fee reserved into statemine_sovereign
    });
    Statemine::execute_with(|| {
        use statemine_runtime::Assets;
        // recipient receive rmrk in statemine
        assert_eq!(
            rmrk(1f64),
            Assets::balance(RMRK_ASSET_ID, &AccountId::from(BOB))
        );
    });
}
