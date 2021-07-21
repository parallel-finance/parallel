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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{log, pallet_prelude::*, traits::SortedMembers, transactional};
use frame_system::pallet_prelude::*;
pub use pallet::*;

use sp_std::{convert::TryInto, vec::Vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Nominee Election Coefficients
/// https://docs.parallel.fi/dev/staking/staking-election
#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct NomineeCoefficients {
    // R: Reputation, 0 or 1
    // CR: Commission Rate
    // N: Nomination of one validator
    // EEP: Average Era Points of one validator in the past week.
    // EEPA: Average Era Points of All validators in the past week.
    // SR: Slash Record, default 1, set to 0 if ever slashed in the past month.

    // Commission rate factor
    // A constant shows how much influence of the Commission Rate to validator's final score
    pub crf: u32,
    // Nomination factor
    // A constant shows how much influence of the Nomination to validator's final score
    pub nf: u32,
    // Era Points factor
    // A constant shows how much influence of the Era Points to validator's final score
    pub epf: u32,
    //
    // Score: R * (CRF * (1 - CR) + NF * (1 / N) + EPF * (EEP / EEPA)) * SR
}

impl Default for NomineeCoefficients {
    fn default() -> Self {
        NomineeCoefficients {
            crf: 100,
            nf: 1000,
            epf: 10,
        }
    }
}

/// Info of the validator to be elected
#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, Default)]
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

// A value placed in storage that represents the current version of the NomineeElection storage. This value
// is used by the `on_runtime_upgrade` logic to determine whether we run storage migration logic.
// This should match directly with the semantic versions of the Rust crate.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
enum Releases {
    V0_0_0,
    V1_0_0,
}

impl Default for Releases {
    fn default() -> Self {
        Releases::V0_0_0
    }
}

pub mod migrations {
    use super::*;

    pub mod v1 {
        use super::*;

        pub fn migrate<T: Config>() -> Weight {
            log::info!("Migrating NomineeElection to Releases::V1_0_0");

            Coefficients::<T>::put(NomineeCoefficients::default());

            StorageVersion::<T>::put(Releases::V1_0_0);
            log::info!("Completed NomineeElection migration to Releases::V1_0_0");

            T::DbWeight::get().writes(2)
        }
    }
}

#[frame_support::pallet]
pub mod pallet {

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The origin which can update staking election coefficients
        type UpdateOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can feed whitelisted validators
        type WhitelistUpdateOrigin: EnsureOrigin<Self::Origin>;

        /// The maximum size of selected validators
        #[pallet::constant]
        type MaxValidators: Get<u32>;

        /// Approved accouts which can set validators
        type Members: SortedMembers<Self::AccountId>;
    }

    /// True if network has been upgraded to this version.
    /// Storage version of the pallet.
    ///
    /// This is set to v1.0.0 for new networks.
    #[pallet::storage]
    pub(crate) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

    /// Nominee election coefficients
    #[pallet::storage]
    #[pallet::getter(fn coefficients)]
    pub type Coefficients<T: Config> = StorageValue<_, NomineeCoefficients, ValueQuery>;

    /// Validators selected by off-chain client
    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> =
        StorageValue<_, BoundedVec<ValidatorInfo<T::AccountId>, T::MaxValidators>, ValueQuery>;

    /// Whitelisted validators selected by council
    #[pallet::storage]
    #[pallet::getter(fn whitelisted_validators)]
    pub type WhitelistedValidators<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, T::MaxValidators>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Coefficients updated (old_coefficients, new_coefficients)
        CoefficientsUpdated(NomineeCoefficients, NomineeCoefficients),
        /// Validator set updated (old_validators, new_validators)
        ValidorsUpdated(
            BoundedVec<ValidatorInfo<T::AccountId>, T::MaxValidators>,
            BoundedVec<ValidatorInfo<T::AccountId>, T::MaxValidators>,
        ),
        /// New validator was added to whitelist
        WhitelistedValidatorAdded(T::AccountId),
        /// New validator was removed from whitelist
        WhitelistedValidatorRemoved(T::AccountId),
        /// Whitelisted validators were reset
        WhitelistedValidatorsReset,
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The maximum number of validators exceeded
        MaxValidatorsExceeded,
        /// Feeded validators cannot be empty
        NoEmptyValidators,
        /// Invalid validators feeder
        BadValidatorsFeeder,
        /// Validator not found, thus not removed
        ValidatorNotFound,
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub coefficients: NomineeCoefficients,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            GenesisConfig {
                coefficients: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            StorageVersion::<T>::put(Releases::V1_0_0);
            Coefficients::<T>::put(self.coefficients.clone());
        }
    }

    #[cfg(feature = "std")]
    impl GenesisConfig {
        /// Direct implementation of `GenesisBuild::build_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn build_storage<T: Config>(&self) -> Result<sp_runtime::Storage, String> {
            <Self as frame_support::traits::GenesisBuild<T>>::build_storage(self)
        }

        /// Direct implementation of `GenesisBuild::assimilate_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn assimilate_storage<T: Config>(
            &self,
            storage: &mut sp_runtime::Storage,
        ) -> Result<(), String> {
            <Self as frame_support::traits::GenesisBuild<T>>::assimilate_storage(self, storage)
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            if StorageVersion::<T>::get() == Releases::V0_0_0 {
                migrations::v1::migrate::<T>()
            } else {
                T::DbWeight::get().reads(1)
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set nominee score coefficients
        #[pallet::weight(1000)]
        #[transactional]
        pub fn set_coefficients(
            origin: OriginFor<T>,
            coefficients: NomineeCoefficients,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            let old_coefficients = Self::coefficients();
            Coefficients::<T>::put(coefficients.clone());

            Self::deposit_event(Event::<T>::CoefficientsUpdated(
                old_coefficients,
                coefficients,
            ));
            Ok(())
        }

        /// Set selected validators
        ///
        /// If the validators passed are empty, return an error
        #[pallet::weight(1000)]
        #[transactional]
        pub fn set_validators(
            origin: OriginFor<T>,
            mut validators: Vec<ValidatorInfo<T::AccountId>>,
        ) -> DispatchResult {
            let feeder = ensure_signed(origin)?;
            ensure!(
                T::Members::contains(&feeder),
                Error::<T>::BadValidatorsFeeder
            );
            ensure!(!validators.is_empty(), Error::<T>::NoEmptyValidators);

            let whitelisted_validators = Self::whitelisted_validators();
            validators.retain(|v| whitelisted_validators.iter().all(|wv| wv != &v.address));

            let old_validators = Self::validators();
            let new_validators: BoundedVec<ValidatorInfo<T::AccountId>, T::MaxValidators> =
                validators
                    .try_into()
                    .map_err(|_| Error::<T>::MaxValidatorsExceeded)?;

            Validators::<T>::put(new_validators.clone());
            Self::deposit_event(Event::<T>::ValidorsUpdated(old_validators, new_validators));
            Ok(())
        }

        /// Add new validator to whitelist
        #[pallet::weight(1000)]
        #[transactional]
        pub fn add_whitelist_validator(
            origin: OriginFor<T>,
            validator_id: T::AccountId,
        ) -> DispatchResult {
            T::WhitelistUpdateOrigin::ensure_origin(origin)?;

            WhitelistedValidators::<T>::try_append(validator_id.clone())
                .map_err(|_| Error::<T>::MaxValidatorsExceeded)?;

            Self::deposit_event(Event::<T>::WhitelistedValidatorAdded(validator_id));
            Ok(())
        }

        /// Remove validator from whitelist
        #[pallet::weight(1000)]
        #[transactional]
        pub fn remove_whitelisted_validator(
            origin: OriginFor<T>,
            validator_id: T::AccountId,
        ) -> DispatchResult {
            T::WhitelistUpdateOrigin::ensure_origin(origin)?;

            if let Some(removed) = WhitelistedValidators::<T>::mutate(|vs| {
                vs.iter()
                    .position(|v| v == &validator_id)
                    .map(|idx| vs.remove(idx))
            }) {
                Self::deposit_event(Event::<T>::WhitelistedValidatorRemoved(removed));
                Ok(())
            } else {
                Err(Error::<T>::ValidatorNotFound.into())
            }
        }

        /// Reset whitelisted validators
        #[pallet::weight(1000)]
        #[transactional]
        pub fn reset_whitelisted_validators(origin: OriginFor<T>) -> DispatchResult {
            T::WhitelistUpdateOrigin::ensure_origin(origin)?;

            WhitelistedValidators::<T>::kill();

            Self::deposit_event(Event::<T>::WhitelistedValidatorsReset);
            Ok(())
        }
    }
}
