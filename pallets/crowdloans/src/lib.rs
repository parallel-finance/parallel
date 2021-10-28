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

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod crowdloan_structs;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::crowdloan_structs::*;
    use cumulus_primitives_core::ParaId;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{
            fungibles::{Inspect, Mutate, Transfer},
            Get,
        },
        transactional, Blake2_128Concat, PalletId,
    };
    use frame_system::ensure_signed;
    use frame_system::pallet_prelude::OriginFor;
    use primitives::{Balance, CurrencyId};
    use sp_runtime::{
        traits::{AccountIdConversion, Zero},
        DispatchError,
    };
    use xcm::latest::prelude::*;
    use xcm::DoubleEncoded;

    pub type AssetIdOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
    pub type BalanceOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Assets for deposit/withdraw assets to/from crowdloan account
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, Balance = Balance>;

        /// XCM message sender
        type XcmSender: SendXcm;

        /// Returns the parachain ID we are running with.
        #[pallet::constant]
        type SelfParaId: Get<ParaId>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Relay network
        #[pallet::constant]
        type RelayNetwork: Get<NetworkId>;

        /// Account derivative index
        #[pallet::constant]
        type DerivativeIndex: Get<u16>;

        /// The origin which can create vault
        type CreateVaultOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can pariticipate
        type PariticipateOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can close vault
        type CloseOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can call auction failed
        type AuctionFailedOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can call auction completed
        type AuctionCompletedOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can call slot expired
        type SlotExpiredOrigin: EnsureOrigin<Self::Origin>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emits when new vault created
        VaultCreated(ParaId, AssetIdOf<T>),
        /// Emits when user contributes amount to vault
        VaultContributed(ParaId, T::AccountId, BalanceOf<T>),
        /// Emits when vault particpates in crowdloan
        VaultParticipated(ParaId, BalanceOf<T>),
        /// Emits when vault is closed
        VaultClosed(ParaId),
        /// Emits when auction fails
        VaultAuctionFailed(ParaId),
        /// Emits when a user claims refund from vault
        VaultClaimRefund(ParaId, T::AccountId, BalanceOf<T>),
        /// Emits when a vault is expired
        VaultSlotExpired(ParaId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Vault is not in correct phase
        IncorrectVaultPhase,
        /// Crowdload ParaId aready exists
        CrowdloanAlreadyExists,
        /// Amount is not enough
        InsufficientBalance,
        /// Vault does not exist
        VaultDoesNotExist,
        /// Vault contributed greater than issuance
        ContributedGreaterThanIssuance,
        /// Xcm fees given are too low to execute on relaychain
        XcmFeesCompensationTooLow,
        /// Vault with specific ctoken already created
        CTokenVaultAlreadyCreated,
    }

    #[pallet::storage]
    #[pallet::getter(fn vaults)]
    pub type Vaults<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ParaId,
        Vault<ParaId, AssetIdOf<T>, BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub exchange_rate: usize,
        pub reserve_factor: usize,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                exchange_rate: 1,
                reserve_factor: 1,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {}
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        [u8; 32]: From<<T as frame_system::Config>::AccountId>,
    {
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
        pub fn create_vault(
            origin: OriginFor<T>,
            currency: AssetIdOf<T>,
            crowdloan: ParaId,
            ctoken: AssetIdOf<T>,
            contribution_strategy: ContributionStrategy<ParaId, CurrencyId, Balance>,
        ) -> DispatchResult {
            // 1. EnsureOrigin
            T::CreateVaultOrigin::ensure_origin(origin)?;

            // 2. make sure both ctoken is a new asset (total_issuance == Zero::zero())
            let ctoken_issuance = T::Assets::total_issuance(ctoken);

            // make sure both project_shares and currency_shares are new assets
            ensure!(
                ctoken_issuance == Zero::zero(),
                Error::<T>::CTokenVaultAlreadyCreated
            );

            // 3. make sure no similar vault already exists as identified by crowdloan
            // add new vault to vaults storage
            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure no similar vault already exists as identified by crowdloan
                ensure!(vault.is_none(), Error::<T>::CrowdloanAlreadyExists);

                // 4. mutate our storage to register a new vault
                // inialize new vault
                let new_vault =
                    crowdloan_structs::Vault::from((ctoken, currency, contribution_strategy));

                // store update
                *vault = Some(new_vault);

                // Emit event of trade with rate calculated
                Self::deposit_event(Event::<T>::VaultCreated(crowdloan, ctoken));

                Ok(())
            })
        }

        /// Contribute `amount` to the vault of `crowdloan` and receive some
        /// shares from it
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn contribute(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // 1. Make sure crowdloan has a vault linked to it
            let vault = Self::vault(crowdloan)?;

            // 2. Make sure the vault.phase == CollectingContributions
            ensure!(
                vault.phase == VaultPhase::CollectingContributions,
                Error::<T>::IncorrectVaultPhase
            );

            // 3. Make sure origin has at least amount of vault.currency (checked in transfer)
            // 4. Wire amount of vault.currency to the pallet's account id
            T::Assets::transfer(
                vault.relay_currency,
                &who,
                &Self::account_id(),
                amount,
                true,
            )
            .map_err(|_: DispatchError| Error::<T>::InsufficientBalance)?;

            // 5. Create amount of vault.ctoken to origin
            T::Assets::mint_into(vault.ctoken, &who, amount)?;

            // emit event
            Self::deposit_event(Event::<T>::VaultContributed(crowdloan, who, amount));

            Ok(().into())
        }

        /// Once a auction loan vault is expired, move the coins to the relay chain
        /// and participate in a relay chain crowdloan by using the call `call`.
        #[pallet::weight(10_000)]
        pub fn participate(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. EnsureOrigin
            T::PariticipateOrigin::ensure_origin(origin)?;

            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T>::VaultDoesNotExist);

                if let Some(vault_contents) = vault {
                    // 2. Make sure vault.contributed is less than total_issuance(vault.currency_shares)
                    let vault_ctoken_issuance = T::Assets::total_issuance(vault_contents.ctoken);

                    ensure!(
                        vault_contents.contributed < vault_ctoken_issuance,
                        Error::<T>::ContributedGreaterThanIssuance
                    );

                    // 3. Execute vault.contribution_strategy with parameters crowdloan,
                    // cannot underflow because we checked that vault_contents.contributed < vault_ctoken_issuance
                    let amount = vault_ctoken_issuance - vault_contents.contributed;

                    vault_contents.contribution_strategy.execute(
                        crowdloan,
                        vault_contents.relay_currency,
                        amount,
                    )?;

                    // 4. Set vault.contributed to total_issuance(vault.currency_shares)
                    vault_contents.contributed = vault_ctoken_issuance;

                    // update storage
                    *vault = Some(*vault_contents);

                    // Emit event of trade with rate calculated
                    Self::deposit_event(Event::<T>::VaultParticipated(crowdloan, amount));
                }

                Ok(())
            })
        }

        /// Mark the associated vault as closed and stop accepting contributions for it
        #[pallet::weight(10_000)]
        pub fn close(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. EnsureOrigin
            T::CloseOrigin::ensure_origin(origin)?;

            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T>::VaultDoesNotExist);

                if let Some(vault_contents) = vault {
                    // 2. Make sure vault.phase == VaultPhase::CollectingContributions
                    ensure!(
                        vault_contents.phase == VaultPhase::CollectingContributions,
                        Error::<T>::IncorrectVaultPhase
                    );

                    // 3. Change vault.phase to Closed
                    vault_contents.phase = VaultPhase::Closed;

                    // update storage
                    *vault = Some(*vault_contents);

                    // Emit event of trade with rate calculated
                    Self::deposit_event(Event::<T>::VaultClosed(crowdloan));
                }
                Ok(())
            })
        }

        /// If a `crowdloan` failed, get the coins back and mark the vault as ready
        /// for distribution
        #[pallet::weight(10_000)]
        pub fn auction_failed(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. `EnsureOrigin`
            T::AuctionFailedOrigin::ensure_origin(origin)?;

            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T>::VaultDoesNotExist);

                if let Some(vault_contents) = vault {
                    // 2. Make sure `vault.phase == Closed`
                    ensure!(
                        vault_contents.phase == VaultPhase::Closed,
                        Error::<T>::IncorrectVaultPhase
                    );

                    // 3. Execute the `refund` function of the `contribution_strategy`
                    vault_contents
                        .contribution_strategy
                        .refund(crowdloan, vault_contents.relay_currency)?;

                    // 4. Set `vault.phase` to `Failed`
                    vault_contents.phase = VaultPhase::Failed;

                    // update storage
                    *vault = Some(*vault_contents);

                    // Emit event of trade with rate calculated
                    Self::deposit_event(Event::<T>::VaultAuctionFailed(crowdloan));
                }
                Ok(())
            })
        }

        /// If a `crowdloan` failed, claim back your share of the assets you
        /// contributed
        #[pallet::weight(10_000)]
        pub fn claim_refund(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T>::VaultDoesNotExist);

                if let Some(vault_contents) = vault {
                    // 1. Make sure `vault.phase == Failed` **or `Expired`** (more on that later)
                    ensure!(
                        vault_contents.phase == VaultPhase::Failed
                            || vault_contents.phase == VaultPhase::Expired,
                        Error::<T>::IncorrectVaultPhase
                    );

                    // 2. Make sure `origin` has at least `amount` of `vault.ctoken`
                    // get amount origin has
                    let origin_ctoken_amount =
                        <T as Config>::Assets::balance(vault_contents.ctoken, &who);

                    ensure!(
                        origin_ctoken_amount >= amount,
                        Error::<T>::InsufficientBalance
                    );

                    // 3. Burns `amount` from `vault.ctoken`
                    T::Assets::burn_from(vault_contents.ctoken, &who, amount)?;

                    // 4. Wire `amount` of `vault.currency` from our account id to the caller
                    T::Assets::transfer(
                        vault_contents.relay_currency,
                        &Self::account_id(),
                        &who,
                        amount,
                        false,
                    )?;

                    // Emit event of trade with rate calculated
                    Self::deposit_event(Event::<T>::VaultClaimRefund(crowdloan, who, amount));
                }
                Ok(())
            })
        }

        /// If a `crowdloan` succeeded and its slot expired, use `call` to
        /// claim back the funds lent to the parachain
        #[pallet::weight(10_000)]
        pub fn slot_expired(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. `EnsureOrigin`
            T::SlotExpiredOrigin::ensure_origin(origin)?;

            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T>::VaultDoesNotExist);

                if let Some(vault_contents) = vault {
                    // 2. Execute the `withdraw` function of our `contribution_strategy`
                    vault_contents
                        .contribution_strategy
                        .withdraw(crowdloan, vault_contents.relay_currency)?;

                    // 3. Modify `vault.phase` to `Expired
                    vault_contents.phase = VaultPhase::Expired;

                    // update storage
                    *vault = Some(*vault_contents);

                    // Emit event of trade with rate calculated
                    Self::deposit_event(Event::<T>::VaultSlotExpired(crowdloan));
                }
                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T>
    where
        [u8; 32]: From<<T as frame_system::Config>::AccountId>,
    {
        /// Crowdloan pool account
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        /// Parachain sovereign account
        pub fn para_account_id() -> T::AccountId {
            T::SelfParaId::get().into_account()
        }

        // Returns a stored Vault.
        //
        // Returns `Err` if market does not exist.
        pub fn vault(
            crowdloan: ParaId,
        ) -> Result<Vault<ParaId, AssetIdOf<T>, BalanceOf<T>>, DispatchError> {
            Vaults::<T>::try_get(crowdloan).map_err(|_err| Error::<T>::VaultDoesNotExist.into())
        }

        #[allow(dead_code)]
        fn ump_transact(call: DoubleEncoded<()>, weight: Weight) -> Result<Xcm<()>, DispatchError> {
            // let fees = Self::xcm_fees_compensation();
            let fees = 1_000_000_000_000;
            ensure!(!fees.is_zero(), Error::<T>::XcmFeesCompensationTooLow);

            // let account_id = Self::account_id();
            let asset: MultiAsset = (MultiLocation::here(), fees).into();

            Ok(Xcm(vec![
                WithdrawAsset(MultiAssets::from(asset.clone())),
                BuyExecution {
                    fees: asset.clone(),
                    weight_limit: Unlimited,
                },
                Transact {
                    origin_type: OriginKind::SovereignAccount,
                    require_weight_at_most: weight,
                    call,
                },
                RefundSurplus,
                DepositAsset {
                    assets: asset.into(),
                    max_assets: 1,
                    beneficiary: MultiLocation {
                        parents: 1,
                        interior: X1(AccountId32 {
                            network: NetworkId::Any,
                            id: Self::para_account_id().into(),
                        }),
                    },
                },
            ]))
        }
    }
}

// impl<T: Config> ExchangeRateProvider for Pallet<T> {
//     fn get_exchange_rate() -> Rate {
//         ExchangeRate::<T>::get()
//     }
// }
