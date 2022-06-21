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

use frame_support::traits::GenesisBuild;

use frame_support::traits::Currency;
use pallet_loans::{InterestRateModel, JumpModel, Market, MarketState};
use pallet_traits::{
    ump::{XcmCall, XcmWeightFeeMisc},
    xcm::AssetType,
};
use polkadot_parachain::primitives::Sibling;
use primitives::{tokens::*, AccountId, Balance, CurrencyId, Rate, Ratio};
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

pub const RMRK_ASSET_ID: u32 = 8;
pub const USDT_ASSET_ID: u32 = 1984;
pub const RMRK: CurrencyId = 126;

pub const RMRK_WEIGHT_PER_SEC: u128 = 20_000_000_000;
pub const USDT_WEIGHT_PER_SEC: u128 = 30_000_000;

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
            constants::paras::statemine::{
                ID as StatemineChainId, PALLET_INSTANCE as StatemineAssetInstance,
            },
            AssetRegistry, Assets, LiquidStaking, Loans, Origin, Runtime, System, XcmHelper,
        };
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

        <pallet_liquid_staking::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
            &pallet_liquid_staking::GenesisConfig {
                exchange_rate: Rate::one(),
                reserve_factor: Ratio::from_perthousand(5),
            },
            &mut t,
        )
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| {
            System::set_block_number(1);
            Assets::force_create(
                Origin::root(),
                KSM,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                Origin::root(),
                KSM,
                b"Kusama".to_vec(),
                b"KSM".to_vec(),
                12,
                false,
            )
            .unwrap();
            Assets::mint(
                Origin::signed(AccountId::from(ALICE)),
                KSM,
                MultiAddress::Id(AccountId::from(ALICE)),
                ksm(100f64),
            )
            .unwrap();

            //initialize for statemine rmrk
            Assets::force_create(
                Origin::root(),
                RMRK,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                Origin::root(),
                RMRK,
                b"RMRK".to_vec(),
                b"RMRK".to_vec(),
                RMRK_DECIMAL,
                false,
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
            AssetRegistry::register_asset(Origin::root(), RMRK, statemine_rmrk_asset_type.clone())
                .unwrap();
            AssetRegistry::update_asset_units_per_second(
                Origin::root(),
                statemine_rmrk_asset_type,
                RMRK_WEIGHT_PER_SEC,
            )
            .unwrap();

            XcmHelper::update_xcm_weight_fee(
                Origin::root(),
                XcmCall::TransferToSiblingchain(Box::new((1, Parachain(1000)).into())),
                XcmWeightFeeMisc {
                    weight: WEIGHT_IN_STATEMINE,
                    fee: FEE_IN_STATEMINE,
                },
            )
            .unwrap();

            //initialize for statemine usdt
            Assets::force_create(
                Origin::root(),
                USDT,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                Origin::root(),
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
            AssetRegistry::register_asset(Origin::root(), USDT, statemine_usdt_asset_type.clone())
                .unwrap();
            AssetRegistry::update_asset_units_per_second(
                Origin::root(),
                statemine_usdt_asset_type,
                USDT_WEIGHT_PER_SEC,
            )
            .unwrap();

            //initialize for liquidate staking
            Assets::force_create(
                Origin::root(),
                SKSM,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                Origin::root(),
                SKSM,
                b"Parallel Kusama".to_vec(),
                b"sKSM".to_vec(),
                12,
                false,
            )
            .unwrap();

            Assets::mint(
                Origin::signed(AccountId::from(ALICE)),
                SKSM,
                MultiAddress::Id(AccountId::from(ALICE)),
                ksm(100f64),
            )
            .unwrap();
            LiquidStaking::update_staking_ledger_cap(Origin::root(), ksm(10000f64)).unwrap();

            Assets::mint(
                Origin::signed(AccountId::from(ALICE)),
                KSM,
                MultiAddress::Id(XcmHelper::account_id()),
                ksm(100f64),
            )
            .unwrap();

            Loans::add_market(Origin::root(), KSM, market_mock(PKSM)).unwrap();
            Loans::activate_market(Origin::root(), KSM).unwrap();
        });
        ext
    }

    pub fn parallel_build(self) -> sp_io::TestExternalities {
        use parallel_runtime::{Assets, Origin, Runtime, System};
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
            Assets::force_create(
                Origin::root(),
                DOT,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                Origin::root(),
                DOT,
                b"Polkadot".to_vec(),
                b"DOT".to_vec(),
                12,
                false,
            )
            .unwrap();
            Assets::mint(
                Origin::signed(AccountId::from(ALICE)),
                DOT,
                MultiAddress::Id(AccountId::from(ALICE)),
                dot(100f64),
            )
            .unwrap();
        });
        ext
    }

    pub fn statemine_build(self) -> sp_io::TestExternalities {
        use statemine_runtime::{Assets, Balances, Origin, Runtime, System};

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
                Origin::root(),
                RMRK_ASSET_ID,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                Origin::root(),
                RMRK_ASSET_ID.into(),
                b"RMRK".to_vec(),
                b"RMRK".to_vec(),
                RMRK_DECIMAL,
                false,
            )
            .unwrap();
            Assets::mint(
                Origin::signed(AccountId::from(ALICE)),
                RMRK_ASSET_ID,
                MultiAddress::Id(AccountId::from(ALICE)),
                rmrk(10),
            )
            .unwrap();

            // prepare for usdt
            Assets::force_create(
                Origin::root(),
                USDT_ASSET_ID,
                MultiAddress::Id(AccountId::from(ALICE)),
                true,
                1,
            )
            .unwrap();
            Assets::force_set_metadata(
                Origin::root(),
                USDT_ASSET_ID,
                b"USDT".to_vec(),
                b"USDT".to_vec(),
                USDT_DECIMAL,
                false,
            )
            .unwrap();
            Assets::mint(
                Origin::signed(AccountId::from(ALICE)),
                USDT_ASSET_ID,
                MultiAddress::Id(AccountId::from(ALICE)),
                usdt(10),
            )
            .unwrap();
        });
        ext
    }
}
