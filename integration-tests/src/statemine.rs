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
use frame_support::assert_ok;
use primitives::{tokens::*, AccountId};
use sp_runtime::{traits::AccountIdConversion, MultiAddress};
use xcm::latest::prelude::*;
use xcm_emulator::TestExt;

// pub const FEE_IN_KUSAMA: u128 = 29_642_910;
pub const STATEMINE_TOTAL_FEE_AMOUNT: u128 = 1_000_000_000; //still can be decreased further but we add some margin here

#[test]
fn transfer_statemine_rmrk() {
    //reserve transfer rmrk from statemine to heiko
    Statemine::execute_with(|| {
        use statemine_runtime::{Origin, PolkadotXcm};

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
            Box::new((X2(PalletInstance(50), GeneralIndex(8)), rmrk(2)).into()),
            0
        ));
    });

    // Rerun the Statemine::execute to actually send the egress message via XCM
    Statemine::execute_with(|| {});

    //check rmrk transferred and then transfer it back to statemine with ksm as fee
    Heiko::execute_with(|| {
        use heiko_runtime::{Assets, Origin, XTokens};
        //with RMRK_WEIGHT_PER_SEC set in heiko rmrk fee is 12_000_000 which is 0.0012rmrk~=0.004$
        assert_eq!(Assets::balance(RMRK, &AccountId::from(BOB)), 19988000000);
        assert_ok!(Assets::mint(
            RuntimeOrigin::signed(AccountId::from(ALICE)),
            KSM,
            MultiAddress::Id(AccountId::from(BOB)),
            ksm(1f64),
        )); //mint some ksm to BOB to pay for the xcm fee
        assert_ok!(XTokens::transfer_multicurrencies(
            RuntimeOrigin::signed(BOB.into()),
            vec![(KSM, STATEMINE_TOTAL_FEE_AMOUNT), (RMRK, rmrk(1)),],
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
        ));
    });
    // check reserved ksm move from heiko sovereign to statemine sovereign
    KusamaNet::execute_with(|| {
        let heiko_sovereign: AccountId = ParaId::from(2085u32).into_account_truncating();
        let statemine_sovereign: AccountId = ParaId::from(1000u32).into_account_truncating();
        assert_eq!(
            ksm(100f64) - (STATEMINE_TOTAL_FEE_AMOUNT - FEE_IN_STATEMINE),
            kusama_runtime::Balances::free_balance(&heiko_sovereign)
        ); //fee deducted from heiko_sovereign
           // https://github.com/open-web3-stack/open-runtime-module-library/pull/786/files
           // teleport will bypass the statemine_sovereign so it'll always be zero
        assert_eq!(
            0,
            // STATEMINE_TOTAL_FEE_AMOUNT - FEE_IN_STATEMINE - FEE_IN_KUSAMA,
            kusama_runtime::Balances::free_balance(&statemine_sovereign)
        ); // fee reserved into statemine_sovereign
    });
    // recipient receive rmrk in statemine
    Statemine::execute_with(|| {
        use statemine_runtime::Assets;
        assert_eq!(
            rmrk(1),
            Assets::balance(RMRK_ASSET_ID, &AccountId::from(BOB))
        );
    });
}

#[test]
fn transfer_statemine_usdt() {
    //reserve transfer usdt from statemine to heiko
    Statemine::execute_with(|| {
        use statemine_runtime::{Origin, PolkadotXcm};

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
                    X2(PalletInstance(50), GeneralIndex(USDT_ASSET_ID as u128)),
                    usdt(2),
                )
                    .into()
            ),
            0
        ));
    });

    Statemine::execute_with(|| {});

    //check usdt transferred and then transfer it back to statemine with ksm as fee
    Heiko::execute_with(|| {
        use heiko_runtime::{Assets, Origin, XTokens};
        //with USDT_WEIGHT_PER_SEC set in heiko usdt fee is 0.018$
        assert_eq!(Assets::balance(USDT, &AccountId::from(BOB)), 1982000);
        assert_ok!(Assets::mint(
            RuntimeOrigin::signed(AccountId::from(ALICE)),
            KSM,
            MultiAddress::Id(AccountId::from(BOB)),
            ksm(1f64),
        )); //mint some ksm to BOB to pay for the xcm fee
        assert_ok!(XTokens::transfer_multicurrencies(
            RuntimeOrigin::signed(BOB.into()),
            vec![(KSM, STATEMINE_TOTAL_FEE_AMOUNT), (USDT, usdt(1)),],
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
        ));
    });

    // recipient receive 1 usdt in statemine
    Statemine::execute_with(|| {
        use statemine_runtime::Assets;
        assert_eq!(
            usdt(1),
            Assets::balance(USDT_ASSET_ID, &AccountId::from(BOB))
        );
    });
}
