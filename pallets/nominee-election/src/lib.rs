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

//! # Nominee Election pallet
//!
//! ## Overview
//!
//! This pallet stores the offchain elected validators on-chain and maintain
//! a whitelisted validators which can be selected by council.
//!
//! R: Reputation, 0 or 1
//! CR: Commission Rate
//! N: Nomination of one validator
//! EEP: Average Era Points of one validator in the past week.
//! EEPA: Average Era Points of All validators in the past week.
//! SR: Slash Record, default 1, set to 0 if ever slashed in the past month.
//!
//! crf: A constant shows how much influence of the Commission Rate to validator's final score
//!
//! nf: A constant shows how much influence of the Nomination to validator's final score
//!
//! epf: A constant shows how much influence of the Era Points to validator's final score
//!
//! Score: R * (CRF * (1 - CR) + NF * (1 / N) + EPF * (EEP / EEPA)) * SR

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{pallet_prelude::*, traits::SortedMembers, transactional};
use frame_system::pallet_prelude::*;

pub use pallet::*;

use scale_info::TypeInfo;
use sp_std::{convert::TryInto, vec::Vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Info of the validator to be elected
#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, Default, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct ValidatorInfo<AccountId> {
    pub name: Option<Vec<u8>>,
    // Account Id
    pub address: AccountId,
    // Nomination (token amount)
    pub stakes: u128,
    // Score
    pub score: u128,
}

#[frame_support::pallet]
pub mod pallet {

    use super::*;

    type ValidatorSet<T> = BoundedVec<
        ValidatorInfo<<T as frame_system::Config>::AccountId>,
        <T as Config>::MaxValidators,
    >;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The origin which can update staking election coefficients
        type UpdateOrigin: EnsureOrigin<Self::Origin>;

        /// The maximum size of selected validators
        #[pallet::constant]
        type MaxValidators: Get<u32>;

        /// Approved accouts which can set validators
        type Members: SortedMembers<Self::AccountId>;
    }

    /// Validators selected by off-chain client
    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> = StorageValue<_, ValidatorSet<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Validator set updated (old_validators, new_validators)
        ValidorsUpdated(ValidatorSet<T>, ValidatorSet<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The maximum number of validators exceeded
        MaxValidatorsExceeded,
        /// Feeded validators cannot be empty
        NoEmptyValidators,
        /// Invalid validators feeder
        BadValidatorsFeeder,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set selected validators
        ///
        /// If the validators passed are empty, return an error
        #[pallet::weight(1000)]
        #[transactional]
        pub fn set_validators(
            origin: OriginFor<T>,
            validators: Vec<ValidatorInfo<T::AccountId>>,
        ) -> DispatchResult {
            let feeder = ensure_signed(origin)?;
            ensure!(
                T::Members::contains(&feeder),
                Error::<T>::BadValidatorsFeeder
            );
            ensure!(!validators.is_empty(), Error::<T>::NoEmptyValidators);

            let old_validators = Self::validators();
            let new_validators: ValidatorSet<T> = validators
                .try_into()
                .map_err(|_| Error::<T>::MaxValidatorsExceeded)?;

            Validators::<T>::put(new_validators.clone());
            Self::deposit_event(Event::<T>::ValidorsUpdated(old_validators, new_validators));
            Ok(())
        }
    }
}
