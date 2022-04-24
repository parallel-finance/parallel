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

use primitives::{tokens::*, AccountId, Balance};
use sp_runtime::MultiAddress;

pub const ALICE: [u8; 32] = [0u8; 32];
pub const BOB: [u8; 32] = [1u8; 32];
pub const KSM_DECIMAL: u32 = 12;
pub const DOT_DECIMAL: u32 = 10;

pub fn ksm(n: f64) -> Balance {
    (n as u128) * 10u128.pow(KSM_DECIMAL)
}

pub fn dot(n: f64) -> Balance {
    (n as u128) * 10u128.pow(DOT_DECIMAL)
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

    pub fn kusama_build(self) -> sp_io::TestExternalities {
        use vanilla_runtime::{Assets, Origin, Runtime, System};
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
        });
        ext
    }

    pub fn polkadot_build(self) -> sp_io::TestExternalities {
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
}
