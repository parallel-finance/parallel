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

//! Liquidate pallet
//!
//! This pallets provides offchain worker to call the liquidate operation in loans pallet.
//! The collator may opt-in with a pre-funded account.

#![cfg_attr(not(feature = "std"), no_std)]


use sp_core::{
	crypto::KeyTypeId,
};
use sp_runtime::{
	FixedPointNumber, Percent,
	offchain::{
		Duration,
		storage_lock::{StorageLock, Time}
	}
};
use sp_std::collections::btree_map::BTreeMap;
use frame_support::{
	// dispatch::DispatchResultWithPostInfo,
	pallet_prelude::*,
	log,
};
use frame_system::pallet_prelude::*;

use primitives::{Balance, CurrencyId, PriceFeeder};
pub use pallet::*;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"liqu");
pub const LOCK_PERIOD: u64 = 20000; // in milli-seconds
pub const LIQUIDATE_FACTOR: Percent = Percent::from_percent(50); // 0.5

pub enum Error {
	/// There is no pre-configured currencies
	NoCurrencies,
}
pub mod crypto {
	use super::KEY_TYPE;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		// traits::Verify,
	};
	// use sp_core::sr25519::Signature as Sr25519Signature;
	app_crypto!(sr25519, KEY_TYPE);

	// pub sturct TestAuthId;
	// impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature> for TestAuthId {
	// 	type RuntimeAppPublic = Public;
	// 	type GenericSignature = sp_core::sr25519::Signature;
	// 	type GenericPUblic = sp_core::sr25519::Public;
	// }
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_loans::Config {
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			Self::liquidate(block_number);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	fn liquidate(_block_number: T::BlockNumber) {
		let mut lock = StorageLock::<Time>::with_deadline(
			b"liquidate::lock",
			Duration::from_millis(LOCK_PERIOD),
		);

		if let Err(_) = lock.try_lock() {
			log::error!("liquidate error: get lock failed");
			return
		}

		// The currencies collator want to liquidate
		// let currencies: Vec<CurrencyId> = StorageValueRef::persistent(b"liquidate::currencies")
		// 	.get()
		// 	.flattan()
		// 	.ok_or(Error::NoCurrencies);
		

		let aggregated_account_borrows = pallet_loans::AccountBorrows::<T>::iter()
			.fold(BTreeMap::<T::AccountId, (Balance, Vec<(CurrencyId, Balance)>)>::new(), |acc, (k1, k2, snapshot)| {
				let loans_value = T::PriceFeeder::get_price(&k1).unwrap().0.checked_mul_int(snapshot.principal).unwrap();
				let existing = acc.get(&k2).unwrap();
				let total_loans_value = existing.0 + loans_value;
				let mut loans_detail = existing.1;
				loans_detail.push((k1, snapshot.principal));
				acc.insert(k2, (total_loans_value, loans_detail));
				acc
			});

		let aggregated_account_collatoral = pallet_loans::AccountCollateral::<T>::iter()
			.fold(BTreeMap::<T::AccountId, (Balance, Vec<(CurrencyId, Balance)>)>::new(), |acc, (k1, k2, balance)| {
				let collatoral_value = pallet_loans::CollateralFactor::<T>::get(&k1).mul_floor(
					T::PriceFeeder::get_price(&k1).unwrap().0.checked_mul_int(balance).unwrap()
				);
				let existing = acc.get(&k2).unwrap();
				let totoal_collatoral_value = existing.0 + collatoral_value;
				let mut collatoral_detail = existing.1;
				collatoral_detail.push((k1, balance));
				acc.insert(k2, (totoal_collatoral_value, collatoral_detail));
				acc
			});

		let underwater_account_borrows = aggregated_account_borrows.iter()
			.filter(|(account, (total_loans_value, _))| {
				total_loans_value > &aggregated_account_collatoral.get(account).unwrap().0
			})
			.map(|(account, (total_loans_value, loans_detail))| {
				// TODO change to 0.5, configurable by runners
				// TODO shortfall compare with 0.5 * max
				let liquidation_value = LIQUIDATE_FACTOR.mul_floor(*total_loans_value);
				let liquidation_loan = (*loans_detail).find(
					|(currency, balance)| T::PriceFeeder::get_price(currency).unwrap().0.checked_mul_int(balance) >= liquidation_value
				);
				let liquidation_collatoral = 
					aggregated_account_collatoral
					.get(account)
					.unwarp() // TODO several assets to liquidation
					.1
					.find(|(currency, balance)| 
						T::PriceFeeder::get_price(currency).unwrap().0.checked_mul_int(balance) >= liquidation_loan
					);
				(
					account,
					liquidation_loan.unwrap().0,
					liquidation_collatoral.unwrap().0, 
					liquidation_value,
				)
			})
			.for_each(|llc, llb, lcc, lcb| {
				// submit_liquidation(llc, llb, lcc, lcb);
				// pallet_loans::liquidate_borrow(loan_currency, loan_amount, collatoral_currency, collatoral_amount)
				log::info!("new transaction needs to be submitted");
			});

	

	}

	// fn submit_liquidation(
	// 	borrower: T::AccountId,
	// 	loan_currency: CurrencyId,
	// 	collatoral_currency: CurrencyId,
	// 	liquidation_value: Balance
	// ) {
	// 	// 

	// }
}

// impl<T: Config> BlockNumberProvider for Pallet<T> {
// 	type BlockNumber = T::BlockNumber;
// 	fn current_block_number() -> Self::BlockNumber {
// 		<frame_system::Pallet<T>>::block_number()
// 	}
// }