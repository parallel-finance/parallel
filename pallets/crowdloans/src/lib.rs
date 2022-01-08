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
    use primitives::{Balance, CurrencyId, LeasePeriod, ParaId, TrieIndex, VaultId};
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
        /// [para_id, vault_id, ctoken_id, phase, contribution_strategy, cap, end_block, trie_index]
        VaultCreated(
            ParaId,
            VaultId,
            AssetIdOf<T>,
            VaultPhase,
            ContributionStrategy,
            BalanceOf<T>,
            BlockNumberFor<T>,
            TrieIndex,
        ),
        /// Existing vault was updated
        /// [para_id, vault_id, contribution_strategy, cap, end_block]
        VaultUpdated(
            ParaId,
            VaultId,
            ContributionStrategy,
            BalanceOf<T>,
            BlockNumberFor<T>,
        ),
        /// Vault was opened
        /// [para_id, vault_id, pre_phase, now_phase]
        VaultPhaseUpdated(ParaId, VaultId, VaultPhase, VaultPhase),
        /// Vault is trying to do contributing
        /// [para_id, vault_id, contributor, amount, referral_code]
        VaultDoContributing(ParaId, VaultId, T::AccountId, BalanceOf<T>, Vec<u8>),
        /// Vault is trying to do withdrawing
        /// [para_id, vault_id, amount, target_phase]
        VaultDoWithdrawing(ParaId, VaultId, BalanceOf<T>, VaultPhase),
        /// Vault successfully contributed
        /// [para_id, vault_id, contributor, amount, referral_code]
        VaultContributed(ParaId, VaultId, T::AccountId, BalanceOf<T>, Vec<u8>),
        /// A user claimed CToken from vault
        /// [para_id, vault_id, ctoken_id, account, amount, phase]
        VaultClaimed(
            ParaId,
            VaultId,
            AssetIdOf<T>,
            T::AccountId,
            BalanceOf<T>,
            VaultPhase,
        ),
        /// A user withdrew contributed assets from vault
        /// [para_id, vault_id, account, amount, phase]
        VaultWithdrew(ParaId, VaultId, T::AccountId, BalanceOf<T>, VaultPhase),
        /// A user redeemed contributed assets using CToken
        /// [para_id, vault_id, ctoken_id, account, amount, phase]
        VaultRedeemed(
            ParaId,
            VaultId,
            AssetIdOf<T>,
            T::AccountId,
            BalanceOf<T>,
            VaultPhase,
        ),
        /// Vrfs updated
        /// [vrf_data]
        VrfsUpdated(BoundedVec<ParaId, T::MaxVrfs>),
        /// Notification received
        /// [multi_location, query_id, res]
        NotificationReceived(Box<MultiLocation>, QueryId, Option<(u32, XcmError)>),
        /// All contributions migrated
        /// [para_id, vault_id]
        AllMigrated(ParaId, VaultId),
        /// Partially contributions migrated
        /// [para_id, vault_id]
        PartiallyMigrated(ParaId, VaultId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Vault is not in correct phase
        IncorrectVaultPhase,
        /// Crowdloan ParaId aready exists
        CrowdloanAlreadyExists,
        /// Contribution is not enough
        InsufficientContribution,
        /// There are no contributions stored in contributed childstorage
        NoContributions,
        /// Balance is not enough
        InsufficientBalance,
        /// Last lease period must be greater than first lease period.
        LastPeriodBeforeFirstPeriod,
        /// CToken does not exist
        CTokenDoesNotExist,
        /// Vault already exists
        VaultAlreadyExists,
        /// Vault does not exist
        VaultDoesNotExist,
        /// CToken for provided (leaseStart, leaseEnd) is different with what has been created previously
        InvalidCToken,
        /// Vault for provided ParaId not ended
        VaultNotEnded,
        /// No contributions allowed during the VRF delay
        VrfDelayInProgress,
        /// Attempted contribution violates contribution cap
        ExceededCap,
        /// Current relay block is greater than vault end block
        ExceededEndBlock,
        /// Exceeded maximum vrfs
        ExceededMaxVrfs,
        /// Capacity cannot be zero value
        ZeroCap,
        /// Invalid params input
        InvalidParams,
    }

    #[pallet::storage]
    #[pallet::getter(fn vaults)]
    pub type Vaults<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, ParaId>,
            NMapKey<Blake2_128Concat, LeasePeriod>,
            NMapKey<Blake2_128Concat, LeasePeriod>,
        ),
        Vault<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn vrfs)]
    pub type Vrfs<T: Config> =
        StorageValue<_, BoundedVec<ParaId, <T as Config>::MaxVrfs>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn ctoken_of)]
    pub type CTokensRegistry<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, LeasePeriod>,
            NMapKey<Blake2_128Concat, LeasePeriod>,
        ),
        AssetIdOf<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn current_lease)]
    pub type LeasesRegistry<T: Config> =
        StorageMap<_, Blake2_128Concat, ParaId, (LeasePeriod, LeasePeriod), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_trie_index)]
    pub type NextTrieIndex<T> = StorageValue<_, TrieIndex, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn xcm_request)]
    pub type XcmRequests<T> = StorageMap<_, Blake2_128Concat, QueryId, XcmRequest<T>, OptionQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new vault via a governance decision
        ///
        /// - `crowdloan`: parachain id of the crowdloan, should be consistent with relaychain
        /// - `ctoken`: ctoken is used for the vault, should be unique
        /// - `lease_start`: lease start index
        /// - `lease_end`: lease end index
        /// - `contribution_strategy`: currently, only XCM strategy is supported.
        /// - `cap`: the capacity limit for the vault
        /// - `end_block`: the crowdloan end block for the vault
        #[pallet::weight(<T as Config>::WeightInfo::create_vault())]
        #[transactional]
        pub fn create_vault(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            ctoken: AssetIdOf<T>,
            lease_start: LeasePeriod,
            lease_end: LeasePeriod,
            contribution_strategy: ContributionStrategy,
            #[pallet::compact] cap: BalanceOf<T>,
            end_block: BlockNumberFor<T>,
        ) -> DispatchResult {
            T::CreateVaultOrigin::ensure_origin(origin)?;

            ensure!(!cap.is_zero(), Error::<T>::ZeroCap);

            ensure!(
                lease_start <= lease_end,
                Error::<T>::LastPeriodBeforeFirstPeriod
            );

            if let Some(c) = Self::ctoken_of((&lease_start, &lease_end)) {
                ensure!(c == ctoken, Error::<T>::InvalidCToken);
            }

            ensure!(
                !Vaults::<T>::contains_key((&crowdloan, &lease_start, &lease_end)),
                Error::<T>::VaultAlreadyExists
            );

            // origin shouldn't be able to create a new vault if the previous one is not finished
            if let Some(vault) = Self::current_vault(crowdloan) {
                if vault.phase != VaultPhase::Failed && vault.phase != VaultPhase::Expired {
                    return Err(DispatchError::from(Error::<T>::VaultNotEnded));
                }
            }

            ensure!(
                T::RelayChainBlockNumberProvider::current_block_number() <= end_block,
                Error::<T>::ExceededEndBlock
            );

            let trie_index = Self::next_trie_index();
            let next_trie_index = trie_index.checked_add(1).ok_or(ArithmeticError::Overflow)?;
            let new_vault = Vault::new(
                lease_start,
                lease_end,
                ctoken,
                contribution_strategy,
                cap,
                end_block,
                trie_index,
            );

            log::trace!(
                target: "crowdloans::create_vault",
                "para_id: {:?}, lease_start: {:?}, lease_end: {:?}, trie_index: {:?}, ctoken: {:?}",
                crowdloan,
                lease_start,
                lease_end,
                trie_index,
                ctoken
            );

            NextTrieIndex::<T>::put(next_trie_index);
            Vaults::<T>::insert((&crowdloan, &lease_start, &lease_end), new_vault);
            CTokensRegistry::<T>::insert((&lease_start, &lease_end), ctoken);
            LeasesRegistry::<T>::insert(&crowdloan, (lease_start, lease_end));

            Self::deposit_event(Event::<T>::VaultCreated(
                crowdloan,
                (lease_start, lease_end),
                ctoken,
                VaultPhase::Pending,
                contribution_strategy,
                cap,
                end_block,
                trie_index,
            ));

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

            let Vault {
                lease_start,
                lease_end,
                contribution_strategy,
                cap,
                end_block,
                ..
            } = vault;

            Vaults::<T>::insert((&crowdloan, &lease_start, &lease_end), vault);

            Self::deposit_event(Event::<T>::VaultUpdated(
                crowdloan,
                (lease_start, lease_end),
                contribution_strategy,
                cap,
                end_block,
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
                vault.phase = VaultPhase::Contributing;
                Self::deposit_event(Event::<T>::VaultPhaseUpdated(
                    crowdloan,
                    (vault.lease_start, vault.lease_end),
                    VaultPhase::Pending,
                    VaultPhase::Contributing,
                ));
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
                &Self::account_id(),
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

                Self::do_contribute(
                    &who,
                    crowdloan,
                    (vault.lease_start, vault.lease_end),
                    amount,
                    referral_code.clone(),
                )?;
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

            Vaults::<T>::insert(
                (
                    &crowdloan,
                    &vault.lease_start.clone(),
                    &vault.lease_end.clone(),
                ),
                vault,
            );

            log::trace!(
                target: "crowdloans::contribute",
                "who: {:?}, para_id: {:?}, amount: {:?}, referral_code: {:?}",
                &who,
                &crowdloan,
                &amount,
                &referral_code
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
                Self::deposit_event(Event::<T>::VaultPhaseUpdated(
                    crowdloan,
                    (vault.lease_start, vault.lease_end),
                    VaultPhase::Contributing,
                    VaultPhase::Closed,
                ));
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
                Self::deposit_event(Event::<T>::VaultPhaseUpdated(
                    crowdloan,
                    (vault.lease_start, vault.lease_end),
                    VaultPhase::Closed,
                    VaultPhase::Contributing,
                ));
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
                Self::deposit_event(Event::<T>::VaultPhaseUpdated(
                    crowdloan,
                    (vault.lease_start, vault.lease_end),
                    VaultPhase::Closed,
                    VaultPhase::Succeeded,
                ));
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
                Self::do_withdraw(
                    crowdloan,
                    (vault.lease_start, vault.lease_end),
                    vault.contributed,
                    VaultPhase::Failed,
                )?;
                Ok(())
            })
        }

        /// If a `crowdloan` succeeded, claim the liquid derivatives of the
        /// contributed assets
        #[pallet::weight(<T as Config>::WeightInfo::claim())]
        #[transactional]
        pub fn claim(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            lease_start: LeasePeriod,
            lease_end: LeasePeriod,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let ctoken = Self::ctoken_of((&lease_start, &lease_end))
                .ok_or(Error::<T>::CTokenDoesNotExist)?;
            let vault = Self::vaults((&crowdloan, &lease_start, &lease_end))
                .ok_or(Error::<T>::VaultDoesNotExist)?;

            ensure!(
                vault.phase == VaultPhase::Succeeded,
                Error::<T>::IncorrectVaultPhase
            );

            let (amount, _) =
                Self::contribution_get(vault.trie_index, &who, ChildStorageKind::Contributed);
            ensure!(!amount.is_zero(), Error::<T>::NoContributions);

            log::trace!(
                target: "crowdloans::claim",
                "who: {:?}, ctoken: {:?}, amount: {:?}, para_id: {:?}, lease_start: {:?}, lease_end: {:?}",
                &who,
                &ctoken,
                &amount,
                &crowdloan,
                &lease_start,
                &lease_end
            );

            T::Assets::mint_into(ctoken, &who, amount)?;

            Self::contribution_kill(vault.trie_index, &who, ChildStorageKind::Contributed);

            Self::deposit_event(Event::<T>::VaultClaimed(
                crowdloan,
                (vault.lease_start, vault.lease_end),
                ctoken,
                who,
                amount,
                vault.phase,
            ));

            Ok(())
        }

        /// If a `crowdloan` failed, withdraw the contributed assets
        #[pallet::weight(<T as Config>::WeightInfo::withdraw())]
        #[transactional]
        pub fn withdraw(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            lease_start: LeasePeriod,
            lease_end: LeasePeriod,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let vault = Self::vaults((&crowdloan, &lease_start, &lease_end))
                .ok_or(Error::<T>::VaultDoesNotExist)?;

            ensure!(
                vault.phase == VaultPhase::Failed,
                Error::<T>::IncorrectVaultPhase
            );

            let (amount, _) =
                Self::contribution_get(vault.trie_index, &who, ChildStorageKind::Contributed);
            ensure!(!amount.is_zero(), Error::<T>::NoContributions);

            log::trace!(
                target: "crowdloans::withdraw",
                "who: {:?}, amount: {:?}, para_id: {:?}, lease_start: {:?}, lease_end: {:?}",
                &who,
                &amount,
                &crowdloan,
                &lease_start,
                &lease_end
            );

            T::Assets::transfer(
                T::RelayCurrency::get(),
                &Self::account_id(),
                &who,
                amount,
                false,
            )?;

            Self::contribution_kill(vault.trie_index, &who, ChildStorageKind::Contributed);

            Self::deposit_event(Event::<T>::VaultWithdrew(
                crowdloan,
                (vault.lease_start, vault.lease_end),
                who,
                amount,
                vault.phase,
            ));

            Ok(())
        }

        /// If a `crowdloan` expired, redeem the contributed assets
        /// using ctoken
        #[pallet::weight(<T as Config>::WeightInfo::redeem())]
        #[transactional]
        pub fn redeem(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            lease_start: LeasePeriod,
            lease_end: LeasePeriod,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let ctoken = Self::ctoken_of((&lease_start, &lease_end))
                .ok_or(Error::<T>::CTokenDoesNotExist)?;
            let vault = Self::vaults((&crowdloan, &lease_start, &lease_end))
                .ok_or(Error::<T>::VaultDoesNotExist)?;

            ensure!(
                vault.phase == VaultPhase::Expired,
                Error::<T>::IncorrectVaultPhase
            );

            log::trace!(
                target: "crowdloans::redeem",
                "who: {:?}, ctoken: {:?}, amount: {:?}, para_id: {:?}, lease_start: {:?}, lease_end: {:?}",
                &who,
                &ctoken,
                &amount,
                &crowdloan,
                &lease_start,
                &lease_end
            );

            let ctoken_balance = T::Assets::reducible_balance(ctoken, &who, false);
            ensure!(ctoken_balance >= amount, Error::<T>::InsufficientBalance);

            T::Assets::burn_from(ctoken, &who, amount)?;
            T::Assets::transfer(
                T::RelayCurrency::get(),
                &Self::account_id(),
                &who,
                amount,
                false,
            )?;

            Self::deposit_event(Event::<T>::VaultRedeemed(
                crowdloan,
                (vault.lease_start, vault.lease_end),
                ctoken,
                who,
                amount,
                vault.phase,
            ));

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
                Self::do_withdraw(
                    crowdloan,
                    (vault.lease_start, vault.lease_end),
                    vault.contributed,
                    VaultPhase::Expired,
                )?;
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
            ensure!(!Self::has_vrfs(), Error::<T>::VrfDelayInProgress);

            let contributions =
                Self::contribution_iterator(vault.trie_index, ChildStorageKind::Pending);
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
                Self::do_contribute(
                    &who,
                    crowdloan,
                    (vault.lease_start, vault.lease_end),
                    amount,
                    referral_code,
                )?;
                migrated_count += 1;
            }

            let Vault {
                lease_start,
                lease_end,
                ..
            } = vault;
            Vaults::<T>::insert((&crowdloan, &lease_start, &lease_end), vault);

            if all_migrated {
                Self::deposit_event(Event::<T>::AllMigrated(crowdloan, (lease_start, lease_end)));
            } else {
                Self::deposit_event(Event::<T>::PartiallyMigrated(
                    crowdloan,
                    (lease_start, lease_end),
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
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
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

        pub(crate) fn current_vault(crowdloan: ParaId) -> Option<Vault<T>> {
            Self::current_lease(crowdloan).and_then(|(lease_start, lease_end)| {
                Self::vaults((&crowdloan, &lease_start, &lease_end))
            })
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
        ) -> Result<Vec<u8>, DispatchError> {
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

            Ok(referral_code)
        }

        #[require_transactional]
        fn do_migrate_contribution(
            who: &AccountIdOf<T>,
            vault: &mut Vault<T>,
            amount: BalanceOf<T>,
            src_child_storage_kind: ChildStorageKind,
            dst_child_storage_kind: ChildStorageKind,
        ) -> DispatchResult {
            let referral_code = Self::do_update_contribution(
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
                Some(referral_code),
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
                    vault_id: (lease_start, lease_end),
                    who,
                    amount,
                    referral_code,
                } if executed => {
                    let mut vault = Self::vaults((&crowdloan, &lease_start, &lease_end))
                        .ok_or(Error::<T>::VaultDoesNotExist)?;
                    T::Assets::burn_from(T::RelayCurrency::get(), &Self::account_id(), amount)?;
                    Self::do_migrate_contribution(
                        &who,
                        &mut vault,
                        amount,
                        ChildStorageKind::Flying,
                        ChildStorageKind::Contributed,
                    )?;
                    Vaults::<T>::insert((&crowdloan, &lease_start, &lease_end), vault);

                    Self::deposit_event(Event::<T>::VaultContributed(
                        crowdloan,
                        (lease_start, lease_end),
                        who,
                        amount,
                        referral_code,
                    ));
                }
                XcmRequest::Contribute {
                    crowdloan,
                    vault_id: (lease_start, lease_end),
                    who,
                    amount,
                    ..
                } if !executed => {
                    let mut vault = Self::vaults((&crowdloan, &lease_start, &lease_end))
                        .ok_or(Error::<T>::VaultDoesNotExist)?;
                    T::Assets::transfer(
                        T::RelayCurrency::get(),
                        &Self::account_id(),
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
                    Vaults::<T>::insert((&crowdloan, &lease_start, &lease_end), vault);
                }
                XcmRequest::Withdraw {
                    crowdloan,
                    vault_id: (lease_start, lease_end),
                    amount,
                    target_phase,
                } if executed => {
                    let mut vault = Self::vaults((&crowdloan, &lease_start, &lease_end))
                        .ok_or(Error::<T>::VaultDoesNotExist)?;
                    T::Assets::mint_into(T::RelayCurrency::get(), &Self::account_id(), amount)?;
                    let pre_phase = sp_std::mem::replace(&mut vault.phase, target_phase);
                    Vaults::<T>::insert((&crowdloan, &lease_start, &lease_end), vault);
                    Self::deposit_event(Event::<T>::VaultPhaseUpdated(
                        crowdloan,
                        (lease_start, lease_end),
                        pre_phase,
                        target_phase,
                    ));
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
            let mut vault = Self::current_vault(crowdloan).ok_or(Error::<T>::VaultDoesNotExist)?;
            ensure!(vault.phase == phase, Error::<T>::IncorrectVaultPhase);
            cb(&mut vault)?;
            Vaults::<T>::insert(
                (
                    &crowdloan,
                    &vault.lease_start.clone(),
                    &vault.lease_end.clone(),
                ),
                vault,
            );
            Ok(())
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
            vault_id: VaultId,
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
                    vault_id,
                    who: who.clone(),
                    amount,
                    referral_code: referral_code.clone(),
                },
            );

            Self::deposit_event(Event::<T>::VaultDoContributing(
                crowdloan,
                vault_id,
                who.clone(),
                amount,
                referral_code,
            ));

            Ok(())
        }

        #[require_transactional]
        fn do_withdraw(
            crowdloan: ParaId,
            vault_id: VaultId,
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
                    vault_id,
                    amount,
                    target_phase,
                },
            );

            Self::deposit_event(Event::<T>::VaultDoWithdrawing(
                crowdloan,
                vault_id,
                amount,
                target_phase,
            ));
            Ok(())
        }
    }
}
