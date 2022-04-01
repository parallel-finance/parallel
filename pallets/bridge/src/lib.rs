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
        tokens::{
            fungibles::{Inspect, Mutate, Transfer},
            BalanceConversion,
        },
        Get, SortedMembers,
    },
    transactional, PalletId,
};
use frame_system::{ensure_signed_or_root, pallet_prelude::*};
use primitives::{Balance, BridgeInterval, ChainId, ChainNonce, CurrencyId, Ratio};
use scale_info::prelude::{vec, vec::Vec};
use sp_runtime::{
    traits::{AccountIdConversion, Zero},
    ArithmeticError,
};

mod benchmarking;
mod mock;
mod tests;
mod types;
pub mod weights;

pub use pallet::*;
use types::BridgeType;
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
    use primitives::BridgeInterval;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Relay members has permission to materialize the assets
        type RelayMembers: SortedMembers<Self::AccountId>;

        /// The origin which can update bridged token
        type UpdateTokenOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can update bridged chain
        type UpdateChainOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can clean accumulated cap value
        type CapOrigin: EnsureOrigin<Self::Origin>;

        /// The root operator account id
        #[pallet::constant]
        type RootOperatorAccountId: Get<Self::AccountId>;

        /// Assets for teleport/materialize assets to/from bridge pallet
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        /// An account to pay the bonus
        #[pallet::constant]
        type GiftAccount: Get<Self::AccountId>;

        /// A bonus amount converter
        type GiftConvert: BalanceConversion<Balance, CurrencyId, Balance>;

        /// Currency id of the native token
        #[pallet::constant]
        type NativeCurrencyId: Get<AssetIdOf<Self>>;

        /// The essential balance for an existed account
        #[pallet::constant]
        type ExistentialDeposit: Get<Balance>;

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
        /// The bridge token is not available in cross-chain
        BridgeTokenDisabled,
        /// The RelayMembers already vote for the proposal
        MemberAlreadyVoted,
        /// The bridging amount is too low
        BridgingAmountTooLow,
        /// The bridging amount is exceed the capacity
        BridgeOutCapExceeded,
        /// The bridging amount is exceed the capacity
        BridgeInCapExceeded,
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
        /// Vote threshold has updated
        /// [vote_threshold]
        VoteThresholdUpdated(u32),

        /// New chain_id has been registered
        /// [chain_id]
        ChainRegistered(ChainId),

        /// The chain_id has been unregistered
        /// [chain_id]
        ChainRemoved(ChainId),

        /// New bridge_token_id has been registered
        /// [asset_id, bridge_token_id, external, fee, enable, out_cap, out_amount, in_cap, in_amount]
        BridgeTokenRegistered(
            AssetIdOf<T>,
            CurrencyId,
            bool,
            BalanceOf<T>,
            bool,
            BalanceOf<T>,
            BalanceOf<T>,
            BalanceOf<T>,
            BalanceOf<T>,
        ),

        /// The bridge_token_id has been unregistered
        /// [asset_id, bridge_token_id]
        BridgeTokenRemoved(AssetIdOf<T>, CurrencyId),

        /// Bridge token fee has updated
        /// [bridge_token_id, fee]
        BridgeTokenFeeUpdated(CurrencyId, BalanceOf<T>),

        /// The status of the bridge token has updated
        /// [bridge_token_id, enabled]
        BridgeTokenStatusUpdated(CurrencyId, bool),

        /// The status of the bridge token cap has updated
        /// [bridge_token_id, bridge_type, new_cap]
        BridgeTokenCapUpdated(CurrencyId, BridgeType, BalanceOf<T>),

        /// The accumulated cap value cleaned
        /// [bridge_token_id, bridge_type]
        BridgeTokenAccumulatedValueCleaned(CurrencyId, BridgeType),

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
    pub type ChainNonces<T: Config> =
        StorageMap<_, Blake2_128Concat, ChainId, ChainNonce, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn bridge_registry)]
    pub type BridgeRegistry<T: Config> =
        StorageMap<_, Blake2_128Concat, ChainId, Vec<BridgeInterval>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn bridge_token)]
    pub type BridgeTokens<T: Config> =
        StorageMap<_, Twox64Concat, AssetIdOf<T>, BridgeToken, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn asset_id)]
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
        /// Register the specified chain_id
        ///
        /// Only registered chains are allowed to do cross-chain
        ///
        /// - `chain_id`: should be unique.
        #[pallet::weight(T::WeightInfo::register_chain())]
        #[transactional]
        pub fn register_chain(origin: OriginFor<T>, chain_id: ChainId) -> DispatchResult {
            T::UpdateChainOrigin::ensure_origin(origin)?;

            // Registered chain_id cannot be pallet's chain_id or a existed chain_id
            ensure!(
                chain_id != T::ChainId::get() && !Self::chain_registered(chain_id),
                Error::<T>::ChainIdAlreadyRegistered
            );

            // Write a new chain_id into storage
            ChainNonces::<T>::insert(chain_id, 0);
            let inital_registry: Vec<BridgeInterval> = vec![];
            BridgeRegistry::<T>::insert(chain_id, inital_registry);
            Self::deposit_event(Event::ChainRegistered(chain_id));

            Ok(())
        }

        /// Unregister the specified chain_id
        #[pallet::weight(T::WeightInfo::unregister_chain())]
        #[transactional]
        pub fn unregister_chain(origin: OriginFor<T>, chain_id: ChainId) -> DispatchResult {
            T::UpdateChainOrigin::ensure_origin(origin)?;

            // Unregistered chain_id should be existed
            Self::ensure_chain_registered(chain_id)?;

            // Unregister the chain_id
            ChainNonces::<T>::remove(chain_id);
            BridgeRegistry::<T>::remove(chain_id);

            Self::deposit_event(Event::ChainRemoved(chain_id));

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
            T::UpdateTokenOrigin::ensure_origin(origin)?;

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
                bridge_token.enable,
                bridge_token.out_cap,
                bridge_token.out_amount,
                bridge_token.in_cap,
                bridge_token.in_amount,
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
            T::UpdateTokenOrigin::ensure_origin(origin)?;
            Self::ensure_bridge_token_registered(bridge_token_id)?;

            let asset_id = Self::asset_id(bridge_token_id);
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
            T::UpdateTokenOrigin::ensure_origin(origin)?;

            Self::try_mutate_bridge_token(bridge_token_id, |token| {
                token.fee = new_fee;

                Self::deposit_event(Event::BridgeTokenFeeUpdated(bridge_token_id, new_fee));
                Ok(())
            })
        }

        /// Set the cross-chain transaction fee for a registered bridge token
        #[pallet::weight(T::WeightInfo::set_bridge_token_status())]
        #[transactional]
        pub fn set_bridge_token_status(
            origin: OriginFor<T>,
            bridge_token_id: CurrencyId,
            enable: bool,
        ) -> DispatchResult {
            T::UpdateTokenOrigin::ensure_origin(origin)?;

            Self::try_mutate_bridge_token(bridge_token_id, |token| {
                token.enable = enable;

                Self::deposit_event(Event::BridgeTokenStatusUpdated(bridge_token_id, enable));
                Ok(())
            })
        }

        /// Set the cross-chain transaction cap for a registered bridge token
        #[pallet::weight(T::WeightInfo::set_bridge_token_cap())]
        #[transactional]
        pub fn set_bridge_token_cap(
            origin: OriginFor<T>,
            bridge_token_id: CurrencyId,
            bridge_type: BridgeType,
            new_cap: BalanceOf<T>,
        ) -> DispatchResult {
            T::UpdateTokenOrigin::ensure_origin(origin)?;

            Self::try_mutate_bridge_token(bridge_token_id, |token| {
                match bridge_type {
                    BridgeType::BridgeOut => token.out_cap = new_cap,
                    BridgeType::BridgeIn => token.in_cap = new_cap,
                };

                Self::deposit_event(Event::BridgeTokenCapUpdated(
                    bridge_token_id,
                    bridge_type,
                    new_cap,
                ));
                Ok(())
            })
        }

        /// Clean the accumulated cap value to make bridge work again
        #[pallet::weight(T::WeightInfo::clean_cap_accumulated_value())]
        #[transactional]
        pub fn clean_cap_accumulated_value(
            origin: OriginFor<T>,
            bridge_token_id: CurrencyId,
            bridge_type: BridgeType,
        ) -> DispatchResult {
            T::CapOrigin::ensure_origin(origin)?;

            Self::try_mutate_bridge_token(bridge_token_id, |token| {
                match bridge_type {
                    BridgeType::BridgeIn => token.in_amount = Zero::zero(),
                    BridgeType::BridgeOut => token.out_amount = Zero::zero(),
                };

                Self::deposit_event(Event::BridgeTokenAccumulatedValueCleaned(
                    bridge_token_id,
                    bridge_type,
                ));
                Ok(())
            })
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

            let asset_id = Self::asset_id(bridge_token_id);
            let BridgeToken {
                external,
                fee,
                enable,
                ..
            } = Self::bridge_token(asset_id);
            ensure!(enable, Error::<T>::BridgeTokenDisabled);
            Self::update_bridge_token_cap(asset_id, amount, BridgeType::BridgeOut)?;

            if external {
                T::Assets::burn_from(asset_id, &who, amount)?;
                T::Assets::mint_into(asset_id, &Self::account_id(), fee)?;
            } else {
                T::Assets::transfer(asset_id, &who, &Self::account_id(), amount, false)?;
            }

            let actual_amount = amount
                .checked_sub(fee)
                .ok_or(Error::<T>::BridgingAmountTooLow)?;
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
            let who = Self::ensure_relay_member(origin)?;
            Self::ensure_chain_registered(src_id)?;
            Self::ensure_chain_nonce_valid(src_id, src_nonce)?;
            Self::materialize_allowed(bridge_token_id, amount)?;

            let call = MaterializeCall {
                bridge_token_id,
                to,
                amount,
            };
            let now = <frame_system::Pallet<T>>::block_number();
            let mut proposal = match Self::votes(src_id, (src_nonce, call.clone())) {
                Some(p) => p,
                None => {
                    Self::deposit_event(Event::<T>::MaterializeInitialized(
                        who.clone(),
                        src_id,
                        src_nonce,
                        call.bridge_token_id,
                        call.clone().to,
                        call.amount,
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

            if favour {
                proposal.votes_for.push(who.clone());
                Self::deposit_event(Event::<T>::MaterializeVoteFor(
                    src_id,
                    src_nonce,
                    who,
                    call.bridge_token_id,
                    call.clone().to,
                    call.amount,
                ));
            } else {
                proposal.votes_against.push(who.clone());
                Self::deposit_event(Event::MaterializeVoteAgainst(
                    src_id,
                    src_nonce,
                    who,
                    call.bridge_token_id,
                    call.clone().to,
                    call.amount,
                ));
            }

            ProposalVotes::<T>::insert(src_id, (src_nonce, call.clone()), proposal.clone());

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

    /// Checks if the origin is relay members
    fn ensure_relay_member(origin: T::Origin) -> Result<T::AccountId, Error<T>> {
        let who = match ensure_signed_or_root(origin).map_err(|_| Error::<T>::OriginNoPermission)? {
            Some(account_id) => account_id,
            None => return Ok(T::RootOperatorAccountId::get()),
        };

        ensure!(
            T::RelayMembers::contains(&who),
            Error::<T>::OriginNoPermission
        );
        Ok(who)
    }

    /// Checks if a chain is registered
    fn chain_registered(chain_id: ChainId) -> bool {
        ChainNonces::<T>::contains_key(chain_id)
    }

    fn ensure_chain_registered(chain_id: ChainId) -> DispatchResult {
        ensure!(
            Self::chain_registered(chain_id),
            Error::<T>::ChainIdNotRegistered
        );

        Ok(())
    }

    fn ensure_chain_nonce_valid(chain_id: ChainId, chain_nonce: ChainNonce) -> DispatchResult {
        ensure!(
            !Self::has_bridged(chain_id, chain_nonce),
            Error::<T>::ProposalAlreadyComplete
        );

        Ok(())
    }

    fn ensure_amount_valid(amount: BalanceOf<T>) -> DispatchResult {
        ensure!(amount > 0, Error::<T>::BridgingAmountTooLow);

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

    /// Get the count of members in the `RelayMembers`.
    pub fn get_members_count() -> u32 {
        T::RelayMembers::count() as u32
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

    /// Make sure the bridging amount under the bridge cap
    fn ensure_under_bridge_cap(
        bridge_token: BridgeToken,
        amount: BalanceOf<T>,
        bridge_type: BridgeType,
    ) -> Result<Balance, DispatchError> {
        let new_amount = match bridge_type {
            BridgeType::BridgeOut => {
                let new_out_amount = bridge_token
                    .out_amount
                    .checked_add(amount)
                    .ok_or(ArithmeticError::Overflow)?;
                ensure!(
                    new_out_amount <= bridge_token.out_cap,
                    Error::<T>::BridgeOutCapExceeded
                );
                new_out_amount
            }
            BridgeType::BridgeIn => {
                let new_in_amount = bridge_token
                    .in_amount
                    .checked_add(amount)
                    .ok_or(ArithmeticError::Overflow)?;
                ensure!(
                    new_in_amount <= bridge_token.in_cap,
                    Error::<T>::BridgeInCapExceeded
                );
                new_in_amount
            }
        };

        Ok(new_amount)
    }

    pub fn materialize_allowed(
        bridge_token_id: CurrencyId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        Self::ensure_bridge_token_registered(bridge_token_id)?;

        let asset_id = Self::asset_id(bridge_token_id);
        let bridge_token = Self::bridge_token(asset_id);
        ensure!(bridge_token.enable, Error::<T>::BridgeTokenDisabled);
        Self::ensure_under_bridge_cap(bridge_token, amount, BridgeType::BridgeIn)?;

        Self::ensure_amount_valid(amount)?;

        Ok(())
    }

    pub fn change_vote_threshold() -> DispatchResult {
        let new_threshold =
            Ratio::from_percent(T::ThresholdPercentage::get()).mul_ceil(Self::get_members_count());
        Self::ensure_valid_threshold(new_threshold, Self::get_members_count())?;

        // Set a new vote threshold
        VoteThreshold::<T>::put(new_threshold);
        Self::deposit_event(Event::VoteThresholdUpdated(new_threshold));

        Ok(())
    }

    /// Increments the chain nonce for the specified chain_id
    fn bump_nonce(chain_id: ChainId) -> ChainNonce {
        let nonce = Self::chain_nonces(chain_id) + 1;
        ChainNonces::<T>::insert(chain_id, nonce);

        nonce
    }

    fn merge_overlapping_intervals(mut registry: Vec<BridgeInterval>) -> Vec<BridgeInterval> {
        registry.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        let mut merged: Vec<BridgeInterval> = vec![];
        for r in registry {
            match merged.last_mut() {
                None => merged.push(r),
                Some(last_merged) => {
                    if r.0 > last_merged.1 {
                        merged.push(r);
                    } else {
                        (*last_merged).1 = r.1.max(last_merged.1);
                    }
                }
            }
        }
        merged
    }

    pub fn has_bridged(chain_id: ChainId, chain_nonce: ChainNonce) -> bool {
        Self::bridge_registry(&chain_id).map_or(false, |registry| {
            registry
                .iter()
                .any(|&r| (chain_nonce >= r.0 && chain_nonce <= r.1))
        })
    }

    /// Records completed bridge transactions
    #[require_transactional]
    fn update_bridge_registry(chain_id: ChainId, nonce: ChainNonce) {
        match Self::bridge_registry(&chain_id) {
            None => {}
            Some(mut registry) => {
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
                BridgeRegistry::<T>::insert(chain_id, registry);
            }
        }
    }

    /// Update bridge token amount
    #[require_transactional]
    fn update_bridge_token_cap(
        asset_id: AssetIdOf<T>,
        amount: BalanceOf<T>,
        bridge_type: BridgeType,
    ) -> DispatchResult {
        let mut bridge_token = Self::bridge_token(asset_id);
        let new_amount =
            Self::ensure_under_bridge_cap(bridge_token.clone(), amount, bridge_type.clone())?;
        match bridge_type {
            BridgeType::BridgeOut => {
                bridge_token.out_amount = new_amount;
            }
            BridgeType::BridgeIn => {
                bridge_token.in_amount = new_amount;
            }
        };
        BridgeTokens::<T>::insert(asset_id, bridge_token);

        Ok(())
    }

    #[require_transactional]
    fn try_mutate_bridge_token<F>(bridge_token_id: CurrencyId, op: F) -> DispatchResult
    where
        F: FnOnce(&mut BridgeToken) -> DispatchResult,
    {
        Self::ensure_bridge_token_registered(bridge_token_id)?;

        let asset_id = Self::asset_id(bridge_token_id);
        let mut bridge_token = Self::bridge_token(asset_id);
        op(&mut bridge_token)?;
        BridgeTokens::<T>::insert(asset_id, bridge_token);

        Ok(())
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

    /// Attempts to finalize or cancel the proposal if the vote count allows.
    #[require_transactional]
    fn resolve_proposal(
        src_id: ChainId,
        src_nonce: ChainNonce,
        call: MaterializeCallOf<T>,
    ) -> DispatchResult {
        if let Some(mut proposal) = Self::votes(src_id, (src_nonce, call.clone())) {
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

    #[require_transactional]
    fn execute_materialize(
        src_id: ChainId,
        src_nonce: ChainNonce,
        call: MaterializeCallOf<T>,
    ) -> DispatchResult {
        Self::ensure_chain_registered(src_id)?;
        Self::ensure_bridge_token_registered(call.bridge_token_id)?;

        let asset_id = Self::asset_id(call.bridge_token_id);
        Self::update_bridge_token_cap(asset_id, call.amount, BridgeType::BridgeIn)?;
        Self::deposit_event(Event::ProposalApproved(src_id, src_nonce));

        let BridgeToken { external, .. } = Self::bridge_token(asset_id);
        if external {
            T::Assets::mint_into(asset_id, &call.to, call.amount)?;
        } else {
            T::Assets::transfer(asset_id, &Self::account_id(), &call.to, call.amount, true)?;
        }

        Self::grant_incentive_bonus(call.clone().to, asset_id, call.amount)?;
        Self::update_bridge_registry(src_id, src_nonce);

        log::trace!(
            target: "bridge::execute_materialize",
            "src_id: {:?}, nonce {:?}, bridge_token_id: {:?}, to {:?}, amount: {:?}",
            src_id,
            src_nonce,
            call.bridge_token_id,
            call.to,
            call.amount,
        );

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
    #[require_transactional]
    fn cancel_materialize(src_id: ChainId, src_nonce: ChainNonce) -> DispatchResult {
        Self::update_bridge_registry(src_id, src_nonce);
        Self::deposit_event(Event::ProposalRejected(src_id, src_nonce));

        Ok(())
    }

    /// Reward some native tokens to users who don't have enough balance
    #[require_transactional]
    fn grant_incentive_bonus(
        who: T::AccountId,
        asset_id: CurrencyId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        let gift_account = T::GiftAccount::get();
        let native_currency_id = T::NativeCurrencyId::get();
        let gift_amount =
            T::GiftConvert::to_asset_balance(amount, asset_id).unwrap_or_else(|_| Zero::zero());
        let beneficiary_native_balance =
            T::Assets::reducible_balance(native_currency_id, &who, true);
        let reducible_balance =
            T::Assets::reducible_balance(native_currency_id, &gift_account, false);

        if !gift_amount.is_zero()
            && reducible_balance >= gift_amount
            && beneficiary_native_balance < gift_amount
        {
            let diff = T::ExistentialDeposit::get() + gift_amount - beneficiary_native_balance;
            T::Assets::transfer(native_currency_id, &gift_account, &who, diff, false)?;
        }

        Ok(())
    }
}
