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
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use pallet_xcm::ensure_response;
    use primitives::{Balance, CurrencyId, ParaId, TrieIndex};
    use sp_runtime::{
        traits::{AccountIdConversion, BlockNumberProvider, Convert, Hash, Zero},
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

        /// The origin which can migrate pending contribution
        type MigrateOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can set vrfs
        type VrfOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

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
        type RelayChainBlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;

        /// To expose XCM helper functions
        type XCM: XcmHelper<Self, BalanceOf<Self>, AssetIdOf<Self>, Self::AccountId>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New vault was created
        /// [para_id, ctoken_id]
        VaultCreated(ParaId, AssetIdOf<T>),
        /// Existing vault was updated
        /// [para_id, vault_id, cap, end_block, contribution_strategy]
        VaultUpdated(
            ParaId,
            u32,
            BalanceOf<T>,
            BlockNumberFor<T>,
            ContributionStrategy,
        ),
        /// Vault was opened
        /// [para_id]
        VaultOpened(ParaId),
        /// Vault was closed
        /// [para_id]
        VaultClosed(ParaId),
        /// Vault was reopened
        /// [para_id]
        VaultReOpened(ParaId),
        /// Vault is successful
        /// [para_id]
        VaultSucceeded(ParaId),
        /// Vault is failing
        /// [para_id]
        VaultFailed(ParaId),
        /// Vault is expiring
        /// [para_id]
        VaultExpired(ParaId),
        /// Vault is trying to do contributing
        /// [para_id, contributor, amount, referral_code]
        VaultDoContributing(ParaId, T::AccountId, BalanceOf<T>, Vec<u8>),
        /// Vault is trying to do withdrawing
        /// [para_id, amount, target_phase]
        VaultDoWithdrawing(ParaId, BalanceOf<T>, VaultPhase),
        /// Vault successfully contributed
        /// [para_id, contributor, amount]
        VaultContributed(ParaId, T::AccountId, BalanceOf<T>, Vec<u8>),
        /// A user claimed refund from vault
        /// [ctoken_id, account, amount]
        VaultClaimRefund(AssetIdOf<T>, T::AccountId, BalanceOf<T>),
        /// Vrfs updated
        /// [vrf_data]
        VrfsUpdated(BoundedVec<ParaId, T::MaxVrfs>),
        /// Notification received
        /// [multi_location, query_id, res]
        NotificationReceived(Box<MultiLocation>, QueryId, Option<(u32, XcmError)>),
        /// All contributions migrated
        /// [para_id]
        AllMigrated(ParaId),
        /// Partially contributions migrated
        /// [para_id, non_migrated_count]
        PartiallyMigrated(ParaId, u32),
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
        /// Current relay block is greater than vault end block
        ExceededEndBlock,
        /// Exceeded maximum vrfs
        ExceededMaxVrfs,
        /// Pending contribution must be killed before entering `Contributing` vault phase
        PendingContributionNotKilled,
        /// Capacity cannot be zero value
        ZeroCap,
        /// Invalid params input
        InvalidParams,
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
    #[pallet::getter(fn xcm_request)]
    pub type XcmRequests<T> = StorageMap<_, Blake2_128Concat, QueryId, XcmRequest<T>, OptionQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new vault via a governance decision
        ///
        /// - `ctoken`: ctoken is used for the vault, should be unique
        /// - `crowdloan`: parachain id of the crowdloan, should be consistent with relaychain
        /// - `contribution_strategy`: currently, only XCM strategy is supported.
        /// - `cap`: the capacity limit for the vault
        /// - `end_block`: the crowdloan end block for the vault
        #[pallet::weight(<T as Config>::WeightInfo::create_vault())]
        #[transactional]
        pub fn create_vault(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            ctoken: AssetIdOf<T>,
            contribution_strategy: ContributionStrategy,
            #[pallet::compact] cap: BalanceOf<T>,
            end_block: BlockNumberFor<T>,
        ) -> DispatchResult {
            T::CreateVaultOrigin::ensure_origin(origin)?;

            ensure!(!cap.is_zero(), Error::<T>::ZeroCap);

            let ctoken_issuance = T::Assets::total_issuance(ctoken);
            ensure!(
                ctoken_issuance.is_zero() && !CTokensRegistry::<T>::contains_key(ctoken),
                Error::<T>::CTokenAlreadyTaken
            );

            // origin shouldn't be able to create a new vault if the previous one is not finished
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

            ensure!(
                T::RelayChainBlockNumberProvider::current_block_number() <= end_block,
                Error::<T>::ExceededEndBlock
            );

            let trie_index = Self::next_trie_index();
            let next_trie_index = trie_index.checked_add(1).ok_or(ArithmeticError::Overflow)?;
            let new_vault = Vault::new(
                next_index,
                ctoken,
                contribution_strategy,
                cap,
                end_block,
                trie_index,
            );

            log::trace!(
                target: "crowdloans::create_vault",
                "ctoken_issuance: {:?}, next_index: {:?}, trie_index: {:?}, ctoken: {:?}",
                ctoken_issuance,
                next_index,
                trie_index,
                ctoken
            );

            NextTrieIndex::<T>::put(next_trie_index);
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
            end_block: Option<BlockNumberFor<T>>,
            contribution_strategy: Option<ContributionStrategy>,
        ) -> DispatchResult {
            T::UpdateVaultOrigin::ensure_origin(origin)?;

            let mut vault = Self::current_vault(crowdloan).ok_or(Error::<T>::VaultDoesNotExist)?;

            if let Some(cap) = cap {
                ensure!(!cap.is_zero(), Error::<T>::ZeroCap);
                vault.cap = cap;
            }

            if let Some(end_block) = end_block {
                ensure!(
                    T::RelayChainBlockNumberProvider::current_block_number() <= end_block,
                    Error::<T>::ExceededEndBlock
                );

                vault.end_block = end_block;
            }

            if let Some(contribution_strategy) = contribution_strategy {
                vault.contribution_strategy = contribution_strategy;
            }

            Vaults::<T>::insert(crowdloan, vault.id, vault.clone());

            Self::deposit_event(Event::<T>::VaultUpdated(
                crowdloan,
                vault.id,
                vault.cap,
                vault.end_block,
                vault.contribution_strategy,
            ));

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
                    Self::contribution_iterator(vault.trie_index, ChildStorageKind::Pending)
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

            ensure!(!amount.is_zero(), Error::<T>::InvalidParams);

            ensure!(
                T::RelayChainBlockNumberProvider::current_block_number() <= vault.end_block,
                Error::<T>::ExceededEndBlock
            );

            ensure!(
                vault.phase == VaultPhase::Contributing || vault.phase == VaultPhase::Pending,
                Error::<T>::IncorrectVaultPhase
            );

            ensure!(
                amount >= T::MinContribution::get(),
                Error::<T>::InsufficientContribution
            );

            ensure!(!Self::in_vrf(crowdloan), Error::<T>::VrfDelayInProgress);

            ensure!(
                Self::total_contribution(&vault, amount)? <= vault.cap,
                Error::<T>::ExceededCap
            );

            T::Assets::transfer(
                T::RelayCurrency::get(),
                &who,
                &Self::vault_account_id(crowdloan),
                amount,
                true,
            )?;

            if vault.phase == VaultPhase::Contributing && !Self::has_vrfs() {
                Self::do_update_contribution(
                    &who,
                    &mut vault,
                    amount,
                    Some(referral_code.clone()),
                    ArithmeticKind::Addition,
                    ChildStorageKind::Flying,
                )?;
                Self::do_contribute(&who, crowdloan, amount, referral_code)?;
            } else {
                Self::do_update_contribution(
                    &who,
                    &mut vault,
                    amount,
                    Some(referral_code.clone()),
                    ArithmeticKind::Addition,
                    ChildStorageKind::Pending,
                )?;
            }

            Vaults::<T>::insert(crowdloan, vault.id, vault);

            log::trace!(
                target: "crowdloans::contribute",
                "who: {:?}, crowdloan: {:?}, amount: {:?}",
                &who,
                &crowdloan,
                &amount,
            );

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

        /// Mark the associated vault as `Succeed` if vault is `Closed`
        #[pallet::weight(<T as Config>::WeightInfo::auction_succeeded())]
        #[transactional]
        pub fn auction_succeeded(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            T::OpenCloseOrigin::ensure_origin(origin)?;

            log::trace!(
                target: "crowdloans::auction_succeeded",
                "pre-toggle. crowdloan: {:?}",
                crowdloan,
            );

            Self::try_mutate_vault(crowdloan, VaultPhase::Closed, |vault| {
                vault.phase = VaultPhase::Succeeded;
                Self::deposit_event(Event::<T>::VaultSucceeded(crowdloan));
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
                Self::do_withdraw(crowdloan, vault.contributed, VaultPhase::Failed)?;
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

            ensure!(!amount.is_zero(), Error::<T>::InvalidParams);

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

            log::trace!(
                target: "crowdloans::slot_expired",
                "pre-toggle. crowdloan: {:?}",
                crowdloan,
            );

            Self::try_mutate_vault(crowdloan, VaultPhase::Succeeded, |vault| {
                Self::do_withdraw(crowdloan, vault.contributed, VaultPhase::Expired)?;
                Ok(())
            })
        }

        /// Migrate pending contribution by sending xcm
        #[pallet::weight(<T as Config>::WeightInfo::migrate_pending())]
        #[transactional]
        pub fn migrate_pending(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            T::MigrateOrigin::ensure_origin(origin)?;

            let mut vault = Self::current_vault(crowdloan).ok_or(Error::<T>::VaultDoesNotExist)?;
            ensure!(
                vault.phase == VaultPhase::Pending || vault.phase == VaultPhase::Contributing,
                Error::<T>::IncorrectVaultPhase
            );
            let contributions =
                Self::contribution_iterator(vault.trie_index, ChildStorageKind::Pending);
            // TODO: remove 2nd read
            let count: u32 =
                Self::contribution_iterator(vault.trie_index, ChildStorageKind::Pending)
                    .count()
                    .try_into()
                    .map_err(|_| ArithmeticError::Overflow)?;
            let mut migrated_count = 0u32;
            let mut all_migrated = true;

            // single migration has a processing limit
            for (who, (amount, referral_code)) in contributions {
                if migrated_count >= T::MigrateKeysLimit::get() {
                    all_migrated = false;
                    break;
                }
                Self::do_migrate_contribution(
                    &who,
                    &mut vault,
                    amount,
                    ChildStorageKind::Pending,
                    ChildStorageKind::Flying,
                )?;
                Self::do_contribute(&who, crowdloan, amount, referral_code)?;
                migrated_count += 1;
            }

            if all_migrated {
                Self::deposit_event(Event::<T>::AllMigrated(crowdloan));
            } else {
                Self::deposit_event(Event::<T>::PartiallyMigrated(
                    crowdloan,
                    count - migrated_count,
                ));
            }

            Ok(())
        }

        #[pallet::weight(<T as Config>::WeightInfo::notification_received())]
        #[transactional]
        pub fn notification_received(
            origin: OriginFor<T>,
            query_id: QueryId,
            response: Response,
        ) -> DispatchResultWithPostInfo {
            let responder = ensure_response(<T as Config>::Origin::from(origin))?;
            if let Response::ExecutionResult(res) = response {
                if let Some(request) = Self::xcm_request(&query_id) {
                    Self::do_notification_received(query_id, request, res)?;
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

        fn in_vrf(crowdloan: ParaId) -> bool {
            Self::vrfs().iter().any(|&c| c == crowdloan)
        }

        fn next_index(crowdloan: ParaId) -> u32 {
            Self::current_index(crowdloan)
                .and_then(|idx| idx.checked_add(1u32))
                .unwrap_or(0)
        }

        pub(crate) fn current_vault(crowdloan: ParaId) -> Option<Vault<T>> {
            Self::current_index(crowdloan).and_then(|index| Self::vaults(crowdloan, index))
        }

        fn total_contribution(
            vault: &Vault<T>,
            amount: BalanceOf<T>,
        ) -> Result<BalanceOf<T>, ArithmeticError> {
            vault
                .contributed
                .checked_add(vault.flying)
                .and_then(|sum| sum.checked_add(vault.pending))
                .and_then(|sum| sum.checked_add(amount))
                .ok_or(ArithmeticError::Overflow)
        }

        fn notify_placeholder() -> <T as Config>::Call {
            <T as Config>::Call::from(Call::<T>::notification_received {
                query_id: Default::default(),
                response: Default::default(),
            })
        }

        /// Get and recalculate the user's contribution for the specified kind of child storage
        #[require_transactional]
        fn do_update_contribution(
            who: &AccountIdOf<T>,
            vault: &mut Vault<T>,
            amount: BalanceOf<T>,
            new_referral_code: Option<Vec<u8>>,
            arithmetic_kind: ArithmeticKind,
            child_storage_kind: ChildStorageKind,
        ) -> DispatchResult {
            use ArithmeticKind::*;
            use ChildStorageKind::*;

            let (contribution, old_referral_code) =
                Self::contribution_get(vault.trie_index, who, child_storage_kind);
            let referral_code = new_referral_code.unwrap_or(old_referral_code);
            let new_contribution = match (child_storage_kind, arithmetic_kind) {
                (Pending, Addition) => {
                    vault.pending = vault
                        .pending
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Overflow)?;
                    contribution
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Overflow)?
                }
                (Pending, Subtraction) => {
                    vault.pending = vault
                        .pending
                        .checked_sub(amount)
                        .ok_or(ArithmeticError::Underflow)?;
                    contribution
                        .checked_sub(amount)
                        .ok_or(ArithmeticError::Underflow)?
                }
                (Flying, Addition) => {
                    vault.flying = vault
                        .flying
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Overflow)?;
                    contribution
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Overflow)?
                }
                (Flying, Subtraction) => {
                    vault.flying = vault
                        .flying
                        .checked_sub(amount)
                        .ok_or(ArithmeticError::Underflow)?;
                    contribution
                        .checked_sub(amount)
                        .ok_or(ArithmeticError::Underflow)?
                }
                (Contributed, Addition) => {
                    vault.contributed = vault
                        .contributed
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Overflow)?;
                    contribution
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Overflow)?
                }
                (Contributed, Subtraction) => {
                    vault.contributed = vault
                        .contributed
                        .checked_sub(amount)
                        .ok_or(ArithmeticError::Underflow)?;
                    contribution
                        .checked_sub(amount)
                        .ok_or(ArithmeticError::Underflow)?
                }
            };
            if new_contribution.is_zero() {
                Self::contribution_kill(vault.trie_index, who, child_storage_kind);
            } else {
                Self::contribution_put(
                    vault.trie_index,
                    who,
                    &new_contribution,
                    &referral_code,
                    child_storage_kind,
                );
            }
            Ok(())
        }

        #[require_transactional]
        fn do_migrate_contribution(
            who: &AccountIdOf<T>,
            vault: &mut Vault<T>,
            amount: BalanceOf<T>,
            src_child_storage_kind: ChildStorageKind,
            dst_child_storage_kind: ChildStorageKind,
        ) -> DispatchResult {
            Self::do_update_contribution(
                who,
                vault,
                amount,
                None,
                ArithmeticKind::Subtraction,
                src_child_storage_kind,
            )?;

            Self::do_update_contribution(
                who,
                vault,
                amount,
                None,
                ArithmeticKind::Addition,
                dst_child_storage_kind,
            )?;

            Ok(())
        }

        #[require_transactional]
        fn do_notification_received(
            query_id: QueryId,
            request: XcmRequest<T>,
            res: Option<(u32, XcmError)>,
        ) -> DispatchResult {
            let executed = res.is_none();
            match request {
                XcmRequest::Contribute {
                    crowdloan,
                    who,
                    amount,
                    referral_code,
                } if executed => {
                    let mut vault =
                        Self::current_vault(crowdloan).ok_or(Error::<T>::VaultDoesNotExist)?;
                    T::Assets::mint_into(vault.ctoken, &who, amount)?;
                    T::Assets::burn_from(
                        T::RelayCurrency::get(),
                        &Self::vault_account_id(crowdloan),
                        amount,
                    )?;
                    Self::do_migrate_contribution(
                        &who,
                        &mut vault,
                        amount,
                        ChildStorageKind::Flying,
                        ChildStorageKind::Contributed,
                    )?;
                    Vaults::<T>::insert(crowdloan, vault.id, vault);

                    Self::deposit_event(Event::<T>::VaultContributed(
                        crowdloan,
                        who,
                        amount,
                        referral_code,
                    ));
                }
                XcmRequest::Contribute {
                    crowdloan,
                    who,
                    amount,
                    ..
                } if !executed => {
                    let mut vault =
                        Self::current_vault(crowdloan).ok_or(Error::<T>::VaultDoesNotExist)?;
                    T::Assets::transfer(
                        T::RelayCurrency::get(),
                        &Self::vault_account_id(crowdloan),
                        &who,
                        amount,
                        true,
                    )?;
                    Self::do_update_contribution(
                        &who,
                        &mut vault,
                        amount,
                        None,
                        ArithmeticKind::Subtraction,
                        ChildStorageKind::Flying,
                    )?;
                    Vaults::<T>::insert(crowdloan, vault.id, vault);
                }
                XcmRequest::Withdraw {
                    crowdloan,
                    amount,
                    target_phase,
                } if executed => {
                    let mut vault =
                        Self::current_vault(crowdloan).ok_or(Error::<T>::VaultDoesNotExist)?;
                    T::Assets::mint_into(
                        T::RelayCurrency::get(),
                        &Self::vault_account_id(crowdloan),
                        amount,
                    )?;
                    vault.phase = target_phase;
                    Vaults::<T>::insert(crowdloan, vault.id, vault);

                    match target_phase {
                        VaultPhase::Failed => {
                            Self::deposit_event(Event::<T>::VaultFailed(crowdloan));
                        }
                        VaultPhase::Expired => {
                            Self::deposit_event(Event::<T>::VaultExpired(crowdloan));
                        }
                        _ => { /* do nothing */ }
                    }
                }
                _ => {}
            }

            if executed {
                XcmRequests::<T>::remove(&query_id);
            }

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

        pub(crate) fn id_from_index(index: TrieIndex, kind: ChildStorageKind) -> child::ChildInfo {
            let mut buf = Vec::new();
            buf.extend_from_slice({
                match kind {
                    ChildStorageKind::Pending => b"crowdloan:pending",
                    ChildStorageKind::Flying => b"crowdloan:flying",
                    ChildStorageKind::Contributed => b"crowdloan:contributed",
                }
            });
            buf.extend_from_slice(&index.encode()[..]);
            child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
        }

        pub(crate) fn contribution_put(
            index: TrieIndex,
            who: &T::AccountId,
            balance: &BalanceOf<T>,
            referral_code: &[u8],
            kind: ChildStorageKind,
        ) {
            who.using_encoded(|b| {
                child::put(
                    &Self::id_from_index(index, kind),
                    b,
                    &(balance, referral_code),
                )
            });
        }

        pub(crate) fn contribution_get(
            index: TrieIndex,
            who: &T::AccountId,
            kind: ChildStorageKind,
        ) -> (BalanceOf<T>, Vec<u8>) {
            who.using_encoded(|b| {
                child::get_or_default::<(BalanceOf<T>, Vec<u8>)>(
                    &Self::id_from_index(index, kind),
                    b,
                )
            })
        }

        pub(crate) fn contribution_kill(
            index: TrieIndex,
            who: &T::AccountId,
            kind: ChildStorageKind,
        ) {
            who.using_encoded(|b| child::kill(&Self::id_from_index(index, kind), b));
        }

        fn contribution_iterator(
            index: TrieIndex,
            kind: ChildStorageKind,
        ) -> ChildTriePrefixIterator<(T::AccountId, (BalanceOf<T>, Vec<u8>))> {
            ChildTriePrefixIterator::<_>::with_prefix_over_key::<Identity>(
                &Self::id_from_index(index, kind),
                &[],
            )
        }

        #[require_transactional]
        fn do_contribute(
            who: &AccountIdOf<T>,
            crowdloan: ParaId,
            amount: BalanceOf<T>,
            referral_code: Vec<u8>,
        ) -> Result<(), DispatchError> {
            let query_id = T::XCM::do_contribute(
                crowdloan,
                T::AccountIdToMultiLocation::convert(T::RefundLocation::get()),
                T::RelayCurrency::get(),
                amount,
                who,
                Self::notify_placeholder(),
            )?;

            XcmRequests::<T>::insert(
                query_id,
                XcmRequest::Contribute {
                    crowdloan,
                    who: who.clone(),
                    amount,
                    referral_code: referral_code.clone(),
                },
            );

            Self::deposit_event(Event::<T>::VaultDoContributing(
                crowdloan,
                who.clone(),
                amount,
                referral_code,
            ));

            Ok(())
        }

        #[require_transactional]
        fn do_withdraw(
            crowdloan: ParaId,
            amount: BalanceOf<T>,
            target_phase: VaultPhase,
        ) -> Result<(), DispatchError> {
            log::trace!(
                target: "crowdloans::do_withdraw",
                "para_id: {:?}, amount: {:?}",
                &crowdloan,
                &amount,
            );

            let query_id = T::XCM::do_withdraw(
                crowdloan,
                T::AccountIdToMultiLocation::convert(T::RefundLocation::get()),
                T::RelayCurrency::get(),
                Self::para_account_id(),
                Self::notify_placeholder(),
            )?;

            XcmRequests::<T>::insert(
                query_id,
                XcmRequest::Withdraw {
                    crowdloan,
                    amount,
                    target_phase,
                },
            );

            Self::deposit_event(Event::<T>::VaultDoWithdrawing(
                crowdloan,
                amount,
                target_phase,
            ));
            Ok(())
        }
    }
}
