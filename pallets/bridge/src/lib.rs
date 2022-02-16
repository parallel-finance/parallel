// The main logic reference to chainbridge-substrate v1
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

//! # Bridge pallet
//!
//! ## Overview
//!
//! The bridge pallet implement the transfer of tokens between `parallel` and `eth chains`
//! and the security of funds is secured by multiple signatures mechanism

#![cfg_attr(not(feature = "std"), no_std)]

use crate::types::{BridgeToken, MaterializeCall, Proposal, ProposalStatus};
use frame_support::{
    log,
    pallet_prelude::*,
    require_transactional,
    traits::{
        tokens::fungibles::{Inspect, Mutate, Transfer},
        ChangeMembers, Get, SortedMembers,
    },
    transactional, PalletId,
};
use frame_system::{ensure_signed_or_root, pallet_prelude::*};
use primitives::{Balance, BridgeId, ChainId, ChainNonce, CurrencyId, Ratio};
use scale_info::prelude::{vec, vec::Vec};
use sp_runtime::traits::AccountIdConversion;

mod benchmarking;
mod mock;
mod tests;
mod types;
pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

type AssetIdOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

type BalanceOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

type MaterializeCallOf<T> =
    MaterializeCall<CurrencyId, <T as frame_system::Config>::AccountId, BalanceOf<T>>;

type ProposalOf<T> =
    Proposal<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber>;

pub type TeleAccount = Vec<u8>;

#[frame_support::pallet]
pub mod pallet {
    use primitives::BridgeId;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Admin members has permission to manage the pallet
        type AdminMembers: SortedMembers<Self::AccountId>;

        /// The origin which can update bridge configurations
        /// register or unregister the chains
        /// register or unregister the bridge
        /// set bridge token fee.
        /// Root can always do this.
        type OperateOrigin: EnsureOrigin<Self::Origin>;

        /// The root operator account id
        #[pallet::constant]
        type RootOperatorAccountId: Get<Self::AccountId>;

        /// Assets for teleport/materialize assets to/from bridge pallet
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        /// The identifier for this chain.
        /// This must be unique and must not collide with existing IDs within a set of bridged chains.
        #[pallet::constant]
        type ChainId: Get<ChainId>;

        /// The bridge's pallet id, keep all teleported assets.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Each proposal can live up to [ProposalLifetime] blocks
        #[pallet::constant]
        type ProposalLifetime: Get<Self::BlockNumber>;

        /// The threshold percentage of relayers required to approve a proposal
        #[pallet::constant]
        type ThresholdPercentage: Get<u32>;

        /// Information on runtime weights.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    /// Error for the Bridge Pallet
    #[pallet::error]
    pub enum Error<T> {
        /// The new threshold is invalid
        InvalidVoteThreshold,
        /// Origin has no permission to operate on the bridge
        OriginNoPermission,
        /// The chain_id is invalid, it cannot be a existed chain_id or this chain_id
        ChainIdAlreadyRegistered,
        /// The chain_id is not registered and the related operation will be invalid
        ChainIdNotRegistered,
        /// The bridge token is invalid, it cannot be a existed bridge_token_id
        BridgeTokenAlreadyRegistered,
        /// The bridge token is not registered and the related operation will be invalid
        BridgeTokenNotRegistered,
        /// The AdminMember already vote for the proposal
        MemberAlreadyVoted,
        /// The bridged amount is too low
        BridgedAmountTooLow,
        /// No proposal was found
        ProposalDoesNotExist,
        /// Proposal has been finished
        ProposalAlreadyComplete,
        /// The proposal has exceeded its life time.
        ProposalExpired,
    }

    /// Event for the Bridge Pallet
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Vote threshold has changed
        /// [vote_threshold]
        VoteThresholdChanged(u32),

        /// New chain_id has been registered
        /// [chain_id]
        ChainRegistered(ChainId),

        /// The chain_id has been unregistered
        /// [chain_id]
        ChainRemoved(ChainId),

        /// New bridge_token_id has been registered
        /// [asset_id, bridge_token_id, external, fee]
        BridgeTokenRegistered(AssetIdOf<T>, CurrencyId, bool, BalanceOf<T>),

        /// The bridge_token_id has been unregistered
        /// [asset_id, bridge_token_id]
        BridgeTokenRemoved(AssetIdOf<T>, CurrencyId),

        /// Bridge token fee has changed
        /// [bridge_token_id, fee]
        BridgeTokenFeeChanged(CurrencyId, BalanceOf<T>),

        /// Event emitted when bridge token is destoryed by teleportation
        /// [ori_address, dest_id, chain_nonce, bridge_token_id, dst_address, amount, fee]
        TeleportBurned(
            T::AccountId,
            ChainId,
            ChainNonce,
            CurrencyId,
            TeleAccount,
            BalanceOf<T>,
            BalanceOf<T>,
        ),

        /// Event emitted when a proposal is initialized by materialization
        /// [voter, src_id, src_nonce, bridge_token_id, dst_address, amount]
        MaterializeInitialized(
            T::AccountId,
            ChainId,
            ChainNonce,
            CurrencyId,
            T::AccountId,
            BalanceOf<T>,
        ),

        /// Event emitted when bridge token is issued by materialization
        /// [src_id, chain_nonce, bridge_token_id, dst_address, amount]
        MaterializeMinted(ChainId, ChainNonce, CurrencyId, T::AccountId, BalanceOf<T>),

        /// Vote for a proposal
        /// [src_id, src_nonce, voter, bridge_token_id, dst_address, amount]
        MaterializeVoteFor(
            ChainId,
            ChainNonce,
            T::AccountId,
            CurrencyId,
            T::AccountId,
            BalanceOf<T>,
        ),

        /// Vote against a proposal
        /// [src_id, src_nonce, voter, bridge_token_id, dst_address, amount]
        MaterializeVoteAgainst(
            ChainId,
            ChainNonce,
            T::AccountId,
            CurrencyId,
            T::AccountId,
            BalanceOf<T>,
        ),

        /// Proposal was approved successfully
        /// [src_id, src_nonce]
        ProposalApproved(ChainId, ChainNonce),

        /// Proposal was rejected
        /// [src_id, src_nonce]
        ProposalRejected(ChainId, ChainNonce),
    }

    #[pallet::type_value]
    pub fn DefaultVoteThreshold() -> u32 {
        1u32
    }
    #[pallet::storage]
    #[pallet::getter(fn vote_threshold)]
    pub type VoteThreshold<T: Config> = StorageValue<_, u32, ValueQuery, DefaultVoteThreshold>;

    #[pallet::storage]
    #[pallet::getter(fn chain_nonces)]
    pub type ChainNonces<T: Config> = StorageMap<_, Blake2_256, ChainId, ChainNonce, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn bridge_registry)]
    pub type BridgeRegistry<T: Config> =
        StorageMap<_, Blake2_128Concat, ChainId, Vec<BridgeId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn bridge_tokens)]
    pub type BridgeTokens<T: Config> =
        StorageMap<_, Twox64Concat, AssetIdOf<T>, BridgeToken, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn asset_ids)]
    pub type AssetIds<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, AssetIdOf<T>, ValueQuery>;

    /// Mapping of [chain_id -> (nonce, call) -> proposal]
    #[pallet::storage]
    #[pallet::getter(fn votes)]
    pub type ProposalVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ChainId,
        Blake2_128Concat,
        (ChainNonce, MaterializeCallOf<T>),
        ProposalOf<T>,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set the threshold required to reach multi-signature consensus
        #[pallet::weight(T::WeightInfo::set_vote_threshold())]
        #[transactional]
        pub fn set_vote_threshold(origin: OriginFor<T>, threshold: u32) -> DispatchResult {
            T::OperateOrigin::ensure_origin(origin)?;
            Self::ensure_valid_threshold(threshold, Self::get_members_count())?;

            // Set a new voting threshold
            VoteThreshold::<T>::put(threshold);
            Self::deposit_event(Event::VoteThresholdChanged(threshold));

            Ok(())
        }

        /// Register the specified chain_id
        ///
        /// Only registered chains are allowed to do cross-chain
        ///
        /// - `chain_id`: should be unique.
        #[pallet::weight(T::WeightInfo::register_chain())]
        #[transactional]
        pub fn register_chain(origin: OriginFor<T>, chain_id: ChainId) -> DispatchResult {
            T::OperateOrigin::ensure_origin(origin)?;

            // Registered chain_id cannot be pallet's chain_id or a existed chain_id
            ensure!(
                chain_id != T::ChainId::get() && !Self::chain_registered(chain_id),
                Error::<T>::ChainIdAlreadyRegistered
            );

            // Write a new chain_id into storage
            ChainNonces::<T>::insert(chain_id, 0);
            let inital_registry: Vec<BridgeId> = vec![];
            BridgeRegistry::<T>::insert(chain_id, inital_registry);
            Self::deposit_event(Event::ChainRegistered(chain_id));

            Ok(())
        }

        /// Unregister the specified chain_id
        #[pallet::weight(T::WeightInfo::unregister_chain())]
        #[transactional]
        pub fn unregister_chain(origin: OriginFor<T>, id: ChainId) -> DispatchResult {
            T::OperateOrigin::ensure_origin(origin)?;

            // Unregistered chain_id should be existed
            Self::ensure_chain_registered(id)?;

            // Unregister the chain_id
            ChainNonces::<T>::remove(id);
            BridgeRegistry::<T>::remove(id);

            Self::deposit_event(Event::ChainRemoved(id));

            Ok(())
        }

        /// Register the specified bridge_token_id
        ///
        /// Only registered bridge_tokens are allowed to cross-chain
        ///
        /// - `bridge_token`: bridge_token_id should be unique.
        #[pallet::weight(T::WeightInfo::register_bridge_token())]
        #[transactional]
        pub fn register_bridge_token(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            bridge_token: BridgeToken,
        ) -> DispatchResultWithPostInfo {
            T::OperateOrigin::ensure_origin(origin)?;

            ensure!(
                !BridgeTokens::<T>::contains_key(asset_id)
                    && !AssetIds::<T>::contains_key(bridge_token.clone().id),
                Error::<T>::BridgeTokenAlreadyRegistered,
            );

            BridgeTokens::<T>::insert(asset_id, bridge_token.clone());
            AssetIds::<T>::insert(bridge_token.id, asset_id);

            Self::deposit_event(Event::BridgeTokenRegistered(
                asset_id,
                bridge_token.id,
                bridge_token.external,
                bridge_token.fee,
            ));
            Ok(().into())
        }

        /// Unregister the specified bridge_token_id
        #[pallet::weight(T::WeightInfo::unregister_bridge_token())]
        #[transactional]
        pub fn unregister_bridge_token(
            origin: OriginFor<T>,
            bridge_token_id: CurrencyId,
        ) -> DispatchResultWithPostInfo {
            T::OperateOrigin::ensure_origin(origin)?;
            Self::ensure_bridge_token_registered(bridge_token_id)?;

            let asset_id = AssetIds::<T>::get(bridge_token_id);
            BridgeTokens::<T>::remove(asset_id);
            AssetIds::<T>::remove(bridge_token_id);

            Self::deposit_event(Event::BridgeTokenRemoved(asset_id, bridge_token_id));
            Ok(().into())
        }

        /// Set the cross-chain transaction fee for a registered bridge token
        #[pallet::weight(T::WeightInfo::set_bridge_token_fee())]
        #[transactional]
        pub fn set_bridge_token_fee(
            origin: OriginFor<T>,
            bridge_token_id: CurrencyId,
            new_fee: BalanceOf<T>,
        ) -> DispatchResult {
            T::OperateOrigin::ensure_origin(origin)?;
            Self::ensure_bridge_token_registered(bridge_token_id)?;

            let asset_id = AssetIds::<T>::get(bridge_token_id);
            BridgeTokens::<T>::mutate(asset_id, |token| {
                token.fee = new_fee;
            });

            Self::deposit_event(Event::BridgeTokenFeeChanged(bridge_token_id, new_fee));
            Ok(())
        }

        /// Teleport the bridge token to specified recipient in the destination chain
        ///
        /// Transfer funds from one account to an account in another registered chain.
        /// Support for native token and tokens of Assets pallet
        /// The caller's assets will be locked into palletId
        ///
        /// - `dest_id`: chain_id of the destination chain, should be registered.
        /// - `bridge_token_id`: bridge token should be registered before teleport.
        /// - `to`: recipient of the bridge token of another chain
        /// - `amount`: amount to be teleported, the decimal of bridge token may be different
        #[pallet::weight(T::WeightInfo::teleport())]
        #[transactional]
        pub fn teleport(
            origin: OriginFor<T>,
            dest_id: ChainId,
            bridge_token_id: CurrencyId,
            to: TeleAccount,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_chain_registered(dest_id)?;
            Self::ensure_bridge_token_registered(bridge_token_id)?;
            Self::ensure_amount_valid(amount)?;

            let asset_id = AssetIds::<T>::get(bridge_token_id);
            let BridgeToken { external, fee, .. } = BridgeTokens::<T>::get(asset_id);
            let actual_amount = amount
                .checked_sub(fee)
                .ok_or(Error::<T>::BridgedAmountTooLow)?;
            if external {
                T::Assets::burn_from(asset_id, &who, amount)?;
                T::Assets::mint_into(asset_id, &Self::account_id(), fee)?;
            } else {
                T::Assets::transfer(asset_id, &who, &Self::account_id(), amount, false)?;
            }

            Self::teleport_internal(who, dest_id, bridge_token_id, to, actual_amount, fee)
        }

        /// Materialize the bridge token to specified recipient in this chain
        ///
        /// The first call to the same cross-chain transaction will create a proposal
        /// And subsequent calls will update the existing state until completion
        ///
        /// - `src_id`: chain_id of the source chain, should be registered.
        /// - `src_nonce`: nonce of the source chain, should be unique to identify the cross-cahin tx.
        /// - `bridge_token_id`: bridge_token_id of the bridge token to be materialized, should be registered.
        /// - `to`: recipient of the bridge token of this chain
        /// - `amount`: amount to be materialized, the decimal of bridge token may be different
        /// - `favour`: whether to favour the cross-chain transaction or not, always be true for now.
        #[pallet::weight(T::WeightInfo::materialize())]
        #[transactional]
        pub fn materialize(
            origin: OriginFor<T>,
            src_id: ChainId,
            src_nonce: ChainNonce,
            bridge_token_id: CurrencyId,
            to: T::AccountId,
            amount: BalanceOf<T>,
            favour: bool,
        ) -> DispatchResult {
            let who = Self::ensure_admin(origin)?;
            Self::ensure_chain_registered(src_id)?;
            Self::ensure_chain_nonce_valid(src_id, src_nonce)?;
            Self::ensure_bridge_token_registered(bridge_token_id)?;
            Self::ensure_amount_valid(amount)?;

            let call = MaterializeCall {
                bridge_token_id,
                to,
                amount,
            };
            Self::commit_vote(who, src_id, src_nonce, call.clone(), favour)?;
            Self::resolve_proposal(src_id, src_nonce, call)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_initialize(block_number: T::BlockNumber) -> u64 {
            let expired =
                ProposalVotes::<T>::iter().filter(|x| (*x).2.can_be_cleaned_up(block_number));
            expired.for_each(|x| {
                let chain_id = x.0;
                let chain_nonce = x.1;
                ProposalVotes::<T>::remove(chain_id, chain_nonce);
            });

            0
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Provides an AccountId for the bridge pallet.
    /// Used for teleport/materialize account.
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    /// Checks if the origin is operateOrigin or admin members
    fn ensure_admin(origin: T::Origin) -> Result<T::AccountId, Error<T>> {
        let who = match ensure_signed_or_root(origin.clone())
            .map_err(|_| Error::<T>::OriginNoPermission)?
        {
            Some(account_id) => account_id,
            None => T::RootOperatorAccountId::get(),
        };
        if T::OperateOrigin::ensure_origin(origin).is_err() {
            ensure!(
                T::AdminMembers::contains(&who),
                Error::<T>::OriginNoPermission
            );
        }

        Ok(who)
    }

    /// Checks if a chain is registered
    fn chain_registered(id: ChainId) -> bool {
        ChainNonces::<T>::contains_key(id)
    }

    fn ensure_chain_registered(id: ChainId) -> DispatchResult {
        ensure!(Self::chain_registered(id), Error::<T>::ChainIdNotRegistered);

        Ok(())
    }

    fn ensure_chain_nonce_valid(id: ChainId, nonce: ChainNonce) -> DispatchResult {
        ensure!(
            !Self::has_bridged(id, nonce),
            Error::<T>::ProposalAlreadyComplete
        );

        Ok(())
    }

    fn ensure_amount_valid(amount: BalanceOf<T>) -> DispatchResult {
        ensure!(amount > 0, Error::<T>::BridgedAmountTooLow);

        Ok(())
    }

    /// Checks if a bridge_token_id is registered
    fn ensure_bridge_token_registered(bridge_token_id: CurrencyId) -> DispatchResult {
        ensure!(
            AssetIds::<T>::contains_key(bridge_token_id),
            Error::<T>::BridgeTokenNotRegistered
        );

        Ok(())
    }

    /// Get the count of members in the `AdminMembers`.
    pub fn get_members_count() -> u32 {
        T::AdminMembers::count() as u32
    }

    /// Check if the threshold is satisfied
    pub fn ensure_valid_threshold(threshold: u32, total: u32) -> DispatchResult {
        ensure!(
            Ratio::from_rational(threshold, total)
                >= Ratio::from_percent(T::ThresholdPercentage::get())
                && threshold > 0
                && threshold <= total,
            Error::<T>::InvalidVoteThreshold
        );

        Ok(())
    }

    /// Increments the chain nonce for the specified chain_id
    fn bump_nonce(id: ChainId) -> ChainNonce {
        let nonce = Self::chain_nonces(id) + 1;
        ChainNonces::<T>::insert(id, nonce);

        nonce
    }

    fn merge_overlapping_intervals(mut registry: Vec<BridgeId>) -> Vec<BridgeId> {
        registry.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        let mut merged: Vec<BridgeId> = vec![];
        for r in registry {
            if merged.is_empty() {
                merged.push(r);
            } else if let Some(last_merged) = merged.last_mut() {
                if r.0 > last_merged.1 {
                    merged.push(r);
                } else {
                    (*last_merged).1 = r.1.max(last_merged.1);
                }
            }
        }
        merged
    }

    pub fn has_bridged(id: ChainId, nonce: ChainNonce) -> bool {
        BridgeRegistry::<T>::get(&id).map_or(false, |registry| {
            registry.iter().any(|&r| (nonce >= r.0 && nonce <= r.1))
        })
    }

    /// Records completed bridge transactions
    #[require_transactional]
    fn update_bridge_registry(id: ChainId, nonce: ChainNonce) {
        if BridgeRegistry::<T>::get(&id).is_none() {
            return;
        }
        let mut registry = BridgeRegistry::<T>::get(&id).unwrap();
        registry.iter_mut().for_each(|x| {
            match *x {
                (nonce_start, _) if nonce_start == (nonce + 1) => x.0 = nonce,
                (_, nonce_end) if nonce_end == (nonce - 1) => x.1 = nonce,
                _ => (),
            };
        });
        let mut registry = Self::merge_overlapping_intervals(registry);
        if !registry.iter().any(|&r| (nonce >= r.0 && nonce <= r.1)) {
            registry.push((nonce, nonce));
        }
        registry.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        BridgeRegistry::<T>::insert(id, registry);
    }

    /// Initiates a transfer of the bridge token
    #[require_transactional]
    fn teleport_internal(
        ori_address: T::AccountId,
        dest_id: ChainId,
        bridge_token_id: CurrencyId,
        dst_address: TeleAccount,
        amount: BalanceOf<T>,
        fee: BalanceOf<T>,
    ) -> DispatchResult {
        let nonce = Self::bump_nonce(dest_id);

        log::trace!(
            target: "bridge::teleport_internal",
            "ori_address: {:?}, dest_id {:?}, nonce {:?},
                bridge_token_id: {:?}, dst_address {:?}, amount: {:?}, fee: {:?}",
            ori_address,
            dest_id,
            nonce,
            bridge_token_id,
            dst_address,
            amount,
            fee,
        );

        Self::deposit_event(Event::TeleportBurned(
            ori_address,
            dest_id,
            nonce,
            bridge_token_id,
            dst_address,
            amount,
            fee,
        ));
        Ok(())
    }

    #[require_transactional]
    fn commit_vote(
        who: T::AccountId,
        src_id: ChainId,
        src_nonce: ChainNonce,
        call: MaterializeCallOf<T>,
        favour: bool,
    ) -> DispatchResult {
        let now = <frame_system::Pallet<T>>::block_number();

        let mut proposal = match Self::votes(src_id, (src_nonce, call.clone())) {
            Some(p) => p,
            None => {
                let MaterializeCall {
                    bridge_token_id,
                    to,
                    amount,
                } = call.clone();

                Self::deposit_event(Event::<T>::MaterializeInitialized(
                    who.clone(),
                    src_id,
                    src_nonce,
                    bridge_token_id,
                    to,
                    amount,
                ));
                Proposal {
                    expiry: now + T::ProposalLifetime::get(),
                    ..Default::default()
                }
            }
        };

        // Ensure the proposal isn't complete and member hasn't already voted
        ensure!(!proposal.is_complete(), Error::<T>::ProposalAlreadyComplete);
        ensure!(!proposal.is_expired(now), Error::<T>::ProposalExpired);
        ensure!(!proposal.has_voted(&who), Error::<T>::MemberAlreadyVoted);

        let MaterializeCall {
            bridge_token_id,
            to,
            amount,
        } = call.clone();
        if favour {
            proposal.votes_for.push(who.clone());
            Self::deposit_event(Event::<T>::MaterializeVoteFor(
                src_id,
                src_nonce,
                who,
                bridge_token_id,
                to,
                amount,
            ));
        } else {
            proposal.votes_against.push(who.clone());
            Self::deposit_event(Event::MaterializeVoteAgainst(
                src_id,
                src_nonce,
                who,
                bridge_token_id,
                to,
                amount,
            ));
        }

        ProposalVotes::<T>::insert(src_id, (src_nonce, call), proposal.clone());

        Ok(())
    }

    /// Attempts to finalize or cancel the proposal if the vote count allows.
    #[require_transactional]
    fn resolve_proposal(
        src_id: ChainId,
        src_nonce: ChainNonce,
        call: MaterializeCallOf<T>,
    ) -> DispatchResult {
        if let Some(mut proposal) = ProposalVotes::<T>::get(src_id, (src_nonce, call.clone())) {
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(!proposal.is_complete(), Error::<T>::ProposalAlreadyComplete);
            ensure!(!proposal.is_expired(now), Error::<T>::ProposalExpired);

            let status =
                proposal.try_to_complete(Self::vote_threshold(), Self::get_members_count());
            ProposalVotes::<T>::insert(src_id, (src_nonce, call.clone()), proposal.clone());

            match status {
                ProposalStatus::Approved => Self::execute_materialize(src_id, src_nonce, call),
                ProposalStatus::Rejected => Self::cancel_materialize(src_id, src_nonce),
                _ => Ok(()),
            }
        } else {
            Err(Error::<T>::ProposalDoesNotExist.into())
        }
    }

    fn execute_materialize(
        src_id: ChainId,
        src_nonce: ChainNonce,
        call: MaterializeCallOf<T>,
    ) -> DispatchResult {
        Self::ensure_chain_registered(src_id)?;
        Self::ensure_bridge_token_registered(call.bridge_token_id)?;

        Self::deposit_event(Event::ProposalApproved(src_id, src_nonce));

        let asset_id = AssetIds::<T>::get(call.bridge_token_id);
        let BridgeToken { external, .. } = BridgeTokens::<T>::get(asset_id);
        if external {
            T::Assets::mint_into(asset_id, &call.to, call.amount)?;
        } else {
            T::Assets::transfer(asset_id, &Self::account_id(), &call.to, call.amount, true)?;
        }

        Self::update_bridge_registry(src_id, src_nonce);

        Self::deposit_event(Event::MaterializeMinted(
            src_id,
            src_nonce,
            call.bridge_token_id,
            call.to,
            call.amount,
        ));
        Ok(())
    }

    /// Cancels a proposal.
    fn cancel_materialize(src_id: ChainId, src_nonce: ChainNonce) -> DispatchResult {
        Self::update_bridge_registry(src_id, src_nonce);
        Self::deposit_event(Event::ProposalRejected(src_id, src_nonce));

        Ok(())
    }
}

impl<T: Config> ChangeMembers<T::AccountId> for Pallet<T> {
    fn change_members_sorted(
        _incoming: &[T::AccountId],
        _outgoing: &[T::AccountId],
        _new: &[T::AccountId],
    ) {
        // nothing
    }

    fn set_prime(_prime: Option<T::AccountId>) {
        // nothing
    }
}
