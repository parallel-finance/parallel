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
        log,
        pallet_prelude::*,
        require_transactional,
        traits::{
            fungibles::{Inspect, Mutate, Transfer},
            Get,
        },
        transactional, Blake2_128Concat, PalletId,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use primitives::{ump::*, Balance, BlockNumber, CurrencyId, ParaId};
    use sp_runtime::{
        traits::{AccountIdConversion, BlockNumberProvider, Convert, Zero},
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

        /// Minimum contribute amount
        #[pallet::constant]
        type MinContribution: Get<BalanceOf<Self>>;

        /// The origin which can update reserve_factor, xcm_fees etc
        type UpdateOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can toggle vrf delay
        type VrfDelayOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can create vault
        type CreateVaultOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can update vault
        type UpdateVaultOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can close/reopen vault
        type OpenCloseOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can call auction failed
        type AuctionFailedOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can call slot expired
        type SlotExpiredOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// Weight information
        type WeightInfo: WeightInfo;

        /// The relay's BlockNumber provider
        type BlockNumberProvider: BlockNumberProvider<BlockNumber = primitives::BlockNumber>;

        /// To expose XCM helper functions
        type XCM: XcmHelper<BalanceOf<Self>, AssetIdOf<Self>, Self::AccountId>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New vault was created
        VaultCreated(ParaId, AssetIdOf<T>),
        /// Existing vault was updated
        VaultUpdated(ParaId),
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
        /// Xcm weight in BuyExecution message
        XcmWeightUpdated(XcmWeightMisc<Weight>),
        /// Fees for extrinsics on relaychain were set to new value
        XcmFeesUpdated(BalanceOf<T>),
        /// Vrf delay toggled
        VrfDelayToggled(bool),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Vault is not in correct phase
        IncorrectVaultPhase,
        /// Crowdloan ParaId aready exists
        CrowdloanAlreadyExists,
        /// Contribution is not enough
        InsufficientContribution,
        /// Balance is not enough
        InsufficientBalance,
        /// Vault does not exist
        VaultDoesNotExist,
        /// Ctoken already taken by another vault
        CTokenAlreadyTaken,
        /// ParaId already taken by another vault
        ParaIdAlreadyTaken,
        /// No contributions allowed during the VRF delay
        VrfDelayInProgress,
        /// Attempted contribution violates contribution cap
        ExceededCap,
        /// Current relay block is greater then vault end block
        ExceededCrowdloanEndBlock,
    }

    #[pallet::storage]
    #[pallet::getter(fn vaults)]
    pub type Vaults<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, ParaId, Blake2_128Concat, u32, Vault<T>, OptionQuery>;

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
            cap: BalanceOf<T>,
            end_block: BlockNumber,
        ) -> DispatchResult {
            T::CreateVaultOrigin::ensure_origin(origin)?;

            let ctoken_issuance = T::Assets::total_issuance(ctoken);
            ensure!(
                ctoken_issuance.is_zero() && !CTokensRegistry::<T>::contains_key(ctoken),
                Error::<T>::CTokenAlreadyTaken
            );

            let next_index = Self::next_index(crowdloan);
            ensure!(
                !Vaults::<T>::contains_key(crowdloan, next_index),
                Error::<T>::CrowdloanAlreadyExists
            );

            let new_vault = Vault::new(next_index, ctoken, contribution_strategy, cap, end_block);

            ensure!(
                T::BlockNumberProvider::current_block_number() <= new_vault.end_block,
                Error::<T>::ExceededCrowdloanEndBlock
            );

            log::trace!(
                target: "crowdloans::create_vault",
                "ctoken_issuance: {:?}, next_index: {:?}, ctoken: {:?}",
                ctoken_issuance,
                next_index,
                ctoken,
            );

            Vaults::<T>::insert(crowdloan, next_index, new_vault);
            CTokensRegistry::<T>::insert(ctoken, (crowdloan, next_index));
            BatchIndexes::<T>::insert(crowdloan, next_index);

            Self::deposit_event(Event::<T>::VaultCreated(crowdloan, ctoken));

            Ok(())
        }

        /// Update an exisiting vault via a governance decision
        #[pallet::weight(<T as Config>::WeightInfo::update_vault())]
        #[transactional]
        pub fn update_vault(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            cap: Option<BalanceOf<T>>,
            end_block: Option<BlockNumber>,
            contribution_strategy: Option<ContributionStrategy>,
        ) -> DispatchResult {
            T::UpdateVaultOrigin::ensure_origin(origin)?;

            let mut vault = Self::current_vault(crowdloan).ok_or(Error::<T>::VaultDoesNotExist)?;

            if let Some(cap) = cap {
                vault.cap = cap;
            }

            if let Some(end_block) = end_block {
                ensure!(
                    T::BlockNumberProvider::current_block_number() <= end_block,
                    Error::<T>::ExceededCrowdloanEndBlock
                );

                vault.end_block = end_block;
            }

            if let Some(contribution_strategy) = contribution_strategy {
                vault.contribution_strategy = contribution_strategy;
            }

            Self::deposit_event(Event::<T>::VaultUpdated(crowdloan));

            Ok(())
        }

        /// Mark the associated vault as ready for real contributions on the relaychain
        #[pallet::weight(<T as Config>::WeightInfo::open())]
        #[transactional]
        pub fn open(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            T::OpenCloseOrigin::ensure_origin(origin)?;

            Self::try_mutate_vault(crowdloan, VaultPhase::Pending, |vault| {
                let amount = vault.pending;
                if amount >= T::MinContribution::get() {
                    Self::do_contribute(None, crowdloan, amount)?;

                    vault.contributed = vault
                        .contributed
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Overflow)?;
                }

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
                T::BlockNumberProvider::current_block_number() <= vault.end_block,
                Error::<T>::ExceededCrowdloanEndBlock
            );

            ensure!(
                vault.phase == VaultPhase::Contributing || vault.phase == VaultPhase::Pending,
                Error::<T>::IncorrectVaultPhase
            );

            ensure!(!Self::is_vrf(), Error::<T>::VrfDelayInProgress);

            ensure!(
                amount >= T::MinContribution::get(),
                Error::<T>::InsufficientContribution
            );

            T::Assets::transfer(
                T::RelayCurrency::get(),
                &who,
                &Self::account_id(),
                amount,
                true,
            )?;

            let total_amount = Self::cap(&vault, amount)?;

            // throw if new value overflows cap
            ensure!(total_amount < vault.cap, Error::<T>::ExceededCap);

            match vault.phase {
                VaultPhase::Contributing => {
                    Self::do_contribute(Some(&who), crowdloan, amount)?;

                    vault.contributed = vault
                        .contributed
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Overflow)?;
                }
                VaultPhase::Pending => {
                    log::trace!(
                        target: "crowdloans::contribute",
                        "Contibute pending. crowdloan: {:?}, amount: {:?}",
                        crowdloan,
                        amount,
                    );

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

            log::trace!(
                target: "crowdloans::toggle_vrf_delay",
                "pre-toggle. is_vrf: {:?}",
                is_vrf,
            );

            IsVrfDelayInProgress::<T>::mutate(|b| *b = !is_vrf);

            Self::deposit_event(Event::<T>::VrfDelayToggled(!is_vrf));

            Ok(())
        }

        /// Mark the associated vault as `Closed` and stop accepting contributions
        #[pallet::weight(<T as Config>::WeightInfo::close())]
        #[transactional]
        pub fn close(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            T::OpenCloseOrigin::ensure_origin(origin)?;

            log::trace!(
                target: "crowdloans::close",
                "pre-toggle. crowdloan: {:?}",
                crowdloan,
            );

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

            log::trace!(
                target: "crowdloans::reopen",
                "pre-toggle. crowdloan: {:?}",
                crowdloan,
            );

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

            log::trace!(
                target: "crowdloans::auction_failed",
                "pre-toggle. crowdloan: {:?}",
                crowdloan,
            );

            Self::try_mutate_vault(crowdloan, VaultPhase::Closed, |vault| {
                Self::do_withdraw(crowdloan, vault.contributed)?;
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

            log::trace!(
                target: "crowdloans::claim_refund",
                "pre-toggle. ctoken: {:?}",
                ctoken,
            );

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
                Self::do_withdraw(crowdloan, vault.contributed)?;
                vault.phase = VaultPhase::Expired;
                Self::deposit_event(Event::<T>::VaultSlotExpired(crowdloan));
                Ok(())
            })
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

        fn next_index(crowdloan: ParaId) -> u32 {
            Self::current_index(crowdloan)
                .and_then(|idx| idx.checked_add(1u32))
                .unwrap_or(0)
        }

        fn current_vault(crowdloan: ParaId) -> Option<Vault<T>> {
            Self::current_index(crowdloan).and_then(|index| Self::vaults(crowdloan, index))
        }

        fn cap(vault: &Vault<T>, amount: BalanceOf<T>) -> Result<BalanceOf<T>, ArithmeticError> {
            vault
                .contributed
                .checked_add(vault.pending)
                .and_then(|sum| sum.checked_add(amount))
                .ok_or(ArithmeticError::Overflow)
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
        ) -> Result<(), DispatchError> {
            T::Assets::burn_from(T::RelayCurrency::get(), &Self::account_id(), amount)?;

            log::trace!(
                target: "crowdloans::do_contribute",
                "who: {:?}, crowdloan: {:?}, amount: {:?}",
                &who,
                &crowdloan,
                &amount,
            );

            T::XCM::do_contribute(
                crowdloan,
                T::AccountIdToMultiLocation::convert(T::RefundLocation::get()),
                T::RelayCurrency::get(),
                amount,
                who,
            )?;

            Ok(())
        }

        #[require_transactional]
        fn do_withdraw(para_id: ParaId, amount: BalanceOf<T>) -> Result<(), DispatchError> {
            log::trace!(
                target: "crowdloans::do_withdraw",
                "para_id: {:?}, amount: {:?}",
                &para_id,
                &amount,
            );

            T::XCM::do_withdraw(
                para_id,
                T::AccountIdToMultiLocation::convert(T::RefundLocation::get()),
                T::RelayCurrency::get(),
                Self::para_account_id(),
            )?;

            T::Assets::mint_into(T::RelayCurrency::get(), &Self::account_id(), amount)?;

            Ok(())
        }
    }
}
