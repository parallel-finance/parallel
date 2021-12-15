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

pub mod types;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use crate::{types::*, weights::WeightInfo};

    use frame_support::{
        dispatch::DispatchResult,
        log,
        pallet_prelude::*,
        require_transactional,
        storage::{child, ChildTriePrefixIterator},
        traits::{
            fungibles::{Inspect, Mutate, Transfer},
            Get,
        },
        transactional, Blake2_128Concat, BoundedVec, PalletId,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use pallet_xcm::ensure_response;
    use primitives::{ump::*, Balance, CurrencyId, ParaId, TrieIndex};
    use sp_runtime::{
        traits::{AccountIdConversion, Convert, Hash, Zero},
        ArithmeticError, DispatchError,
    };
    use sp_std::{boxed::Box, convert::TryInto, vec::Vec};
    use xcm::latest::prelude::*;

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

        /// Assets for deposit/withdraw assets to/from crowdloan account
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        type Origin: IsType<<Self as frame_system::Config>::Origin>
            + Into<Result<pallet_xcm::Origin, <Self as Config>::Origin>>;

        type Call: IsType<<Self as pallet_xcm::Config>::Call> + From<Call<Self>>;

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

        /// Maximum number of vrf crowdloans
        #[pallet::constant]
        type MaxVrfs: Get<u32>;

        /// Maximum keys to be migrated in one extrinsic
        #[pallet::constant]
        type MigrateKeysLimit: Get<u32>;

        /// The origin which can update reserve_factor, xcm_fees etc
        type UpdateOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can migrate pending contribution
        type MigrateOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can set vrfs
        type VrfOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can create vault
        type CreateVaultOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can close/reopen vault
        type OpenCloseOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can call auction failed
        type AuctionFailedOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can call slot expired
        type SlotExpiredOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// Weight information
        type WeightInfo: WeightInfo;

        /// To expose XCM helper functions
        type XCM: XcmHelper<Self, BalanceOf<Self>, AssetIdOf<Self>, Self::AccountId>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New vault was created
        VaultCreated(ParaId, AssetIdOf<T>),
        /// User contributed amount to vault
        VaultContributed(ParaId, T::AccountId, BalanceOf<T>, Vec<u8>),
        /// Vault was opened
        VaultOpened(ParaId),
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
        /// Vrfs updated
        VrfsUpdated(BoundedVec<ParaId, T::MaxVrfs>),
        /// Notification received
        NotificationReceived(Box<MultiLocation>, QueryId, Option<(u32, XcmError)>),
        /// All migrated
        AllMigrated(ParaId),
        /// Partially migrated
        PartiallyMigrated(ParaId),
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
        /// Exceeded maximum vrfs
        ExceededMaxVrfs,
        /// Pending contribution must be killed before entering `Contributing` vault phase
        PendingContributionNotKilled,
    }

    #[pallet::storage]
    #[pallet::getter(fn vaults)]
    pub type Vaults<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, ParaId, Blake2_128Concat, u32, Vault<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn vrfs)]
    pub type Vrfs<T: Config> =
        StorageValue<_, BoundedVec<ParaId, <T as Config>::MaxVrfs>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn ctokens_registry)]
    pub type CTokensRegistry<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetIdOf<T>, (ParaId, u32), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_index)]
    pub type BatchIndexes<T: Config> = StorageMap<_, Blake2_128Concat, ParaId, u32, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_trie_index)]
    pub type NextTrieIndex<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn xcm_inflight)]
    pub type XcmInflight<T> =
        StorageMap<_, Blake2_128Concat, QueryId, XcmInflightRequest<T>, OptionQuery>;

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

            let trie_index = Self::next_trie_index();
            let next_trie_index = trie_index.checked_add(1).ok_or(ArithmeticError::Overflow)?;
            let new_vault = Vault::new(next_index, ctoken, contribution_strategy, trie_index);

            log::trace!(
                target: "crowdloans::create_vault",
                "ctoken_issuance: {:?}, next_index: {:?}, trie_index: {:?}, ctoken: {:?}",
                ctoken_issuance,
                next_index,
                trie_index,
                ctoken,
            );

            NextTrieIndex::<T>::put(next_trie_index);

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

            log::trace!(
                target: "crowdloans::open",
                "pre-toggle. crowdloan: {:?}",
                crowdloan,
            );

            Self::try_mutate_vault(crowdloan, VaultPhase::Pending, |vault| {
                ensure!(
                    Self::contribution_iterator(vault.trie_index, true)
                        .count()
                        .is_zero()
                        && vault.pending.is_zero(),
                    Error::<T>::PendingContributionNotKilled
                );
                vault.phase = VaultPhase::Contributing;
                Self::deposit_event(Event::<T>::VaultOpened(crowdloan));
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

            ensure!(
                amount >= T::MinContribution::get(),
                Error::<T>::InsufficientContribution
            );

            ensure!(
                !Self::vrfs().iter().any(|&c| c == crowdloan),
                Error::<T>::VrfDelayInProgress
            );

            T::Assets::transfer(
                T::RelayCurrency::get(),
                &who,
                &Self::vault_account_id(crowdloan),
                amount,
                true,
            )?;

            log::trace!(
                target: "crowdloans::contribute",
                "who: {:?}, crowdloan: {:?}, amount: {:?}",
                &who,
                &crowdloan,
                &amount,
            );

            if vault.phase == VaultPhase::Contributing && !Self::has_vrfs() {
                Self::do_contribute(&who, &mut vault, crowdloan, amount)?;
            }

            Self::do_pending_contribution(&who, &mut vault, amount)?;

            Vaults::<T>::insert(crowdloan, vault.id, vault);

            Self::deposit_event(Event::<T>::VaultContributed(
                crowdloan,
                who,
                amount,
                referral_code,
            ));

            Ok(().into())
        }

        /// Set crowdloans which entered vrf period
        #[pallet::weight(<T as Config>::WeightInfo::set_vrfs())]
        #[transactional]
        pub fn set_vrfs(origin: OriginFor<T>, vrfs: Vec<ParaId>) -> DispatchResult {
            T::VrfOrigin::ensure_origin(origin)?;

            log::trace!(
                target: "crowdloans::set_vrfs",
                "pre-toggle. vrfs: {:?}",
                vrfs
            );

            Vrfs::<T>::try_mutate(|b| -> Result<(), DispatchError> {
                *b = vrfs.try_into().map_err(|_| Error::<T>::ExceededMaxVrfs)?;
                Ok(())
            })?;

            Self::deposit_event(Event::<T>::VrfsUpdated(Self::vrfs()));

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

            let (crowdloan, index) =
                Self::ctokens_registry(ctoken).ok_or(Error::<T>::VaultDoesNotExist)?;
            let vault = Self::vaults(crowdloan, index).ok_or(Error::<T>::VaultDoesNotExist)?;

            ensure!(
                vault.phase == VaultPhase::Failed || vault.phase == VaultPhase::Expired,
                Error::<T>::IncorrectVaultPhase
            );

            let ctoken_amount = T::Assets::reducible_balance(vault.ctoken, &who, false);
            ensure!(ctoken_amount >= amount, Error::<T>::InsufficientBalance);

            log::trace!(
                target: "crowdloans::claim_refund",
                "pre-toggle. ctoken: {:?}",
                ctoken,
            );

            T::Assets::burn_from(vault.ctoken, &who, amount)?;

            T::Assets::transfer(
                T::RelayCurrency::get(),
                &Self::vault_account_id(crowdloan),
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

        /// migrate pending contribution by sending xcm
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn migrate_pending(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            T::MigrateOrigin::ensure_origin(origin)?;

            let mut vault = Self::current_vault(crowdloan).ok_or(Error::<T>::VaultDoesNotExist)?;
            let contributions = Self::contribution_iterator(vault.trie_index, true);
            let mut migrated_count = 0u32;
            let mut all_migrated = true;
            for (who, (amount, _)) in contributions {
                if migrated_count >= T::MigrateKeysLimit::get() {
                    all_migrated = false;
                    break;
                }
                Self::do_contribute(&who, &mut vault, crowdloan, amount)?;
                migrated_count += 1;
            }

            if all_migrated {
                Self::deposit_event(Event::<T>::AllMigrated(crowdloan));
            } else {
                Self::deposit_event(Event::<T>::PartiallyMigrated(crowdloan));
            }

            Ok(())
        }

        /// Update xcm fees amount to be used in xcm.Withdraw message
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

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn notification_received(
            origin: OriginFor<T>,
            query_id: QueryId,
            response: Response,
        ) -> DispatchResultWithPostInfo {
            let responder = ensure_response(<T as Config>::Origin::from(origin))?;
            if let Response::ExecutionResult(res) = response {
                match (Self::xcm_inflight(&query_id), res) {
                    (Some(request), None) => {
                        match request {
                            XcmInflightRequest::Contribute { index, who, amount } => {
                                let mut vault = Self::current_vault(index)
                                    .ok_or(Error::<T>::VaultDoesNotExist)?;
                                T::Assets::mint_into(vault.ctoken, &who, amount)?;
                                T::Assets::burn_from(
                                    T::RelayCurrency::get(),
                                    &Self::vault_account_id(index),
                                    amount,
                                )?;
                                Self::do_migrate_pending(&who, &mut vault, amount)?;
                                Vaults::<T>::insert(index, vault.id, vault);
                            }
                            XcmInflightRequest::Withdraw { index, amount } => {
                                T::Assets::mint_into(
                                    T::RelayCurrency::get(),
                                    &Self::vault_account_id(index),
                                    amount,
                                )?;
                            }
                        }

                        XcmInflight::<T>::remove(&query_id);
                    }
                    (Some(request), Some(_)) => match request {
                        XcmInflightRequest::Contribute { index, who, amount } => {
                            T::Assets::transfer(
                                T::RelayCurrency::get(),
                                &Self::vault_account_id(index),
                                &who,
                                amount,
                                true,
                            )?;
                        }
                        XcmInflightRequest::Withdraw {
                            index: _,
                            amount: _,
                        } => {}
                    },
                    _ => {}
                }

                Self::deposit_event(Event::<T>::NotificationReceived(
                    Box::new(responder),
                    query_id,
                    res,
                ));
            }
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Crowdloans vault account
        pub fn vault_account_id(crowdloan: ParaId) -> T::AccountId {
            T::PalletId::get().into_sub_account(crowdloan)
        }

        /// Parachain's sovereign account on relaychain
        pub fn para_account_id() -> T::AccountId {
            T::SelfParaId::get().into_account()
        }

        fn has_vrfs() -> bool {
            Self::vrfs().iter().len() != 0
        }

        fn next_index(crowdloan: ParaId) -> u32 {
            Self::current_index(crowdloan)
                .and_then(|idx| idx.checked_add(1u32))
                .unwrap_or(0)
        }

        fn current_vault(crowdloan: ParaId) -> Option<Vault<T>> {
            Self::current_index(crowdloan).and_then(|index| Self::vaults(crowdloan, index))
        }

        fn notify_placeholder() -> <T as Config>::Call {
            <T as Config>::Call::from(Call::<T>::notification_received {
                query_id: Default::default(),
                response: Default::default(),
            })
        }

        #[require_transactional]
        fn do_pending_contribution(
            who: &AccountIdOf<T>,
            vault: &mut Vault<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            vault.pending = vault
                .pending
                .checked_add(amount)
                .ok_or(ArithmeticError::Overflow)?;
            let (pending, _) = Self::contribution_get(vault.trie_index, who, true);
            let new_pending = pending
                .checked_add(amount)
                .ok_or(ArithmeticError::Overflow)?;
            Self::contribution_put(vault.trie_index, who, &new_pending, true);
            Ok(())
        }

        fn do_migrate_pending(
            who: &AccountIdOf<T>,
            vault: &mut Vault<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            vault.pending = vault
                .pending
                .checked_sub(amount)
                .ok_or(ArithmeticError::Underflow)?;
            vault.contributed = vault
                .contributed
                .checked_add(amount)
                .ok_or(ArithmeticError::Overflow)?;

            let (pending, _) = Self::contribution_get(vault.trie_index, who, true);
            let new_pending = pending
                .checked_sub(amount)
                .ok_or(ArithmeticError::Underflow)?;
            if new_pending.is_zero() {
                Self::contribution_kill(vault.trie_index, who, true);
            } else {
                Self::contribution_put(vault.trie_index, who, &new_pending, true);
            }

            let (contributed, _) = Self::contribution_get(vault.trie_index, who, false);
            let new_contributed = contributed
                .checked_add(amount)
                .ok_or(ArithmeticError::Overflow)?;
            Self::contribution_put(vault.trie_index, who, &new_contributed, false);

            Ok(())
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

        fn id_from_index(index: TrieIndex, pending: bool) -> child::ChildInfo {
            let mut buf = Vec::new();
            buf.extend_from_slice({
                if pending {
                    b"crowdloan:pending"
                } else {
                    b"crowdloan"
                }
            });
            buf.extend_from_slice(&index.encode()[..]);
            child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
        }

        fn contribution_put(
            index: TrieIndex,
            who: &T::AccountId,
            balance: &BalanceOf<T>,
            pending: bool,
        ) {
            who.using_encoded(|b| {
                child::put(
                    &Self::id_from_index(index, pending),
                    b,
                    &(balance, &Vec::<u8>::new()),
                )
            });
        }

        fn contribution_get(
            index: TrieIndex,
            who: &T::AccountId,
            pending: bool,
        ) -> (BalanceOf<T>, Vec<u8>) {
            who.using_encoded(|b| {
                child::get_or_default::<(BalanceOf<T>, Vec<u8>)>(
                    &Self::id_from_index(index, pending),
                    b,
                )
            })
        }

        fn contribution_kill(index: TrieIndex, who: &T::AccountId, pending: bool) {
            who.using_encoded(|b| child::kill(&Self::id_from_index(index, pending), b));
        }

        fn contribution_iterator(
            index: TrieIndex,
            pending: bool,
        ) -> ChildTriePrefixIterator<(T::AccountId, (BalanceOf<T>, Vec<u8>))> {
            ChildTriePrefixIterator::<_>::with_prefix_over_key::<Identity>(
                &Self::id_from_index(index, pending),
                &[],
            )
        }

        #[require_transactional]
        fn do_contribute(
            who: &AccountIdOf<T>,
            _vault: &mut Vault<T>,
            crowdloan: ParaId,
            amount: BalanceOf<T>,
        ) -> Result<(), DispatchError> {
            let query_id = T::XCM::do_contribute(
                crowdloan,
                T::AccountIdToMultiLocation::convert(T::RefundLocation::get()),
                T::RelayCurrency::get(),
                amount,
                who,
                Self::notify_placeholder(),
            )?;

            XcmInflight::<T>::insert(
                query_id,
                XcmInflightRequest::Contribute {
                    index: crowdloan,
                    who: who.clone(),
                    amount,
                },
            );

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

            let query_id = T::XCM::do_withdraw(
                para_id,
                T::AccountIdToMultiLocation::convert(T::RefundLocation::get()),
                T::RelayCurrency::get(),
                Self::para_account_id(),
                Self::notify_placeholder(),
            )?;

            XcmInflight::<T>::insert(
                query_id,
                XcmInflightRequest::Withdraw {
                    index: para_id,
                    amount,
                },
            );

            Ok(())
        }
    }
}
