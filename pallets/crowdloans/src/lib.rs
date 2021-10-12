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

use sp_runtime::traits::AccountIdConversion;
mod crowdloan_structs;
use crowdloan_structs::{ContributionStrategy, ParaId, Vault, VaultPhase};

use frame_system::{ensure_signed, RawOrigin};
use frame_system::pallet_prelude::OriginFor;

pub use pallet::*;

use sp_runtime::{
    traits::{AtLeast32BitUnsigned, Zero, One, StaticLookup, UniqueSaturatedInto},
    DispatchError, FixedPointOperand
};

pub type AssetIdOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type BalanceOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;


// TODO: test a simple trait
pub trait MyTrait {
    fn hello(self, crowdloan: ParaId) -> ();
}


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


        /// Specify all the AMMs we are routing between
        // type ContributionStrategyExecutor: ContributionStrategyExecutor<ParaId, AssetIdOf<Self, I>, BalanceOf<Self, I>>;


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
        // Crowdload ParaId does not exist
        CrowdloanDoesNotExists,
        // Amount is not enough
        InsufficientAmount,
        // Vault does not exist
        VaultDoesNotExist,
        // Vault contributed greater than issuance
        ContributedGreaterThanIssuance,
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
    pub type Vaults<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        ParaId,
        Vault<ParaId, AssetIdOf<T, I>, BalanceOf<T, I>>,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I>
    where
        BalanceOf<T, I>: FixedPointOperand,
        AssetIdOf<T, I>: AtLeast32BitUnsigned,
    {
        ////
        //// 1. Vaults Management

        /// Create a new vault via a governance decision
        /// - `currency` is the currency or token which needs to be deposited to fill
        ///   the vault and later participate in the crowdloans
        /// - `crowdloan` represents which crowdloan we are supporting on the relay
        ///   chain
        /// - `ctoken` is a new asset created for this vault to represent the shares
        ///   of the vault's contributors which will later be used for refunding their
        ///   contributions
        /// - `contribution_strategy` represents how we can contribute coins to the
        ///   crowdloan on the relay chain
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn create_vault(
            origin: OriginFor<T>,
            currency: AssetIdOf<T, I>,
            crowdloan: ParaId,
            ctoken: AssetIdOf<T, I>,
            // TODO
            // contribution_strategy: ContributionStrategy<ParaId, AssetIdOf<T, I>, BalanceOf<T, I>>,
            contribution_strategy: ContributionStrategy<ParaId, primitives::CurrencyId>,
        ) -> DispatchResult {
            // 1. EnsureOrigin
            ensure_root(origin)?;

            // 2. make sure both ctoken is a new asset (total_issuance == Zero::zero())
            let ctoken_issuance = T::Assets::total_issuance(ctoken);

            // make sure both project_shares and currency_shares are new assets
            ensure!(ctoken_issuance == Zero::zero(), Error::<T, I>::SharesNotNew);

            // create the ctoken asset
            pallet_assets::Pallet::<T>::force_create(
                RawOrigin::Root.into(),
                ctoken.unique_saturated_into(),
                T::Lookup::unlookup(Self::account_id()),
                true,
                One::one(),
            )?;


            // 3. make sure no similar vault already exists as identified by crowdloan
            // add new vault to vaults storage
            Vaults::<T, I>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure no similar vault already exists as identified by crowdloan
                ensure!(vault.is_none(), Error::<T, I>::CrowdloanAlreadyExists);

                // 4. mutate our storage to register a new vault
                // inialize new vault
                *vault = Some(crowdloan_structs::Vault {
                    ctoken: ctoken,
                    currency: currency,
                    phase: VaultPhase::CollectingContributions,
                    // contribution_strategy: ContributionStrategy::Placeholder(
                    //     crowdloan,
                    //     currency,
                    //     Zero::zero(),
                    // ),                    
                    contribution_strategy: ContributionStrategy::XCM,
                    contributed: Zero::zero(),
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
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1. Make sure crowdloan has a vault linked to it
            let vault = Self::vault(crowdloan)?;

            // 2. Make sure the vault.phase == CollectingContributions
            ensure!(
                vault.phase == VaultPhase::CollectingContributions,
                Error::<T, I>::IncorrectVaultPhase
            );

            // 3. Make sure origin has at least amount of vault.currency
            // get amount origin has
            let origin_currency_amount = T::Assets::balance(vault.currency, &who);

            ensure!(
                origin_currency_amount >= amount,
                Error::<T, I>::InsufficientAmount
            );

            // 4. Wire amount of vault.currency to the pallet's account id
            T::Assets::transfer(vault.currency, &who, &Self::account_id(), amount, true)?;

            // 5. Create amount of vault.ctoken to origin
            T::Assets::mint_into(vault.ctoken, &who, amount)?;

            Ok(())
        }

        ////
        //// 3. Triggering participation in a crowdloan

        /// Once a auction loan vault is expired, move the coins to the relay chain
        /// and participate in a relay chain crowdloan by using the call `call`.
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn participate(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. EnsureOrigin
            ensure_root(origin)?;

            Vaults::<T, I>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T, I>::CrowdloanDoesNotExists);

                if let Some(vault_contents) = vault {
                    // 2. Make sure vault.contributed is less than total_issuance(vault.currency_shares)
                    let vault_currency_issuance =
                        T::Assets::total_issuance(vault_contents.currency);
                    ensure!(
                        vault_contents.contributed < vault_currency_issuance,
                        Error::<T, I>::ContributedGreaterThanIssuance
                    );

                    // 3. Execute vault.contribution_strategy with parameters crowdloan,
                    // vault.currency and total_issuance(vault.ctoken) - vault.contributed

                    todo!();
                    // TODO: implement Executor trait correctly
                    // 
                    // vault_contents.contribution_strategy.hello(crowdloan);
                    // .execute(crowdloan, Zero::zero(), Zero::zero());

                    // 4. Set vault.contributed to total_issuance(vault.currency_shares)
                    vault_contents.contributed = vault_currency_issuance;

                    // update storage
                    *vault = Some(*vault_contents);
                }

                Ok(())
            })
        }

        ////
        //// 4. Handling Auction Closure

        /// Mark the associated vault as closed and stop accepting contributions for it
        #[pallet::weight(10_000)]
        #[allow(unused)]

        pub fn close(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. EnsureOrigin
            ensure_root(origin)?;

            Vaults::<T, I>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T, I>::CrowdloanDoesNotExists);

                if let Some(vault_contents) = vault {
                    // 2. Make sure vault.phase == VaultPhase::CollectingContributions
                    ensure!(
                        vault_contents.phase == VaultPhase::CollectingContributions,
                        Error::<T, I>::IncorrectVaultPhase
                    );

                    // 3. Change vault.phase to Closed
                    vault_contents.phase = VaultPhase::Closed;

                    // update storage
                    *vault = Some(*vault_contents);
                }
                Ok(())
            })
        }

        ////
        //// 5. Handling Failed Auctions

        /// If a `crowdloan` failed, get the coins back and mark the vault as ready
        /// for distribution
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn auction_failed(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. `EnsureOrigin`
            ensure_root(origin)?;

            Vaults::<T, I>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T, I>::CrowdloanDoesNotExists);

                if let Some(vault_contents) = vault {
                    // 2. Make sure `vault.phase == Closed`
                    ensure!(
                        vault_contents.phase == VaultPhase::Closed,
                        Error::<T, I>::IncorrectVaultPhase
                    );

                    // 3. Execute the `refund` function of the `contribution_strategy`
                    todo!();

                    // 4. Set `vault.phase` to `Failed`
                    vault_contents.phase = VaultPhase::Failed;

                    // update storage
                    *vault = Some(*vault_contents);
                }
                Ok(())
            })
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
            let who = ensure_signed(origin)?;

            Vaults::<T, I>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T, I>::CrowdloanDoesNotExists);

                if let Some(vault_contents) = vault {
                    // 1. Make sure `vault.phase == Failed` **or `Expired`** (more on that later)
                    ensure!(
                        vault_contents.phase == VaultPhase::Failed
                            || vault_contents.phase == VaultPhase::Expired,
                        Error::<T, I>::IncorrectVaultPhase
                    );

                    // 2. Make sure `origin` has at least `amount` of `vault.ctoken`
                    // get amount origin has
                    let origin_ctoken_amount =
                        <T as Config<I>>::Assets::balance(vault_contents.ctoken, &who);

                    ensure!(
                        origin_ctoken_amount >= amount,
                        Error::<T, I>::InsufficientAmount
                    );

                    // 3. Burns `amount` from `vault.ctoken`
                    T::Assets::burn_from(vault_contents.ctoken, &who, amount)?;

                    // 4. Wire `amount` of `vault.currency` from our account id to the caller
                    T::Assets::transfer(
                        vault_contents.currency,
                        &Self::account_id(),
                        &who,
                        amount,
                        false,
                    )?;
                }
                Ok(())
            })
        }

        ////
        //// 6. Refunding the contributed assets after auction success

        /// If a `crowdloan` succeeded and its slot expired, use `call` to
        /// claim back the funds lent to the parachain
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn slot_expired(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. `EnsureOrigin`
            ensure_root(origin)?;

            // 2. Execute the `withdraw` function of our `contribution_strategy`
            todo!();

            Vaults::<T, I>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T, I>::CrowdloanDoesNotExists);

                if let Some(vault_contents) = vault {
                    // 3. Modify `vault.phase` to `Expired

                    // 4. Set `vault.phase` to `Failed`
                    vault_contents.phase = VaultPhase::Expired;

                    // update storage
                    *vault = Some(*vault_contents);
                }
                Ok(())
            })
        }
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I>
where
    BalanceOf<T, I>: FixedPointOperand,
    AssetIdOf<T, I>: AtLeast32BitUnsigned,
{
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    // Returns a stored Vault.
    //
    // Returns `Err` if market does not exist.
    pub fn vault(
        crowdloan: ParaId,
    ) -> Result<Vault<ParaId, AssetIdOf<T, I>, BalanceOf<T, I>>, DispatchError> {
        // TODO
    // ) -> Result<Vault<ParaId, AssetIdOf<T, I>, BalanceOf<T, I>>, DispatchError> {
        Vaults::<T, I>::try_get(crowdloan).map_err(|_err| Error::<T, I>::VaultDoesNotExist.into())
    }
}