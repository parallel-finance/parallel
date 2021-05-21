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
//! This pallets provides offchain worker to call the liquidate_borrow operation in loans pallet.
//! The collator may opt-in with a pre-funded account. The liquidate strategy is:
//! - find the unhealthy account which has excessed loans
//! - liquidate the currency with higher loans
//! - liquidator gets anyone of the affordable collaterals

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
use sp_std::prelude::*;
use frame_support::{
	pallet_prelude::*, log,
};
use frame_system::offchain::{
	AppCrypto, CreateSignedTransaction, SendSignedTransaction,
	Signer,
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
		MultiSignature, MultiSigner,
	};
	app_crypto!(sr25519, KEY_TYPE);

	pub struct AuthId;
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for AuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

/// The miscellaneous information when transforming borrow records.
#[derive(Clone)]
struct BorrowMisc {
	currency: CurrencyId,
	amount: Balance,
	value: Balance,
}

/// The miscellaneous information when transforming collateral records.
#[derive(Clone)]
struct CollateralMisc {
	currency: CurrencyId,
	amount: Balance,
	value: Balance,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config + pallet_loans::Config {
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
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
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000)]
        fn liquidate_borrow(
            origin: OriginFor<T>,
            borrower: T::AccountId,
            liquidate_currency: CurrencyId,
            repay_amount: Balance,
            collateral_currency: CurrencyId,
        ) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
            pallet_loans::Pallet::<T>::liquidate_borrow_internal(
                who,
                borrower,
                liquidate_currency,
                repay_amount,
                collateral_currency,
            )?;

            Ok(().into())
        }
	}
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

		// TODO
		// Only liquidate the currencies the collator allows,
		// also check if the accounts has enough free balances.

		let signer = Signer::<T, T::AuthorityId>::all_accounts();
		if !signer.can_sign() {
			log::error!("liquidate error: no available accounts, consider adding one via `author_insertKey` RPC.");
			return
		}

		let aggregated_account_borrows = pallet_loans::AccountBorrows::<T>::iter()
			.fold(BTreeMap::<T::AccountId, (Balance, Vec<BorrowMisc>)>::new(), |mut acc, (k1, k2, snapshot)| {
				let loans_value = T::PriceFeeder::get_price(&k1).unwrap().0.checked_mul_int(snapshot.principal).unwrap();
				let existing = acc.get(&k2).unwrap();
				let total_loans_value = existing.0 + loans_value;
				let mut loans_detail = existing.1.clone();
				loans_detail.push(BorrowMisc {
					currency: k1,
					amount: snapshot.principal,
					value: loans_value,
				});
				acc.insert(k2, (total_loans_value, loans_detail));
				acc
			});

		let aggregated_account_collatoral = pallet_loans::AccountCollateral::<T>::iter()
			.fold(BTreeMap::<T::AccountId, (Balance, Vec<CollateralMisc>)>::new(), |mut acc, (k1, k2, balance)| {
				let collateral_value = T::PriceFeeder::get_price(&k1).unwrap().0.checked_mul_int(balance).unwrap();
				let under_collatoral_value = pallet_loans::CollateralFactor::<T>::get(&k1).mul_floor(collateral_value);
				let existing = acc.get(&k2).unwrap();
				let totoal_under_collatoral_value = existing.0 + under_collatoral_value;
				let mut collatoral_detail = existing.1.clone();
				collatoral_detail.push(CollateralMisc {
					currency: k1,
					amount: balance,
					value: collateral_value
				});
				acc.insert(k2, (totoal_under_collatoral_value, collatoral_detail));
				acc
			});

		let _underwater_account_borrows = aggregated_account_borrows.iter()
			.filter(|(account, (total_loans_value, _))| {
				total_loans_value > &aggregated_account_collatoral.get(account).unwrap().0
			})
			.map(|(account, (_total_loans_value, loans_detail))| {
				// TODO change to 0.5, configurable by runners
				// TODO shortfall compare with 0.5 * max
				// let liquidation_value = LIQUIDATE_FACTOR.mul_floor(*total_loans_value);
				let mut new_loans_detail = loans_detail.clone();
				new_loans_detail.sort_by(|a, b| a.value.cmp(&b.value));
				let liquidate_loans = &new_loans_detail[0];
				let liquidation_collatoral = 
					aggregated_account_collatoral
					.get(account)
					.unwrap()
					.1
					.iter().find(|collateral_item| 
						collateral_item.value >= LIQUIDATE_FACTOR.mul_floor(liquidate_loans.value)
					).unwrap();
				(
					account,
					liquidate_loans.currency,
					LIQUIDATE_FACTOR.mul_floor(liquidate_loans.amount),
					liquidation_collatoral.currency, 
				)
			})
			.for_each(|(borrower, loan_currency, repay_amount, collateral_currency)| {
				// submit_liquidation(llc, llb, lcc, lcb);
				// pallet_loans::liquidate_borrow(loan_currency, loan_amount, collatoral_currency, collatoral_amount)
				let results = signer.send_signed_transaction(
					|_account| {
						Call::liquidate_borrow(
							borrower.clone(),
							loan_currency.clone(),
							repay_amount.clone(),
							collateral_currency.clone(),
						)
					}
				);
				for (acc, res) in &results {
					match res {
						Ok(()) => log::info!("[{:?}] Submitted liquidate borrow, borrower: {:?}", acc.id, borrower),
						Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
					}
				}
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
