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

//! # Crowdloans pallet

#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[macro_use]
extern crate primitives;

pub mod types;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use crate::types::*;

    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{
            fungibles::{Inspect, Mutate, Transfer},
            Get,
        },
        transactional, Blake2_128Concat, PalletId,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use primitives::{ump::XcmWeightMisc, Balance, CurrencyId, ParaId, Ratio};
    use sp_runtime::{
        traits::{AccountIdConversion, Convert, Zero},
        ArithmeticError, DispatchError, MultiSignature,
    };
    use sp_std::cmp::min;
    use sp_std::vec;
    use xcm::{latest::prelude::*, DoubleEncoded};

    use crate::weights::WeightInfo;

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
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        /// XCM message sender
        type XcmSender: SendXcm;

        /// Returns the parachain ID we are running with.
        #[pallet::constant]
        type SelfParaId: Get<ParaId>;

        /// Relay network
        #[pallet::constant]
        type RelayNetwork: Get<NetworkId>;

        /// Relay currency
        #[pallet::constant]
        type RelayCurrency: Get<AssetIdOf<Self>>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Convert `T::AccountId` to `MultiLocation`.
        type AccountIdToMultiLocation: Convert<Self::AccountId, MultiLocation>;

        /// Account on relaychain for receiving refunded fees
        type RefundLocation: Get<Self::AccountId>;

        /// Max reserved token amount for paying xcm fees
        type MaxReserves: Get<BalanceOf<Self>>;

        /// The origin which can update reserve_factor, xcm_fees_compensation etc
        type UpdateOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can create vault
        type CreateVaultOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can close vault
        type CloseOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can call auction failed
        type AuctionFailedOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can call auction completed
        type AuctionCompletedOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can call slot expired
        type SlotExpiredOrigin: EnsureOrigin<Self::Origin>;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New vault was created
        VaultCreated(ParaId, AssetIdOf<T>),
        /// User contributed amount to vault
        VaultContributed(ParaId, T::AccountId, BalanceOf<T>),
        /// Vault was closed
        VaultClosed(ParaId),
        /// Auction failed
        VaultAuctionFailed(ParaId),
        /// A user claimed refund from vault
        VaultClaimRefund(ParaId, T::AccountId, BalanceOf<T>),
        /// A vault was expired
        VaultSlotExpired(ParaId),
        /// ReserveFactor was updated
        ReserveFactorUpdated(Ratio),
        /// Xcm weight in BuyExecution message
        XcmWeightUpdated(XcmWeightMisc<Weight>),
        /// Compensation for extrinsics on relaychain was set to new value
        XcmFeesCompensationUpdated(BalanceOf<T>),
        /// Sent crowdloan.contribute call to relaychain
        Contributing(ParaId, BalanceOf<T>, Option<MultiSignature>),
        /// Sent crowdloan.withdraw call to relaychain
        Withdrawing(ParaId, T::AccountId),
        /// Reserves added
        ReservesAdded(BalanceOf<T>),
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
        /// Vault with specific ctoken already created
        CTokenVaultAlreadyCreated,
        /// Xcm message send failure
        SendXcmError,
    }

    #[pallet::storage]
    #[pallet::getter(fn vaults)]
    pub type Vaults<T: Config> = StorageMap<_, Blake2_128Concat, ParaId, Vault<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn reserve_factor)]
    pub type ReserveFactor<T: Config> = StorageValue<_, Ratio, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_reserves)]
    pub type TotalReserves<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn xcm_fees_compensation)]
    pub type XcmFeesCompensation<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn xcm_weight)]
    pub type XcmWeight<T: Config> = StorageValue<_, XcmWeightMisc<Weight>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub reserve_factor: Ratio,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                reserve_factor: Ratio::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            ReserveFactor::<T>::put(self.reserve_factor);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new vault via a governance decision
        /// - `crowdloan` represents which crowdloan we are supporting on the relay
        ///   chain
        /// - `ctoken` is a new asset created for this vault to represent the shares
        ///   of the vault's contributors which will later be used for refunding their
        ///   contributions
        /// - `contribution_strategy` represents how we can contribute coins to the
        ///   crowdloan on the relay chain
        #[pallet::weight(<T as Config>::WeightInfo::create_vault())]
        #[transactional]
        pub fn create_vault(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            ctoken: AssetIdOf<T>,
            contribution_strategy: ContributionStrategy,
        ) -> DispatchResult {
            // 1. EnsureOrigin
            T::CreateVaultOrigin::ensure_origin(origin)?;

            // 2. make sure both ctoken is a new asset (total_issuance == Zero::zero())
            let ctoken_issuance = T::Assets::total_issuance(ctoken);

            // make sure both project_shares and currency_shares are new assets
            ensure!(
                ctoken_issuance.is_zero(),
                Error::<T>::CTokenVaultAlreadyCreated
            );

            // 3. make sure no similar vault already exists as identified by crowdloan
            // add new vault to vaults storage
            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure no similar vault already exists as identified by crowdloan
                ensure!(vault.is_none(), Error::<T>::CrowdloanAlreadyExists);

                // 4. mutate our storage to register a new vault
                // inialize new vault
                let new_vault = Vault::from((ctoken, contribution_strategy));

                // store update
                *vault = Some(new_vault);

                // Emit event of trade with rate calculated
                Self::deposit_event(Event::<T>::VaultCreated(crowdloan, ctoken));

                Ok(())
            })
        }

        /// Contribute `amount` to the vault of `crowdloan` and receive some
        /// shares from it
        #[pallet::weight(<T as Config>::WeightInfo::contribute())]
        #[transactional]
        pub fn contribute(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // 1. Make sure crowdloan has a vault linked to it
            let mut vault = Self::vault(crowdloan)?;

            // 2. Make sure the vault.phase == CollectingContributions
            ensure!(
                vault.phase == VaultPhase::CollectingContributions,
                Error::<T>::IncorrectVaultPhase
            );

            let reserves = min(
                Self::reserve_factor().mul_floor(amount),
                T::MaxReserves::get(),
            );
            TotalReserves::<T>::try_mutate(|b| -> DispatchResult {
                *b = b.checked_add(reserves).ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;

            // 3. Make sure origin has at least amount of vault.currency (checked in transfer)
            // 4. Wire amount of vault.currency to the pallet's account id
            T::Assets::transfer(
                T::RelayCurrency::get(),
                &who,
                &Self::account_id(),
                amount,
                false,
            )
            .map_err(|_: DispatchError| Error::<T>::InsufficientBalance)?;

            let amount = amount
                .checked_sub(reserves)
                .ok_or(ArithmeticError::Underflow)?;

            // 5. Create amount of vault.ctoken to origin
            T::Assets::mint_into(vault.ctoken, &who, amount)?;

            vault
                .contribution_strategy
                .contribute::<T>(crowdloan, amount)?;

            vault.contributed = vault
                .contributed
                .checked_add(amount)
                .ok_or(ArithmeticError::Overflow)?;

            Vaults::<T>::insert(crowdloan, vault);

            // emit event
            Self::deposit_event(Event::<T>::VaultContributed(crowdloan, who, amount));

            Ok(().into())
        }

        /// Mark the associated vault as closed and stop accepting contributions for it
        #[pallet::weight(<T as Config>::WeightInfo::close())]
        #[transactional]
        pub fn close(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. EnsureOrigin
            T::CloseOrigin::ensure_origin(origin)?;

            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                let mut vault_contents = vault.as_mut().ok_or(Error::<T>::VaultDoesNotExist)?;

                // 2. Make sure vault.phase == VaultPhase::CollectingContributions
                ensure!(
                    vault_contents.phase == VaultPhase::CollectingContributions,
                    Error::<T>::IncorrectVaultPhase
                );

                // 3. Change vault.phase to Closed
                vault_contents.phase = VaultPhase::Closed;

                // Emit event of trade with rate calculated
                Self::deposit_event(Event::<T>::VaultClosed(crowdloan));

                Ok(())
            })
        }

        /// If a `crowdloan` failed, get the coins back and mark the vault as ready
        /// for distribution
        #[pallet::weight(<T as Config>::WeightInfo::auction_failed())]
        #[transactional]
        pub fn auction_failed(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. `EnsureOrigin`
            T::AuctionFailedOrigin::ensure_origin(origin)?;

            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                let mut vault_contents = vault.as_mut().ok_or(Error::<T>::VaultDoesNotExist)?;

                // 2. Make sure `vault.phase == Closed`
                ensure!(
                    vault_contents.phase == VaultPhase::Closed,
                    Error::<T>::IncorrectVaultPhase
                );

                // 3. Execute the `refund` function of the `contribution_strategy`
                vault_contents
                    .contribution_strategy
                    .withdraw::<T>(crowdloan, vault_contents.contributed)?;

                // 4. Set `vault.phase` to `Failed`
                vault_contents.phase = VaultPhase::Failed;

                // Emit event of trade with rate calculated
                Self::deposit_event(Event::<T>::VaultAuctionFailed(crowdloan));

                Ok(())
            })
        }

        /// If a `crowdloan` failed or expired, claim back your share of the assets you
        /// contributed
        #[pallet::weight(<T as Config>::WeightInfo::claim_refund())]
        #[transactional]
        pub fn claim_refund(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                let vault_contents = vault.as_mut().ok_or(Error::<T>::VaultDoesNotExist)?;

                // 1. Make sure `vault.phase == Failed` **or `Expired`** (more on that later)
                ensure!(
                    vault_contents.phase == VaultPhase::Failed
                        || vault_contents.phase == VaultPhase::Expired,
                    Error::<T>::IncorrectVaultPhase
                );

                // 2. Make sure `origin` has at least `amount` of `vault.ctoken`
                // get amount origin has
                let origin_ctoken_amount =
                    <T as Config>::Assets::reducible_balance(vault_contents.ctoken, &who, false);

                ensure!(
                    origin_ctoken_amount >= amount,
                    Error::<T>::InsufficientBalance
                );

                // 3. Burns `amount` from `vault.ctoken`
                T::Assets::burn_from(vault_contents.ctoken, &who, amount)?;

                // 4. Wire `amount` of `vault.currency` from our account id to the caller
                T::Assets::transfer(
                    T::RelayCurrency::get(),
                    &Self::account_id(),
                    &who,
                    amount,
                    false,
                )?;

                // Emit event of trade with rate calculated
                Self::deposit_event(Event::<T>::VaultClaimRefund(crowdloan, who, amount));

                Ok(())
            })
        }

        /// If a `crowdloan` succeeded and its slot expired, use `call` to
        /// claim back the funds lent to the parachain
        #[pallet::weight(<T as Config>::WeightInfo::slot_expired())]
        #[transactional]
        pub fn slot_expired(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. `EnsureOrigin`
            T::SlotExpiredOrigin::ensure_origin(origin)?;

            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                let mut vault_contents = vault.as_mut().ok_or(Error::<T>::VaultDoesNotExist)?;

                ensure!(
                    vault_contents.phase == VaultPhase::Closed,
                    Error::<T>::IncorrectVaultPhase
                );

                // 2. Execute the `withdraw` function of our `contribution_strategy`
                vault_contents
                    .contribution_strategy
                    .withdraw::<T>(crowdloan, vault_contents.contributed)?;

                // 3. Modify `vault.phase` to `Expired
                vault_contents.phase = VaultPhase::Expired;

                // Emit event of trade with rate calculated
                Self::deposit_event(Event::<T>::VaultSlotExpired(crowdloan));

                Ok(())
            })
        }

        #[pallet::weight(<T as Config>::WeightInfo::update_reserve_factor())]
        #[transactional]
        pub fn update_reserve_factor(
            origin: OriginFor<T>,
            reserve_factor: Ratio,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            ReserveFactor::<T>::mutate(|v| *v = reserve_factor);
            Self::deposit_event(Event::<T>::ReserveFactorUpdated(reserve_factor));
            Ok(().into())
        }

        #[pallet::weight(<T as Config>::WeightInfo::update_xcm_fees_compensation())]
        #[transactional]
        pub fn update_xcm_fees_compensation(
            origin: OriginFor<T>,
            #[pallet::compact] fees: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            XcmFeesCompensation::<T>::mutate(|v| *v = fees);
            Self::deposit_event(Event::<T>::XcmFeesCompensationUpdated(fees));
            Ok(().into())
        }

        #[pallet::weight(<T as Config>::WeightInfo::update_xcm_weight())]
        #[transactional]
        pub fn update_xcm_weight(
            origin: OriginFor<T>,
            xcm_weight_misc: XcmWeightMisc<Weight>,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            XcmWeight::<T>::mutate(|v| *v = xcm_weight_misc);
            Self::deposit_event(Event::<T>::XcmWeightUpdated(xcm_weight_misc));
            Ok(().into())
        }

        #[pallet::weight(<T as Config>::WeightInfo::add_reserves())]
        #[transactional]
        pub fn add_reserves(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            T::Assets::transfer(
                T::RelayCurrency::get(),
                &who,
                &Self::account_id(),
                amount,
                false,
            )?;
            TotalReserves::<T>::try_mutate(|b| -> DispatchResult {
                *b = b.checked_add(amount).ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;
            Self::deposit_event(Event::<T>::ReservesAdded(amount));
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Crowdloan pool account
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        /// Parachain sovereign account
        pub fn para_account_id() -> T::AccountId {
            T::SelfParaId::get().into_account()
        }

        fn vault(crowdloan: ParaId) -> Result<Vault<T>, DispatchError> {
            Vaults::<T>::try_get(crowdloan).map_err(|_err| Error::<T>::VaultDoesNotExist.into())
        }

        pub(crate) fn ump_transact(
            call: DoubleEncoded<()>,
            weight: Weight,
        ) -> Result<Xcm<()>, DispatchError> {
            let fees = Self::xcm_fees_compensation();
            let account_id = Self::account_id();
            let relay_currency = T::RelayCurrency::get();
            let asset: MultiAsset = (MultiLocation::here(), fees).into();

            T::Assets::burn_from(relay_currency, &account_id, fees)?;

            TotalReserves::<T>::try_mutate(|b| -> DispatchResult {
                *b = b.checked_sub(fees).ok_or(ArithmeticError::Underflow)?;
                Ok(())
            })?;

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
                    beneficiary: T::AccountIdToMultiLocation::convert(T::RefundLocation::get()),
                },
            ]))
        }
    }
}
