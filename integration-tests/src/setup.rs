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

use frame_support::traits::ConstU32;
use frame_support::traits::GenesisBuild;
use frame_support::WeakBoundedVec;

pub use frame_support::pallet_prelude::Weight;
use frame_support::traits::Currency;
use pallet_loans::{InterestRateModel, JumpModel, Market, MarketState};
use pallet_traits::{
    ump::{XcmCall, XcmWeightFeeMisc},
    xcm::AssetType,
};
use polkadot_parachain::primitives::Sibling;
use primitives::{paras, tokens::*, AccountId, Balance, CurrencyId, Rate, Ratio};
use sp_runtime::{
    traits::{AccountIdConversion, One},
    FixedPointNumber, MultiAddress,
};
use xcm::latest::prelude::*;

pub const ALICE: [u8; 32] = [0u8; 32];
pub const BOB: [u8; 32] = [1u8; 32];
pub const KSM_DECIMAL: u32 = 12;
pub const DOT_DECIMAL: u32 = 10;
pub const RMRK_DECIMAL: u8 = 10;
pub const USDT_DECIMAL: u8 = 6;
pub const HEIKO_DECIMAL: u8 = 12;
pub const PARA_DECIMAL: u8 = 12;
pub const KAR_DECIMAL: u8 = 12;
pub const CLV_DECIMAL: u8 = 18;

pub const RMRK_ASSET_ID: u32 = 8;
pub const USDT_ASSET_ID: u32 = 1984;
pub const RMRK: CurrencyId = 126;

pub const PARA_WEIGHT_PER_SEC: u128 = 231_740_000_000;
pub const HKO_WEIGHT_PER_SEC: u128 = 231_740_000_000;
pub const DOT_WEIGHT_PER_SEC: u128 = 231_740_000_000;
pub const KSM_WEIGHT_PER_SEC: u128 = 231_740_000_000;
pub const RMRK_WEIGHT_PER_SEC: u128 = 20_000_000_000;
pub const USDT_WEIGHT_PER_SEC: u128 = 30_000_000;
pub const KAR_WEIGHT_PER_SEC: u128 = 30_000_000_000;
pub const CLV_WEIGHT_PER_SEC: u128 = 1_000_000_000_000_000;

pub const FEE_IN_STATEMINE: u128 = 15_540_916;
pub const WEIGHT_IN_STATEMINE: u64 = 4_000_000_000;

pub fn ksm(n: f64) -> Balance {
    (n as u128) * 10u128.pow(KSM_DECIMAL)
}

pub fn dot(n: f64) -> Balance {
    (n as u128) * 10u128.pow(DOT_DECIMAL)
}

pub fn rmrk(n: u128) -> Balance {
    n * 10u128.pow(RMRK_DECIMAL.into())
}

pub fn usdt(n: u128) -> Balance {
    n * 10u128.pow(USDT_DECIMAL.into())
}

pub fn heiko(n: u128) -> Balance {
    n * 10u128.pow(HEIKO_DECIMAL.into())
}

pub fn para(n: u128) -> Balance {
    n * 10u128.pow(PARA_DECIMAL.into())
}

pub const fn market_mock(ptoken_id: u32) -> Market<Balance> {
    Market {
        close_factor: Ratio::from_percent(50),
        collateral_factor: Ratio::from_percent(50),
        liquidation_threshold: Ratio::from_percent(55),
        liquidate_incentive: Rate::from_inner(Rate::DIV / 100 * 110),
        liquidate_incentive_reserved_factor: Ratio::from_percent(3),
        state: MarketState::Pending,
        rate_model: InterestRateModel::Jump(JumpModel {
            base_rate: Rate::from_inner(Rate::DIV / 100 * 2),
            jump_rate: Rate::from_inner(Rate::DIV / 100 * 10),
            full_rate: Rate::from_inner(Rate::DIV / 100 * 32),
            jump_utilization: Ratio::from_percent(80),
        }),
        reserve_factor: Ratio::from_percent(15),
        supply_cap: 1_000_000_000_000_000_000_000u128, // set to 1B
        borrow_cap: 1_000_000_000_000_000_000_000u128, // set to 1B
        ptoken_id,
    }
}

pub struct ExtBuilder {
    pub parachain_id: u32,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self { parachain_id: 2085 }
    }
}

impl ExtBuilder {
    pub fn parachain_id(mut self, parachain_id: u32) -> Self {
        self.parachain_id = parachain_id;
        self
    }

    pub fn heiko_build(self) -> sp_io::TestExternalities {
        use heiko_runtime::{
            AssetRegistry, Assets, LiquidStaking, Loans, RuntimeOrigin, System, XcmHelper,
        };
        use paras::statemine::{ID as StatemineChainId, PALLET_INSTANCE as StatemineAssetInstance};
        let mut ext = sp_io::TestExternalities::new(self.init());
        ext.execute_with(|| {
            System::set_block_number(1);
            Assets::force_create(
                RuntimeOrigin::root(),
                KSM,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                RuntimeOrigin::root(),
                KSM,
                b"Kusama".to_vec(),
                b"KSM".to_vec(),
                12,
                false,
            )
            .unwrap();
            Assets::mint(
                RuntimeOrigin::signed(AccountId::from(ALICE)),
                KSM,
                MultiAddress::Id(AccountId::from(ALICE)),
                ksm(100f64),
            )
            .unwrap();

            //initialize for statemine rmrk
            Assets::force_create(
                RuntimeOrigin::root(),
                RMRK,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                RuntimeOrigin::root(),
                RMRK,
                b"RMRK".to_vec(),
                b"RMRK".to_vec(),
                RMRK_DECIMAL,
                false,
            )
            .unwrap();

            let hko_asset_location = MultiLocation::new(
                1,
                X2(
                    Parachain(2085),
                    GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
                        b"HKO".to_vec(),
                        None,
                    )),
                ),
            );
            let hko_asset_type = AssetType::Xcm(hko_asset_location);
            AssetRegistry::register_asset(RuntimeOrigin::root(), HKO, hko_asset_type.clone())
                .unwrap();
            AssetRegistry::update_asset_units_per_second(
                RuntimeOrigin::root(),
                hko_asset_type,
                HKO_WEIGHT_PER_SEC,
            )
            .unwrap();

            let ksm_asset_location = MultiLocation::parent();
            let ksm_asset_type = AssetType::Xcm(ksm_asset_location);
            AssetRegistry::register_asset(RuntimeOrigin::root(), KSM, ksm_asset_type.clone())
                .unwrap();
            AssetRegistry::update_asset_units_per_second(
                RuntimeOrigin::root(),
                ksm_asset_type,
                KSM_WEIGHT_PER_SEC,
            )
            .unwrap();

            let statemine_rmrk_asset_location = MultiLocation::new(
                1,
                X3(
                    Parachain(StatemineChainId),
                    PalletInstance(StatemineAssetInstance),
                    GeneralIndex(RMRK_ASSET_ID as u128),
                ),
            );
            let statemine_rmrk_asset_type = AssetType::Xcm(statemine_rmrk_asset_location);
            AssetRegistry::register_asset(
                RuntimeOrigin::root(),
                RMRK,
                statemine_rmrk_asset_type.clone(),
            )
            .unwrap();
            AssetRegistry::update_asset_units_per_second(
                RuntimeOrigin::root(),
                statemine_rmrk_asset_type,
                RMRK_WEIGHT_PER_SEC,
            )
            .unwrap();

            XcmHelper::update_xcm_weight_fee(
                RuntimeOrigin::root(),
                XcmCall::TransferToSiblingchain(Box::new((1, Parachain(1000)).into())),
                XcmWeightFeeMisc {
                    weight: Weight::from_ref_time(WEIGHT_IN_STATEMINE),
                    fee: FEE_IN_STATEMINE,
                },
            )
            .unwrap();

            //initialize for statemine usdt
            Assets::force_create(
                RuntimeOrigin::root(),
                USDT,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                RuntimeOrigin::root(),
                USDT,
                b"USDT".to_vec(),
                b"USDT".to_vec(),
                USDT_DECIMAL,
                false,
            )
            .unwrap();
            let statemine_usdt_asset_location = MultiLocation::new(
                1,
                X3(
                    Parachain(StatemineChainId),
                    PalletInstance(StatemineAssetInstance),
                    GeneralIndex(USDT_ASSET_ID as u128),
                ),
            );
            let statemine_usdt_asset_type = AssetType::Xcm(statemine_usdt_asset_location);
            AssetRegistry::register_asset(
                RuntimeOrigin::root(),
                USDT,
                statemine_usdt_asset_type.clone(),
            )
            .unwrap();
            AssetRegistry::update_asset_units_per_second(
                RuntimeOrigin::root(),
                statemine_usdt_asset_type,
                USDT_WEIGHT_PER_SEC,
            )
            .unwrap();

            //initialize for acala kar as mock sibling
            Assets::force_create(
                RuntimeOrigin::root(),
                KAR,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                RuntimeOrigin::root(),
                KAR,
                b"KAR".to_vec(),
                b"KAR".to_vec(),
                KAR_DECIMAL,
                false,
            )
            .unwrap();
            let kar_asset_location = MultiLocation::new(
                1,
                X2(
                    Parachain(2000),
                    //since we use hko to mock kar,just use hko location here
                    GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
                        b"HKO".to_vec(),
                        None,
                    )),
                ),
            );
            let kar_asset_type = AssetType::Xcm(kar_asset_location);
            AssetRegistry::register_asset(RuntimeOrigin::root(), KAR, kar_asset_type.clone())
                .unwrap();
            AssetRegistry::update_asset_units_per_second(
                RuntimeOrigin::root(),
                kar_asset_type,
                KAR_WEIGHT_PER_SEC,
            )
            .unwrap();

            //initialize for liquidate staking
            Assets::force_create(
                RuntimeOrigin::root(),
                SKSM,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                RuntimeOrigin::root(),
                SKSM,
                b"Parallel Kusama".to_vec(),
                b"sKSM".to_vec(),
                12,
                false,
            )
            .unwrap();

            Assets::mint(
                RuntimeOrigin::signed(AccountId::from(ALICE)),
                SKSM,
                MultiAddress::Id(AccountId::from(ALICE)),
                ksm(100f64),
            )
            .unwrap();
            LiquidStaking::update_staking_ledger_cap(RuntimeOrigin::root(), ksm(10000f64)).unwrap();

            Assets::mint(
                RuntimeOrigin::signed(AccountId::from(ALICE)),
                KSM,
                MultiAddress::Id(XcmHelper::account_id()),
                ksm(100f64),
            )
            .unwrap();

            Loans::add_market(RuntimeOrigin::root(), KSM, market_mock(PKSM)).unwrap();
            Loans::activate_market(RuntimeOrigin::root(), KSM).unwrap();
        });
        ext
    }

    pub fn parallel_build(self) -> sp_io::TestExternalities {
        use parallel_runtime::{AssetRegistry, Assets, Runtime, RuntimeOrigin, System};
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        <parachain_info::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
            &parachain_info::GenesisConfig {
                parachain_id: self.parachain_id.into(),
            },
            &mut t,
        )
        .unwrap();

        <pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
            &pallet_xcm::GenesisConfig {
                safe_xcm_version: Some(2),
            },
            &mut t,
        )
        .unwrap();

        pallet_balances::GenesisConfig::<Runtime> {
            balances: vec![
                (AccountId::from(ALICE), para(100)),
                (AccountId::from(BOB), para(100)),
            ],
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| {
            System::set_block_number(1);
            Assets::force_create(
                RuntimeOrigin::root(),
                DOT,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                RuntimeOrigin::root(),
                DOT,
                b"Polkadot".to_vec(),
                b"DOT".to_vec(),
                12,
                false,
            )
            .unwrap();
            Assets::mint(
                RuntimeOrigin::signed(AccountId::from(ALICE)),
                DOT,
                MultiAddress::Id(AccountId::from(ALICE)),
                dot(100f64),
            )
            .unwrap();

            //initialize for clv as mock sibling
            Assets::force_create(
                RuntimeOrigin::root(),
                CLV,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                RuntimeOrigin::root(),
                CLV,
                b"CLV".to_vec(),
                b"CLV".to_vec(),
                CLV_DECIMAL,
                false,
            )
            .unwrap();
            let para_asset_location = MultiLocation::new(
                1,
                X2(
                    Parachain(2012),
                    GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
                        b"PARA".to_vec(),
                        None,
                    )),
                ),
            );
            let para_asset_type = AssetType::Xcm(para_asset_location);
            AssetRegistry::register_asset(RuntimeOrigin::root(), PARA, para_asset_type.clone())
                .unwrap();
            AssetRegistry::update_asset_units_per_second(
                RuntimeOrigin::root(),
                para_asset_type,
                PARA_WEIGHT_PER_SEC,
            )
            .unwrap();
            let dot_asset_location = MultiLocation::parent();
            let dot_asset_type = AssetType::Xcm(dot_asset_location);
            AssetRegistry::register_asset(RuntimeOrigin::root(), DOT, dot_asset_type.clone())
                .unwrap();
            AssetRegistry::update_asset_units_per_second(
                RuntimeOrigin::root(),
                dot_asset_type,
                DOT_WEIGHT_PER_SEC,
            )
            .unwrap();
            let clv_asset_location = MultiLocation::new(
                1,
                X2(
                    Parachain(2002),
                    //since we use para to mock clv,just use para location here
                    GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
                        b"PARA".to_vec(),
                        None,
                    )),
                ),
            );
            let clv_asset_type = AssetType::Xcm(clv_asset_location);
            AssetRegistry::register_asset(RuntimeOrigin::root(), CLV, clv_asset_type.clone())
                .unwrap();
            AssetRegistry::update_asset_units_per_second(
                RuntimeOrigin::root(),
                clv_asset_type,
                CLV_WEIGHT_PER_SEC,
            )
            .unwrap();
        });
        ext
    }

    pub fn statemine_build(self) -> sp_io::TestExternalities {
        use statemine_runtime::{Assets, Balances, Runtime, RuntimeOrigin, System};

        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        <parachain_info::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
            &parachain_info::GenesisConfig {
                parachain_id: self.parachain_id.into(),
            },
            &mut t,
        )
        .unwrap();

        <pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
            &pallet_xcm::GenesisConfig {
                safe_xcm_version: Some(2),
            },
            &mut t,
        )
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| {
            System::set_block_number(1);

            Balances::make_free_balance_be(&ALICE.into(), ksm(1f64));

            // sibling account need to have some KSM to be able to receive non_sufficient assets
            let para_acc: AccountId = Sibling::from(2085).into_account_truncating();
            println!("heiko para account in sibling chain:{:?}", para_acc);
            Balances::make_free_balance_be(&para_acc, ksm(1f64));

            // prepare for rmrk
            Assets::force_create(
                RuntimeOrigin::root(),
                RMRK_ASSET_ID,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                RuntimeOrigin::root(),
                RMRK_ASSET_ID.into(),
                b"RMRK".to_vec(),
                b"RMRK".to_vec(),
                RMRK_DECIMAL,
                false,
            )
            .unwrap();
            Assets::mint(
                RuntimeOrigin::signed(AccountId::from(ALICE)),
                RMRK_ASSET_ID,
                MultiAddress::Id(AccountId::from(ALICE)),
                rmrk(10),
            )
            .unwrap();

            // prepare for usdt
            Assets::force_create(
                RuntimeOrigin::root(),
                USDT_ASSET_ID,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                RuntimeOrigin::root(),
                USDT_ASSET_ID,
                b"USDT".to_vec(),
                b"USDT".to_vec(),
                USDT_DECIMAL,
                false,
            )
            .unwrap();
            Assets::mint(
                RuntimeOrigin::signed(AccountId::from(ALICE)),
                USDT_ASSET_ID,
                MultiAddress::Id(AccountId::from(ALICE)),
                usdt(10),
            )
            .unwrap();
        });
        ext
    }

    pub fn karura_build(self) -> sp_io::TestExternalities {
        use heiko_runtime::{AssetRegistry, RuntimeOrigin, System};
        let mut ext = sp_io::TestExternalities::new(self.init());
        ext.execute_with(|| {
            System::set_block_number(1);
            let hko_asset_location = MultiLocation::new(
                0,
                X1(GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
                    b"HKO".to_vec(),
                    None,
                ))),
            );
            let hko_asset_type = AssetType::Xcm(hko_asset_location);
            AssetRegistry::register_asset(RuntimeOrigin::root(), HKO, hko_asset_type.clone())
                .unwrap();
            AssetRegistry::update_asset_units_per_second(
                RuntimeOrigin::root(),
                hko_asset_type,
                HKO_WEIGHT_PER_SEC,
            )
            .unwrap();
        });
        ext
    }

    pub fn clv_build(self) -> sp_io::TestExternalities {
        use heiko_runtime::{AssetRegistry, RuntimeOrigin, System};
        let mut ext = sp_io::TestExternalities::new(self.init());
        ext.execute_with(|| {
            System::set_block_number(1);
            let para_asset_location = MultiLocation::new(
                0,
                X1(GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
                    b"PARA".to_vec(),
                    None,
                ))),
            );
            let para_asset_type = AssetType::Xcm(para_asset_location);
            AssetRegistry::register_asset(RuntimeOrigin::root(), PARA, para_asset_type.clone())
                .unwrap();
            AssetRegistry::update_asset_units_per_second(
                RuntimeOrigin::root(),
                para_asset_type,
                PARA_WEIGHT_PER_SEC,
            )
            .unwrap();
        });
        ext
    }

    fn init(self) -> sp_runtime::Storage {
        use heiko_runtime::Runtime;
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        <parachain_info::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
            &parachain_info::GenesisConfig {
                parachain_id: self.parachain_id.into(),
            },
            &mut t,
        )
        .unwrap();

        <pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
            &pallet_xcm::GenesisConfig {
                safe_xcm_version: Some(2),
            },
            &mut t,
        )
        .unwrap();

        pallet_balances::GenesisConfig::<Runtime> {
            balances: vec![
                (AccountId::from(ALICE), heiko(100)),
                (AccountId::from(BOB), heiko(100)),
            ],
        }
        .assimilate_storage(&mut t)
        .unwrap();

        <pallet_liquid_staking::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
            &pallet_liquid_staking::GenesisConfig {
                exchange_rate: Rate::one(),
                reserve_factor: Ratio::from_perthousand(5),
            },
            &mut t,
        )
        .unwrap();
        t
    }
}
