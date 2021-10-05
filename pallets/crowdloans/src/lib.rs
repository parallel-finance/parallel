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

//! # Crowdloans
//!

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod crowdloan_structs;
use crowdloan_structs::{ClaimStrategy, ContributionStrategy, ParaId, Vault, VaultPhase};

use frame_support::traits::tokens::fungibles;
use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{tokens::fungibles::Inspect, Get, IsType},
    Blake2_128Concat, PalletId,
};
use primitives::{currency::CurrencyId::Asset, AssetId, Balance};

use frame_system::pallet_prelude::*;

use frame_system::ensure_root;
use primitives::currency::CurrencyId;
use sp_arithmetic::traits::Zero;

use sp_runtime::{traits::StaticLookup, DispatchError};

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub type ParaIdOf = ParaId;

pub type AssetIdOf<T> = <<T as Config>::CrowdloanCurrency as fungibles::Inspect<
    <T as frame_system::Config>::AccountId,
>>::AssetId;

pub type BalanceOf<T> = <<T as Config>::CrowdloanCurrency as fungibles::Inspect<
    <T as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Currency type for deposit/withdraw assets to/from crowdloan
        /// module
        type CrowdloanCurrency: fungibles::Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + fungibles::Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + fungibles::Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        // type ParaId: Get<ParaId>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Vault is not in correct phase
        IncorrectVaultPhase,
        /// Vault shares are not new
        SharesNotNew,
        // Crowdload ParaId aready exists
        CrowdloanAlreadyExists,
    }

    #[pallet::event]
    pub enum Event<T: Config> {
        /// Create new vault
        /// [token, crowdloan, project_shares, currency_shares]
        VaultCreated(CurrencyId, ParaId, AssetId, AssetId),
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn vaults)]
    pub type Vaults<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ParaIdOf,
        Vault<ParaId, AssetIdOf<T>, BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        ////
        //// 1. Vaults Management

        /// - `token` is the currency or token which needs to be deposited to fill
        ///   the vault and later participate in the crowdloans
        /// - `crowdloan` represents which crowdloan we are supporting
        /// - `project_shares` and `currency_shares` are new assets created for this
        ///   vault to represent the shares of the vault's contributors
        /// - `until` sets the vault's expiration block, a vault must be "expired"
        ///   until it can be used to participate in its crowdloan
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn create_vault(
            origin: OriginFor<T>,
            token: AssetIdOf<T>,
            crowdloan: ParaId,
            project_shares: AssetIdOf<T>,
            currency_shares: AssetIdOf<T>,
            contribution_strategy: ContributionStrategy<ParaId, AssetIdOf<T>, BalanceOf<T>>,
            claim_strategy: ClaimStrategy<ParaId>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // get
            let project_shares_issuance = T::CrowdloanCurrency::total_issuance(project_shares);
            let currency_shares_issuance = T::CrowdloanCurrency::total_issuance(currency_shares);

            // make sure both project_shares and currency_shares are new assets
            ensure!(
                project_shares_issuance == Zero::zero() && currency_shares_issuance == Zero::zero(),
                Error::<T>::SharesNotNew
            );

            // make sure no similar vault already exists as identified by crowdloan
            ensure!(
                !Vaults::<T>::contains_key(&crowdloan),
                Error::<T>::CrowdloanAlreadyExists
            );

            // add new vault to vaults storage
            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // inialize new vault
                Ok(*vault = Some(crowdloan_structs::Vault {
                    project_shares: project_shares,
                    currency_shares: currency_shares,
                    currency: currency_shares,
                    phase: VaultPhase::CollectingContributions,
                    contribution_strategy: ContributionStrategy::Placeholder(
                        crowdloan,
                        currency_shares,
                        0,
                    ),
                    claim_strategy: ClaimStrategy::Placeholder(crowdloan),
                    contributed: 0,
                }))
            })
        }

        ////
        //// 2. Contribution to Vaults

        /// Contribute `amount` to the vault of `crowdloan` and receive some
        /// shares from it

        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn contribute(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            amount: Balance,
            ptokens_receiver: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            unimplemented!();
        }

        ////
        //// 3. Triggering participation in a crowdloan

        /// Once a auction loan vault is expired, move the coins to the relay chain
        /// and participate in a relay chain crowdloan by using the call `call`.
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn participate(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            ensure_root(origin)?;
            unimplemented!();
        }

        ////
        //// 4. Handling Auction Closure

        /// Mark the associated vault as closed and stop accepting contributions for it
        #[pallet::weight(10_000)]
        #[allow(unused)]

        pub fn close(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            ensure_root(origin)?;
            unimplemented!();
        }

        ////
        //// 5. Handling Failed Auctions

        /// If a `crowdloan` failed, get the coins back and mark the vault as ready
        /// for distribution
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn auction_failed(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            ensure_root(origin)?;
            unimplemented!();
        }

        /// If a `crowdloan` failed, claim back your share of the assets you
        /// contributed
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn claim_refund(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            amount: Balance,
        ) -> DispatchResult {
            unimplemented!();
        }

        ////
        //// 6. Distributing Project Tokens

        /// If a `crowdloan` succeeded, use `call` to receive or claim the
        /// project tokens, can be called many times
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn auction_completed(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            project_token: AssetIdOf<T>,
            total_to_distribute: BalanceOf<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            unimplemented!();
        }

        /// If a `crowdloan` succeeded, claim your derivative project tokens that can
        /// later be exchanged to the actual project token
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn claim_derivative(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            unimplemented!();
        }

        /// If a `crowdloan` succeeded, claim your share of the project tokens
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn claim(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            unimplemented!();
        }

        /// Exchange your derivative pTokens for the actual project tokens
        /// if there are some in the pool
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn claim_project_tokens(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            unimplemented!();
        }

        ////
        //// 7. Refunding the contributed assets after auction success

        /// If a `crowdloan` succeeded and its slot expired, use `call` to
        /// claim back the funds lent to the parachain
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn slot_expired(
            origin: OriginFor<T>,
            crowdloan: ParaId,
        ) -> DispatchResult {
            ensure_root(origin)?;
            unimplemented!();
        }
    }
}
