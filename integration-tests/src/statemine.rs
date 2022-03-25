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
use frame_support::assert_ok;
use frame_support::traits::Currency;
use polkadot_parachain::primitives::Sibling;
use primitives::{AccountId, Balance, CurrencyId};
use sp_runtime::traits::AccountIdConversion;
use xcm::latest::prelude::*;
use xcm_emulator::TestExt;

pub const RMRK_ASSET_ID: u32 = 8;
pub const RMRK_DECIMAL: u8 = 10;
pub const RMRK_MINIMAL_BALANCE: Balance = 10;
pub const RMRK_WEIGHT_PER_SEC: u128 = 100000000000;

pub fn rmrk(n: f64) -> Balance {
	(n as u128) * 10u128.pow(RMRK_DECIMAL.into())
}

#[test]
fn statemine() {
	use heiko_runtime::{AssetRegistrarMetadata, AssetType};
	let statemine_rmrk_asset_location =
		MultiLocation::new(1, X3(Parachain(1000), PalletInstance(50), GeneralIndex(8)));
	let statemine_rmrk_asset_type = AssetType::Xcm(statemine_rmrk_asset_location);
	let statemine_rmrk_asset_id: CurrencyId = statemine_rmrk_asset_type.clone().into();
	let statemine_rmrk_asset_meta = AssetRegistrarMetadata {
		name: b"RMRK".to_vec(),
		symbol: b"RMRK".to_vec(),
		decimals: RMRK_DECIMAL,
		is_frozen: false,
	};
	Heiko::execute_with(|| {
		use heiko_runtime::{AssetManager, Origin};
		assert_eq!(statemine_rmrk_asset_id, 4187061565);
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			statemine_rmrk_asset_type.clone(),
			statemine_rmrk_asset_meta.clone(),
			RMRK_MINIMAL_BALANCE,
			true
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			Origin::root(),
			statemine_rmrk_asset_type,
			RMRK_WEIGHT_PER_SEC,
			0
		));
	});
	Statemine::execute_with(|| {
		use statemine_runtime::{Assets, Balances, Origin, PolkadotXcm, System};

		let origin = Origin::signed(ALICE.into());

		Balances::make_free_balance_be(&ALICE.into(), ksm(10f64));

		// need to have some KSM to be able to receive user assets
		Balances::make_free_balance_be(&Sibling::from(2085).into_account(), ksm(10f64));

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
		Assets::mint(
			Origin::signed(AccountId::from(ALICE)),
			RMRK_ASSET_ID,
			MultiAddress::Id(AccountId::from(ALICE)),
			rmrk(10f64),
		)
		.unwrap();

		System::reset_events();

		let para_acc: AccountId = Sibling::from(2085).into_account();
		println!("{:?}", para_acc);

		assert_ok!(PolkadotXcm::reserve_transfer_assets(
			origin.clone(),
			Box::new(MultiLocation::new(1, X1(Parachain(2085))).into()),
			Box::new(
				Junction::AccountId32 {
					id: BOB,
					network: NetworkId::Any
				}
				.into()
				.into()
			),
			Box::new((X2(PalletInstance(50), GeneralIndex(8)), rmrk(1f64)).into()),
			0
		));
		println!("{:?}", System::events());
	});
	// Rerun the Statemine::execute to actually send the egress message via XCM
	Statemine::execute_with(|| {});
	Heiko::execute_with(|| {
		use heiko_runtime::Assets;
		assert_eq!(
			Assets::balance(statemine_rmrk_asset_id, &AccountId::from(BOB)),
			9940000000
		); //rmrk fee in heiko is 60_000_000 which is 0.006rmrk~=0.08$
	})
}
