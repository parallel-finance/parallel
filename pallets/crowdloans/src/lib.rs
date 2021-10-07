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

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{
        fungibles::{Inspect, Mutate, Transfer},
        Get, Hooks, IsType,
    },
    Blake2_128Concat, PalletId,
};

mod crowdloan_structs;
use crowdloan_structs::{ContributionStrategy, ParaId, Vault, VaultPhase};

use frame_system::pallet_prelude::OriginFor;
pub use pallet::*;

use sp_runtime::{
    traits::{AtLeast32BitUnsigned, StaticLookup, Zero},
    DispatchError, FixedPointOperand,
};

pub type AssetIdOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type BalanceOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_system::ensure_root;

    #[pallet::config]
    pub trait Config<I: 'static = ()>:
        frame_system::Config
        + pallet_assets::Config<AssetId = AssetIdOf<Self, I>, Balance = BalanceOf<Self, I>>
    {
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

        /// Currency type for deposit/withdraw assets to/from crowdloan
        /// module
        type Assets: Transfer<Self::AccountId> + Inspect<Self::AccountId> + Mutate<Self::AccountId>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Vault is not in correct phase
        IncorrectVaultPhase,
        /// Vault shares are not new
        SharesNotNew,
        // Crowdload ParaId aready exists
        CrowdloanAlreadyExists,
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId", AssetIdOf<T, I> = "CurrencyId")]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// Create new vault
        /// [token, crowdloan, project_shares, currency_shares]
        VaultCreated(AssetIdOf<T, I>, ParaId, AssetIdOf<T, I>, AssetIdOf<T, I>),
    }

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<T::BlockNumber> for Pallet<T, I> {}

    #[pallet::pallet]
    pub struct Pallet<T, I = ()>(_);

    #[pallet::storage]
    #[pallet::getter(fn vaults)]
    pub type Vaults<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, ParaId, Vault<ParaId, AssetIdOf<T, I>, u32>, OptionQuery>;

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I>
    where
        BalanceOf<T, I>: FixedPointOperand,
        AssetIdOf<T, I>: AtLeast32BitUnsigned,
    {
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
            token: AssetIdOf<T, I>,
            crowdloan: ParaId,
            project_shares: AssetIdOf<T, I>,
            currency_shares: AssetIdOf<T, I>,
            contribution_strategy: ContributionStrategy<ParaId, AssetIdOf<T, I>, BalanceOf<T, I>>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // get
            let project_shares_issuance = T::Assets::total_issuance(project_shares);
            let currency_shares_issuance = T::Assets::total_issuance(currency_shares);

            // make sure both project_shares and currency_shares are new assets
            ensure!(
                project_shares_issuance == Zero::zero() && currency_shares_issuance == Zero::zero(),
                Error::<T, I>::SharesNotNew
            );

            // add new vault to vaults storage
            Vaults::<T, I>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure no similar vault already exists as identified by crowdloan
                ensure!(vault.is_none(), Error::<T, I>::CrowdloanAlreadyExists);

                // inialize new vault
                *vault = Some(crowdloan_structs::Vault {
                    project_shares,
                    currency_shares,
                    currency: currency_shares,
                    phase: VaultPhase::CollectingContributions,
                    contribution_strategy: ContributionStrategy::Placeholder(
                        crowdloan,
                        currency_shares,
                        0,
                    ),
                    contributed: 0,
                });

                Ok(())
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
            amount: BalanceOf<T, I>,
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
            amount: BalanceOf<T, I>,
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
            project_token: AssetIdOf<T, I>,
            total_to_distribute: BalanceOf<T, I>,
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
            amount: BalanceOf<T, I>,
        ) -> DispatchResult {
            unimplemented!();
        }

        ////
        //// 7. Refunding the contributed assets after auction success

        /// If a `crowdloan` succeeded and its slot expired, use `call` to
        /// claim back the funds lent to the parachain
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn slot_expired(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            ensure_root(origin)?;
            unimplemented!();
        }
    }
}
