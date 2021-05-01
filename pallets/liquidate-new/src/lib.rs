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

#![cfg_attr(not(feature = "std"), no_std)]

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
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


}

impl<T: Config> Pallet<T> {
	fn liquidate(_block_number: T::BlockNumber) {
		// TODO add lock

		let prices = Prices::get();
		let currencies = pallet_loans::Currencies::<T>::get();
		let account_borrows = pallet_loans::AccountBorrows::<T>::get();

		let aggregated_account_borrows = pallet_loans::AccountBorrows::<T>::iter()
			.fold(BTreeMap::new(), |acc, (k1, k2, balance)| {
				let loans_value = balance * prices.get(k1);
				let previous_value = acc.get(k2).unwrap();
				let total_loans = previous_value._1 + loans_value;
				let loans_detail = previous_value._2.push((k1, balance));
				acc.insert(k2, (total_loans, loans_detail);
				acc
			});

		let aggregated_account_collatoral = pallet_loans::AccountCollateral::<T>::iter()
			.fold(BTreeMap::new(), |acc, (k1, k2, balance)| {
				let collatoral_value = balance * prices.get(k1);
				let previous_value = acc.get(k2).unwrap();
				let totoal_collatoral_value = previous_value._1 + collatoral_value;
				let collatoral_detail = previous_value._2.push((k1, balance));
				acc.insert(k2, (totoal_collatoral_value, collatoral_detail);
				acc
			});

		let underwater_account_borrows = aggregated_account_borrows.iter()
			.filter(|(account, (loans, _))| {
				loans > aggregated_account_collatoral.get(account).unwrap()._1
			})
			.map(|account, (total_loans_value, loans_detail)| {
				// TODO change to 0.5, configurable by runners
				let liquidation_value = total_loans_value * 0.2;
				let liquidation_loan = loans_detail.find(|(currency, balance)| balance * prices.get(currency) > liquidation_value);
				let liquidation_collatoral = 
					aggregated_account_collatoral
					.get(account)
					.unwarp()
					._2
					.find(|(currency, balance)| balance * prices.get(currency))
				(
					liquidation_loan_currency, 
					liquidation_loan_balance, 
					liquidation_collatoral_currency, 
					liquidation_collatoral_balance
				)
			})
			.for_each(|llc, llb, lcc, lcb| {
				submit_transaction(llc, llb, lcc, lcb);
			})

	

	}
}