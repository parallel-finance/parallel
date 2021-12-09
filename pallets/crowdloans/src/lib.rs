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
//!
//! ## Overview
//!
//! Support your favorite parachains' crowdloans while releasing liquidity via crowdloans derivatives

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
        require_transactional,
        traits::{
            fungibles::{Inspect, Mutate, Transfer},
            Get,
        },
        transactional, Blake2_128Concat, PalletId,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use primitives::{ump::*, Balance, CurrencyId, ParaId, Ratio};
    use sp_runtime::{
        traits::{AccountIdConversion, Convert, StaticLookup, Zero},
        ArithmeticError, DispatchError,
    };
    use sp_std::vec::Vec;
    use xcm::latest::prelude::*;

    use crate::weights::WeightInfo;
    use pallet_xcm_helper::XcmHelper;

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub type AssetIdOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
    pub type BalanceOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_xcm::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Origin: IsType<<Self as frame_system::Config>::Origin>
            + Into<Result<pallet_xcm::Origin, <Self as Config>::Origin>>;

        type Call: IsType<<Self as pallet_xcm::Config>::Call> + From<Call<Self>>;

        /// Assets for deposit/withdraw assets to/from crowdloan account
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        /// Returns the parachain ID we are running with.
        #[pallet::constant]
        type SelfParaId: Get<ParaId>;

        /// Relay currency
        #[pallet::constant]
        type RelayCurrency: Get<AssetIdOf<Self>>;

        /// Pallet account for collecting contributions
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Convert `T::AccountId` to `MultiLocation`.
        type AccountIdToMultiLocation: Convert<Self::AccountId, MultiLocation>;

        /// Account on relaychain for receiving refunded fees
        #[pallet::constant]
        type RefundLocation: Get<Self::AccountId>;

        /// Xcm fees payer
        #[pallet::constant]
        type XcmFeesPayer: Get<PalletId>;

        /// Minimum contribute amount
        #[pallet::constant]
        type MinContribution: Get<BalanceOf<Self>>;

        /// The origin which can update reserve_factor, xcm_fees etc
        type UpdateOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can toggle vrf delay
        type VrfDelayOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can create vault
        type CreateVaultOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can close/reopen vault
        type OpenCloseOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can call auction failed
        type AuctionFailedOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can call slot expired
        type SlotExpiredOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can add/reduce reserves.
        type ReserveOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// Weight information
        type WeightInfo: WeightInfo;

        /// To expose XCM helper functions
        type XCM: XcmHelper<BalanceOf<Self>, AssetIdOf<Self>, Self::AccountId>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New vault was created
        VaultCreated(ParaId, AssetIdOf<T>),
        /// User contributed amount to vault
        VaultContributed(ParaId, T::AccountId, BalanceOf<T>, Vec<u8>),
        /// Vault was opened
        VaultOpened(ParaId, BalanceOf<T>),
        /// Vault was closed
        VaultClosed(ParaId),
        /// Vault was reopened
        VaultReOpened(ParaId),
        /// Auction failed
        VaultAuctionFailed(ParaId),
        /// A user claimed refund from vault
        VaultClaimRefund(AssetIdOf<T>, T::AccountId, BalanceOf<T>),
        /// A vault was expired
        VaultSlotExpired(ParaId),
        /// ReserveFactor was updated
        ReserveFactorUpdated(Ratio),
        /// Xcm weight in BuyExecution message
        XcmWeightUpdated(XcmWeightMisc<Weight>),
        /// Fees for extrinsics on relaychain were set to new value
        XcmFeesUpdated(BalanceOf<T>),
        /// Reserves added
        ReservesAdded(T::AccountId, BalanceOf<T>),
        /// Vrf delay toggled
        VrfDelayToggled(bool),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Vault is not in correct phase
        IncorrectVaultPhase,
        /// Crowdloan ParaId aready exists
        CrowdloanAlreadyExists,
        /// Amount is not enough
        InsufficientBalance,
        /// Vault does not exist
        VaultDoesNotExist,
        /// Ctoken already taken by another vault
        CTokenAlreadyTaken,
        /// ParaId already taken by another vault
        ParaIdAlreadyTaken,
        /// No contributions allowed during the VRF delay
        VrfDelayInProgress,
    }

    #[pallet::storage]
    #[pallet::getter(fn vaults)]
    pub type Vaults<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, ParaId, Blake2_128Concat, u32, Vault<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn reserve_factor)]
    pub type ReserveFactor<T: Config> = StorageValue<_, Ratio, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn is_vrf)]
    pub type IsVrfDelayInProgress<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn ctokens_registry)]
    pub type CTokensRegistry<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetIdOf<T>, (ParaId, u32), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_index)]
    pub type BatchIndexes<T: Config> = StorageMap<_, Blake2_128Concat, ParaId, u32, OptionQuery>;

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
        #[pallet::weight(<T as Config>::WeightInfo::create_vault())]
        #[transactional]
        pub fn create_vault(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            ctoken: AssetIdOf<T>,
            contribution_strategy: ContributionStrategy,
            xcm_fees_payment_strategy: XcmFeesPaymentStrategy,
        ) -> DispatchResult {
            T::CreateVaultOrigin::ensure_origin(origin)?;

            let ctoken_issuance = T::Assets::total_issuance(ctoken);
            ensure!(
                ctoken_issuance.is_zero() && !CTokensRegistry::<T>::contains_key(ctoken),
                Error::<T>::CTokenAlreadyTaken
            );

            if let Some(vault) = Self::current_vault(crowdloan) {
                if vault.phase != VaultPhase::Failed && vault.phase != VaultPhase::Expired {
                    return Err(DispatchError::from(Error::<T>::ParaIdAlreadyTaken));
                }
            }

            let next_index = Self::next_index(crowdloan);
            ensure!(
                !Vaults::<T>::contains_key(crowdloan, next_index),
                Error::<T>::CrowdloanAlreadyExists
            );

            let new_vault = Vault::new(
                next_index,
                ctoken,
                contribution_strategy,
                xcm_fees_payment_strategy,
            );

            Vaults::<T>::insert(crowdloan, next_index, new_vault);
            CTokensRegistry::<T>::insert(ctoken, (crowdloan, next_index));
            BatchIndexes::<T>::insert(crowdloan, next_index);

            Self::deposit_event(Event::<T>::VaultCreated(crowdloan, ctoken));

            Ok(())
        }

        /// Mark the associated vault as ready for real contributions on the relaychain
        #[pallet::weight(<T as Config>::WeightInfo::open())]
        #[transactional]
        pub fn open(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            T::OpenCloseOrigin::ensure_origin(origin)?;

            Self::try_mutate_vault(crowdloan, VaultPhase::Pending, |vault| {
                let amount = vault.pending;
                Self::do_contribute(None, crowdloan, amount, vault.xcm_fees_payment_strategy)?;

                vault.contributed = vault
                    .contributed
                    .checked_add(amount)
                    .ok_or(ArithmeticError::Overflow)?;

                vault.pending = Zero::zero();
                vault.phase = VaultPhase::Contributing;

                Self::deposit_event(Event::<T>::VaultOpened(crowdloan, amount));

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
            referral_code: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let mut vault = Self::current_vault(crowdloan).ok_or(Error::<T>::VaultDoesNotExist)?;

            ensure!(
                vault.phase == VaultPhase::Contributing || vault.phase == VaultPhase::Pending,
                Error::<T>::IncorrectVaultPhase
            );

            ensure!(!Self::is_vrf(), Error::<T>::VrfDelayInProgress);

            T::Assets::transfer(
                T::RelayCurrency::get(),
                &who,
                &Self::account_id(),
                amount,
                true,
            )
            .map_err(|_: DispatchError| Error::<T>::InsufficientBalance)?;

            match vault.phase {
                VaultPhase::Contributing => {
                    Self::do_contribute(
                        Some(&who),
                        crowdloan,
                        amount,
                        vault.xcm_fees_payment_strategy,
                    )?;

                    vault.contributed = vault
                        .contributed
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Overflow)?;
                }
                VaultPhase::Pending => {
                    vault.pending = vault
                        .pending
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Overflow)?;
                }
                _ => unreachable!(),
            }

            T::Assets::mint_into(vault.ctoken, &who, amount)?;

            Vaults::<T>::insert(crowdloan, vault.id, vault);

            Self::deposit_event(Event::<T>::VaultContributed(
                crowdloan,
                who,
                amount,
                referral_code,
            ));

            Ok(().into())
        }

        /// Mark the start/end of vrf delay, no contribution is allowed if
        /// the vrf delay is in progress
        #[pallet::weight(<T as Config>::WeightInfo::toggle_vrf_delay())]
        #[transactional]
        pub fn toggle_vrf_delay(origin: OriginFor<T>) -> DispatchResult {
            T::VrfDelayOrigin::ensure_origin(origin)?;
            let is_vrf = Self::is_vrf();

            IsVrfDelayInProgress::<T>::mutate(|b| *b = !is_vrf);

            Self::deposit_event(Event::<T>::VrfDelayToggled(!is_vrf));

            Ok(())
        }

        /// Mark the associated vault as `Closed` and stop accepting contributions
        #[pallet::weight(<T as Config>::WeightInfo::close())]
        #[transactional]
        pub fn close(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            T::OpenCloseOrigin::ensure_origin(origin)?;

            Self::try_mutate_vault(crowdloan, VaultPhase::Contributing, |vault| {
                vault.phase = VaultPhase::Closed;
                Self::deposit_event(Event::<T>::VaultClosed(crowdloan));
                Ok(())
            })
        }

        /// Mark the associated vault as `Contributing` and continue to accept contributions
        #[pallet::weight(<T as Config>::WeightInfo::reopen())]
        #[transactional]
        pub fn reopen(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            T::OpenCloseOrigin::ensure_origin(origin)?;

            Self::try_mutate_vault(crowdloan, VaultPhase::Closed, |vault| {
                vault.phase = VaultPhase::Contributing;
                Self::deposit_event(Event::<T>::VaultReOpened(crowdloan));
                Ok(())
            })
        }

        /// If a `crowdloan` failed, get the coins back and mark the vault as ready
        /// for distribution
        #[pallet::weight(<T as Config>::WeightInfo::auction_failed())]
        #[transactional]
        pub fn auction_failed(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            T::AuctionFailedOrigin::ensure_origin(origin)?;

            Self::try_mutate_vault(crowdloan, VaultPhase::Closed, |vault| {
                Self::do_withdraw(
                    crowdloan,
                    vault.contributed,
                    vault.xcm_fees_payment_strategy,
                )?;
                vault.phase = VaultPhase::Failed;
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
            ctoken: AssetIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let vault = Self::ctokens_registry(ctoken)
                .and_then(|(crowdloan, index)| Self::vaults(crowdloan, index))
                .ok_or(Error::<T>::VaultDoesNotExist)?;

            ensure!(
                vault.phase == VaultPhase::Failed || vault.phase == VaultPhase::Expired,
                Error::<T>::IncorrectVaultPhase
            );

            let ctoken_amount = <T as Config>::Assets::reducible_balance(vault.ctoken, &who, false);

            ensure!(ctoken_amount >= amount, Error::<T>::InsufficientBalance);

            T::Assets::burn_from(vault.ctoken, &who, amount)?;

            T::Assets::transfer(
                T::RelayCurrency::get(),
                &Self::account_id(),
                &who,
                amount,
                false,
            )?;

            Self::deposit_event(Event::<T>::VaultClaimRefund(ctoken, who, amount));

            Ok(())
        }

        /// If a `crowdloan` succeeded and its slot expired, use `call` to
        /// claim back the funds lent to the parachain
        #[pallet::weight(<T as Config>::WeightInfo::slot_expired())]
        #[transactional]
        pub fn slot_expired(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            T::SlotExpiredOrigin::ensure_origin(origin)?;

            Self::try_mutate_vault(crowdloan, VaultPhase::Closed, |vault| {
                Self::do_withdraw(
                    crowdloan,
                    vault.contributed,
                    vault.xcm_fees_payment_strategy,
                )?;
                vault.phase = VaultPhase::Expired;
                Self::deposit_event(Event::<T>::VaultSlotExpired(crowdloan));
                Ok(())
            })
        }

        /// Add more reserves so that can be used for xcm fees
        #[pallet::weight(<T as Config>::WeightInfo::add_reserves())]
        #[transactional]
        pub fn add_reserves(
            origin: OriginFor<T>,
            payer: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::ReserveOrigin::ensure_origin(origin)?;
            let payer = T::Lookup::lookup(payer)?;

            T::XCM::add_reserves(
                T::RelayCurrency::get(),
                payer.clone(),
                amount,
                Self::account_id(),
            )?;

            Self::deposit_event(Event::<T>::ReservesAdded(payer, amount));
            Ok(().into())
        }

        /// Update reserve_factor for charging less/more fees
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

        /// Update xm fees amount to be used in xcm.Withdraw message
        #[pallet::weight(<T as Config>::WeightInfo::update_xcm_fees())]
        #[transactional]
        pub fn update_xcm_fees(
            origin: OriginFor<T>,
            #[pallet::compact] fees: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            T::XCM::update_xcm_fees(fees);
            Self::deposit_event(Event::<T>::XcmFeesUpdated(fees));
            Ok(().into())
        }

        /// Update xcm weight to be used in xcm.Transact message
        #[pallet::weight(<T as Config>::WeightInfo::update_xcm_weight())]
        #[transactional]
        pub fn update_xcm_weight(
            origin: OriginFor<T>,
            xcm_weight_misc: XcmWeightMisc<Weight>,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            T::XCM::update_xcm_weight(xcm_weight_misc);
            Self::deposit_event(Event::<T>::XcmWeightUpdated(xcm_weight_misc));
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Crowdloans pool account
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        /// Parachain's sovereign account on relaychain
        pub fn para_account_id() -> T::AccountId {
            T::SelfParaId::get().into_account()
        }

        /// Xcm fees payer account on parachain
        pub fn xcm_fees_payer() -> T::AccountId {
            T::XcmFeesPayer::get().into_account()
        }

        fn next_index(crowdloan: ParaId) -> u32 {
            Self::current_index(crowdloan)
                .and_then(|idx| idx.checked_add(1u32))
                .unwrap_or(0)
        }

        fn current_vault(crowdloan: ParaId) -> Option<Vault<T>> {
            Self::current_index(crowdloan).and_then(|index| Self::vaults(crowdloan, index))
        }

        #[require_transactional]
        fn try_mutate_vault<F>(crowdloan: ParaId, phase: VaultPhase, cb: F) -> DispatchResult
        where
            F: FnOnce(&mut Vault<T>) -> DispatchResult,
        {
            let index = Self::current_index(crowdloan).ok_or(Error::<T>::VaultDoesNotExist)?;
            Vaults::<T>::try_mutate(crowdloan, index, |vault| {
                let vault = vault.as_mut().ok_or(Error::<T>::VaultDoesNotExist)?;
                ensure!(vault.phase == phase, Error::<T>::IncorrectVaultPhase);
                cb(vault)
            })
        }

        #[require_transactional]
        fn do_contribute(
            who: Option<&AccountIdOf<T>>,
            crowdloan: ParaId,
            amount: BalanceOf<T>,
            xcm_fees_payment_strategy: XcmFeesPaymentStrategy,
        ) -> Result<(), DispatchError> {
            ensure!(
                amount >= T::MinContribution::get(),
                Error::<T>::InsufficientBalance
            );

            T::Assets::burn_from(T::RelayCurrency::get(), &Self::account_id(), amount)?;

            T::XCM::do_contribute(
                crowdloan,
                T::AccountIdToMultiLocation::convert(T::RefundLocation::get()),
                T::RelayCurrency::get(),
                Self::account_id(),
                amount,
                Self::xcm_fees_payer(),
                xcm_fees_payment_strategy,
                who,
            )?;

            Ok(())
        }

        #[require_transactional]
        fn do_withdraw(
            para_id: ParaId,
            amount: BalanceOf<T>,
            xcm_fees_payment_strategy: XcmFeesPaymentStrategy,
        ) -> Result<(), DispatchError> {
            T::XCM::do_withdraw(
                para_id,
                T::AccountIdToMultiLocation::convert(T::RefundLocation::get()),
                T::RelayCurrency::get(),
                Self::account_id(),
                Self::para_account_id(),
                Self::xcm_fees_payer(),
                xcm_fees_payment_strategy,
            )?;

            T::Assets::mint_into(T::RelayCurrency::get(), &Self::account_id(), amount)?;

            Ok(())
        }
    }
}
