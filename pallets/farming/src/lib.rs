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

//! # Pure Farming (FAR)
//!
//! pallet-farming is in charge of creating a governance-controlled incentivization program for our different products.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod benchmarking;

pub mod weights;

use frame_support::traits::tokens::Balance as TokenBalance;
use frame_support::{
    pallet_prelude::*,
    traits::{
        fungibles::{Inspect, Mutate, Transfer},
        Get, IsType,
    },
    transactional, Blake2_128Concat, PalletId,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use primitives::{Balance, CurrencyId};
use scale_info::TypeInfo;
use sp_io::hashing::blake2_256;
use sp_runtime::{
    traits::{AccountIdConversion, Saturating, UniqueSaturatedInto, Zero},
    ArithmeticError, SaturatedConversion,
};
use sp_std::{convert::TryInto, result::Result, vec::Vec};

pub use pallet::*;
pub use weights::WeightInfo;

type AssetIdOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

type BalanceOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Currency type for deposit/withdraw assets to/from plm
        /// module
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        /// Defines the pallet's pallet id from which we can define each pool's account id
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// The origin which can create new pools.
        type CreateOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can dispatch reward rules.
        type RewardOrigin: EnsureOrigin<Self::Origin>;

        /// Specifies how many reward tokens can be manipulated by a pool
        #[pallet::constant]
        type MaxRewardTokens: Get<u32>;

        /// Specifies max amount lock item for a user
        #[pallet::constant]
        type MaxUserLockItemsCount: Get<u32>;

        /// Specifies upper limit of lock duration for lock pool
        #[pallet::constant]
        type LockPoolMaxDuration: Get<Self::BlockNumber>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Pool does not exist
        PoolDoesNotExist,
        /// Pool associacted with asset already exists
        PoolAlreadyExists,
        /// Not a newly created asset
        NotANewlyCreatedAsset,
        /// Not a newly created lock pool asset
        NotANewlyCreatedLockPoolAsset,
        /// Not a valid duration
        NotAValidDuration,
        /// Not a valid amount
        NotAValidAmount,
        /// The end block is smaller than start block
        SmallerThanEndBlock,
        /// Reward token does not exist for specified asset.
        RewardTokenDoesNotExistForSpecifiedAsset,
        /// Reward rule does not exist for specified asset.
        RewardRuleDoesNotExistForSpecifiedAsset,
        /// Pool reward rule info does not exist
        PoolRewardRuleDoesNotExist,
        /// User reward info does not exist
        UserRewardDoesNotExist,
        /// User lock info does not exist
        UserLockInfoDoesNotExist,
        /// Codec error
        CodecError,
        /// Excess max lock duration for lock pool
        ExcessMaxLockDuration,
        /// Excess max user lock item count
        ExcessMaxUserLockItemsCount,
        /// start or end block number for reward rule is wrong
        RewardDurationError,
        /// old reward rule is still valid
        RewardRuleStillValid,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Add new pool
        /// [asset_id]
        PoolAdded(AssetIdOf<T>),
        /// Deposited Assets in pool
        /// [sender, asset_id]
        AssetsDeposited(T::AccountId, AssetIdOf<T>),
        /// Withdrew Assets from pool
        /// [sender, asset_id]
        AssetsWithdrew(T::AccountId, AssetIdOf<T>),
        /// Reward Paid for user
        /// [sender, asset_id, amount]
        RewardPaid(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
    pub struct Pool<AssetId, BoundedTokens, BlockNumber> {
        /// Which assets we use to send rewards
        pub reward_tokens: BoundedTokens,
        /// Which asset we use to represent shares of the pool
        pub asset_id: AssetId,
        /// Which asset we use to represent shares of the lock pool
        pub lock_pool_asset_id: AssetId,
        /// lock duration for lock pool
        pub lock_duration: BlockNumber,
    }

    /// Each pool is associated to a unique AssetId (not be mixed with the reward asset)
    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        AssetIdOf<T>,
        Pool<AssetIdOf<T>, BoundedVec<AssetIdOf<T>, T::MaxRewardTokens>, T::BlockNumber>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn user_lock_info)]
    pub type UserLockItems<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        AssetIdOf<T>,
        BoundedVec<(BalanceOf<T>, T::BlockNumber), T::MaxUserLockItemsCount>,
        OptionQuery,
    >;

    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
    pub struct PoolRewardInfo<BlockNumber, BalanceOf> {
        /// When the liquidity program starts
        pub start: BlockNumber,
        /// When the liquidity program stops
        pub end: BlockNumber,
        pub last_update_block: BlockNumber,
        pub reward_rate: BalanceOf,
        pub reward_per_share_stored: BalanceOf,
    }

    impl<
            BlockNumber: Copy + PartialOrd + Saturating + UniqueSaturatedInto<u128>,
            BalanceOf: TokenBalance,
        > PoolRewardInfo<BlockNumber, BalanceOf>
    {
        pub fn last_reward_block_applicable(
            &self,
            current_block_number: BlockNumber,
        ) -> BlockNumber {
            if current_block_number > self.end {
                self.end
            } else {
                current_block_number
            }
        }

        pub fn valid_last_update_block(&self) -> BlockNumber {
            if self.last_update_block > self.start {
                self.last_update_block
            } else {
                self.start
            }
        }

        pub fn reward_per_share(
            &self,
            total_issue: BalanceOf,
            current_block_number: BlockNumber,
        ) -> Result<BalanceOf, ArithmeticError> {
            return if total_issue.is_zero() || self.start >= current_block_number {
                Ok(self.reward_per_share_stored)
            } else {
                let last_reward_block = self.last_reward_block_applicable(current_block_number);
                let block_diff = Self::block_to_balance(
                    last_reward_block.saturating_sub(self.valid_last_update_block()),
                );
                let reward_per_share_add = block_diff
                    .checked_mul(&self.reward_rate)
                    .ok_or(ArithmeticError::Overflow)?
                    .checked_div(&total_issue)
                    .ok_or(ArithmeticError::Overflow)?;

                let ret = self
                    .reward_per_share_stored
                    .checked_add(&reward_per_share_add)
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(ret)
            };
        }

        pub fn update_reward_per_share(
            &mut self,
            total_issue: BalanceOf,
            current_block_number: BlockNumber,
        ) -> Result<(), ArithmeticError> {
            let reward_per_share_stored =
                self.reward_per_share(total_issue, current_block_number)?;
            if reward_per_share_stored > self.reward_per_share_stored {
                self.reward_per_share_stored = reward_per_share_stored;
            }

            let last_reward_block_applicable =
                self.last_reward_block_applicable(current_block_number);
            if last_reward_block_applicable > self.last_update_block {
                self.last_update_block = last_reward_block_applicable;
            }

            Ok(())
        }

        fn block_to_balance(duration: BlockNumber) -> BalanceOf {
            BalanceOf::saturated_from(duration.saturated_into())
        }
    }

    /// Each pool is associated to a unique AssetId (not be mixed with the reward asset)
    #[pallet::storage]
    #[pallet::getter(fn pool_reward_info)]
    pub type PoolsRewards<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetIdOf<T>,
        Blake2_128Concat,
        AssetIdOf<T>,
        PoolRewardInfo<T::BlockNumber, BalanceOf<T>>,
        OptionQuery,
    >;

    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
    pub struct RewardInfo<BalanceOf> {
        pub reward_amount: BalanceOf,
        pub reward_per_share_paid: BalanceOf,
    }

    /// Each pool is associated to a unique AssetId (not be mixed with the reward asset)
    #[pallet::storage]
    #[pallet::getter(fn user_reward_info)]
    pub type UserRewardInfo<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>,
            NMapKey<Blake2_128Concat, AssetIdOf<T>>,
            NMapKey<Blake2_128Concat, AssetIdOf<T>>,
        ),
        RewardInfo<BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create new pool, associated with a unique asset id
        #[pallet::weight(T::WeightInfo::create())]
        #[transactional]
        pub fn create(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_tokens: BoundedVec<AssetIdOf<T>, T::MaxRewardTokens>,
            asset_id: AssetIdOf<T>,
            lock_pool_asset_id: AssetIdOf<T>,
            lock_duration: T::BlockNumber,
        ) -> DispatchResultWithPostInfo {
            T::CreateOrigin::ensure_origin(origin)?;

            ensure!(
                !Pools::<T>::contains_key(&asset),
                Error::<T>::PoolAlreadyExists
            );

            ensure!(
                T::Assets::total_issuance(asset_id).is_zero(),
                Error::<T>::NotANewlyCreatedAsset
            );

            ensure!(
                T::Assets::total_issuance(lock_pool_asset_id).is_zero(),
                Error::<T>::NotANewlyCreatedLockPoolAsset
            );

            ensure!(
                lock_duration <= T::LockPoolMaxDuration::get(),
                Error::<T>::ExcessMaxLockDuration
            );

            let pool = Pool {
                reward_tokens,
                asset_id,
                lock_pool_asset_id,
                lock_duration,
            };

            Pools::<T>::insert(&asset, pool);
            Self::deposit_event(Event::<T>::PoolAdded(asset));
            Ok(().into())
        }

        /// Depositing Assets in a Pool
        #[pallet::weight(T::WeightInfo::deposit())]
        #[transactional]
        pub fn deposit(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(amount != Zero::zero(), Error::<T>::NotAValidAmount);

            let asset_pool_account = Self::pool_account_id(asset)?;
            Pools::<T>::try_mutate(asset, |liquidity_pool| -> DispatchResult {
                let pool = liquidity_pool
                    .as_mut()
                    .ok_or(Error::<T>::PoolDoesNotExist)?;

                Self::update_all_reward_for_user(who.clone(), asset)?;

                T::Assets::transfer(asset, &who, &asset_pool_account, amount, true)?;

                T::Assets::mint_into(pool.asset_id, &who, amount)?;

                Self::deposit_event(Event::<T>::AssetsDeposited(who, asset));
                Ok(())
            })
        }

        /// Claiming Rewards or Withdrawing Assets from a Pool
        #[pallet::weight(T::WeightInfo::withdraw())]
        #[transactional]
        pub fn withdraw(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(amount != Zero::zero(), Error::<T>::NotAValidAmount);

            let current_block_number = <frame_system::Pallet<T>>::block_number();
            let asset_pool_account = Self::pool_account_id(asset)?;
            Pools::<T>::try_mutate(asset, |liquidity_pool| -> DispatchResultWithPostInfo {
                let pool = liquidity_pool
                    .as_mut()
                    .ok_or(Error::<T>::PoolDoesNotExist)?;

                Self::update_all_reward_for_user(who.clone(), asset)?;

                if pool.lock_duration.is_zero() {
                    T::Assets::transfer(asset, &asset_pool_account, &who, amount, true)?;
                    T::Assets::burn_from(pool.asset_id, &who, amount)?;
                } else {
                    T::Assets::mint_into(pool.lock_pool_asset_id, &who, amount)?;
                    T::Assets::burn_from(pool.asset_id, &who, amount)?;

                    let unlock_block = current_block_number + pool.lock_duration;
                    if !UserLockItems::<T>::contains_key(&who, asset) {
                        UserLockItems::<T>::try_mutate(
                            who.clone(),
                            asset,
                            |user_lock_items| -> DispatchResultWithPostInfo {
                                let user_lock_items = user_lock_items
                                    .as_mut()
                                    .ok_or(Error::<T>::UserLockInfoDoesNotExist)?;
                                user_lock_items
                                    .try_push((amount, unlock_block))
                                    .map_err(|_| Error::<T>::ExcessMaxUserLockItemsCount)?;
                                Ok(().into())
                            },
                        )?;
                    } else {
                        let mut user_lock_items: Vec<(BalanceOf<T>, T::BlockNumber)> = Vec::new();
                        user_lock_items.push((amount, unlock_block));
                        let user_lock_items: BoundedVec<
                            (BalanceOf<T>, T::BlockNumber),
                            T::MaxUserLockItemsCount,
                        > = user_lock_items
                            .try_into()
                            .map_err(|_| Error::<T>::ExcessMaxUserLockItemsCount)?;
                        UserLockItems::<T>::insert(&who, asset, user_lock_items);
                    }
                }

                Self::deposit_event(Event::<T>::AssetsWithdrew(who, asset));
                Ok(().into())
            })
        }

        /// Withdrawing Assets from a lock Pool
        #[pallet::weight(T::WeightInfo::withdraw_from_lock_pool())]
        #[transactional]
        pub fn withdraw_from_lock_pool(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let pool = Pools::<T>::try_get(asset).map_err(|_err| Error::<T>::PoolDoesNotExist)?;

            let current_block_number = <frame_system::Pallet<T>>::block_number();
            let asset_pool_account = Self::pool_account_id(asset)?;
            UserLockItems::<T>::try_mutate(
                who.clone(),
                asset,
                |user_lock_items| -> DispatchResultWithPostInfo {
                    let user_lock_items = user_lock_items
                        .as_mut()
                        .ok_or(Error::<T>::UserLockInfoDoesNotExist)?;

                    let mut total_amount: BalanceOf<T> = 0;
                    user_lock_items.iter().for_each(|item| {
                        if current_block_number >= item.1 {
                            total_amount = total_amount + item.0;
                        }
                    });

                    user_lock_items.retain(|item| {
                        if current_block_number < item.1 {
                            true
                        } else {
                            false
                        }
                    });

                    T::Assets::burn_from(pool.lock_pool_asset_id, &who, total_amount)?;
                    T::Assets::transfer(asset, &asset_pool_account, &who, total_amount, true)?;
                    Ok(().into())
                },
            )?;

            Ok(().into())
        }

        /// get all reward token from pool
        #[pallet::weight(T::WeightInfo::get_all_reward())]
        #[transactional]
        pub fn get_all_reward(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::update_all_reward_for_user(who.clone(), asset)?;
            Self::transfer_all_reward_for_user(who, asset)?;

            Ok(().into())
        }

        /// get specified reward token from pool
        #[pallet::weight(T::WeightInfo::get_reward())]
        #[transactional]
        pub fn get_reward(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::update_reward_for_user(Some(who.clone()), asset, reward_asset)?;
            Self::transfer_reward_for_user(who, asset, reward_asset)?;

            Ok(().into())
        }

        /// dispatch reward token with specified amount and duration
        #[pallet::weight(T::WeightInfo::dispatch_reward())]
        #[transactional]
        pub fn dispatch_reward(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            start: T::BlockNumber,
            end: T::BlockNumber,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::RewardOrigin::ensure_origin(origin)?;

            let current_block_number = <frame_system::Pallet<T>>::block_number();
            ensure!(end > start, Error::<T>::RewardDurationError);
            ensure!(
                start > current_block_number,
                Error::<T>::RewardDurationError
            );
            ensure!(amount != Zero::zero(), Error::<T>::NotAValidAmount);
            let pool = Pools::<T>::try_get(asset).map_err(|_err| Error::<T>::PoolDoesNotExist)?;
            ensure!(
                pool.reward_tokens.contains(&reward_asset),
                Error::<T>::RewardTokenDoesNotExistForSpecifiedAsset
            );

            let duration = end.saturating_sub(start);
            let new_rate = amount
                .checked_div(duration.saturated_into())
                .ok_or(ArithmeticError::Overflow)?;
            if PoolsRewards::<T>::contains_key(asset, reward_asset) {
                PoolsRewards::<T>::try_mutate(
                    asset,
                    reward_asset,
                    |reward_rule| -> DispatchResultWithPostInfo {
                        let reward_rule = reward_rule
                            .as_mut()
                            .ok_or(Error::<T>::RewardRuleDoesNotExistForSpecifiedAsset)?;
                        ensure!(
                            current_block_number > reward_rule.end,
                            Error::<T>::RewardRuleStillValid
                        );

                        Self::update_reward_for_user(None, asset, reward_asset)?;

                        reward_rule.reward_rate = new_rate;
                        reward_rule.start = start;
                        reward_rule.end = end;

                        Ok(().into())
                    },
                )?;
            } else {
                let reward_rule = PoolRewardInfo {
                    start,
                    end,
                    last_update_block: current_block_number,
                    reward_rate: new_rate,
                    reward_per_share_stored: 0,
                };

                PoolsRewards::<T>::insert(asset, reward_asset, reward_rule);
            }

            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn reward_earned_internal(
        balance: BalanceOf<T>,
        total_issue: BalanceOf<T>,
        pool_reward_info: &PoolRewardInfo<T::BlockNumber, BalanceOf<T>>,
        user_reward_info: &RewardInfo<BalanceOf<T>>,
    ) -> Result<BalanceOf<T>, ArithmeticError> {
        let current_block_number = <frame_system::Pallet<T>>::block_number();

        let diff = pool_reward_info
            .reward_per_share(total_issue, current_block_number)?
            .checked_sub(user_reward_info.reward_per_share_paid)
            .ok_or(ArithmeticError::Overflow)?;

        let ret = balance
            .checked_mul(diff)
            .ok_or(ArithmeticError::Overflow)?
            .checked_add(user_reward_info.reward_amount)
            .ok_or(ArithmeticError::Overflow)?;

        return Ok(ret);
    }

    // pub fn reward_earned(who: T::AccountId, asset: AssetIdOf<T>, reward_asset:AssetIdOf<T>) -> DispatchResult {
    //     let user_balance = T::Assets::balance(who);
    //
    // }

    fn update_reward_for_user(
        who: Option<T::AccountId>,
        asset: AssetIdOf<T>,
        reward_asset: AssetIdOf<T>,
    ) -> Result<(), ArithmeticError> {
        let current_block_number = <frame_system::Pallet<T>>::block_number();
        let total_issue = T::Assets::total_issuance(asset);

        //1, update pool reward info
        PoolsRewards::<T>::try_mutate(
            asset,
            reward_asset,
            |pool_reward_info| -> Result<(), ArithmeticError> {
                if let Some(pool_reward_info) = pool_reward_info {
                    pool_reward_info.update_reward_per_share(total_issue, current_block_number)?;

                    //2, update user reward info
                    if let Some(who) = who {
                        let user_balance = T::Assets::balance(asset, &who);
                        UserRewardInfo::<T>::try_mutate(
                            (who, asset, reward_asset),
                            |user_reward_info| -> Result<(), ArithmeticError> {
                                if let Some(user_reward_info) = user_reward_info {
                                    let earned = Self::reward_earned_internal(
                                        user_balance,
                                        total_issue,
                                        pool_reward_info,
                                        user_reward_info,
                                    )?;
                                    user_reward_info.reward_amount = earned;
                                    user_reward_info.reward_per_share_paid =
                                        pool_reward_info.reward_per_share_stored;
                                }
                                Ok(())
                            },
                        )?;
                    }
                }
                Ok(())
            },
        )?;
        Ok(())
    }

    fn update_all_reward_for_user(who: T::AccountId, asset: AssetIdOf<T>) -> DispatchResult {
        let pool = Pools::<T>::try_get(asset).map_err(|_err| Error::<T>::PoolDoesNotExist)?;

        for reward_token in pool.reward_tokens.clone() {
            Self::update_reward_for_user(Some(who.clone()), asset, reward_token.clone())?;
        }

        Ok(())
    }

    fn transfer_reward_for_user(
        who: T::AccountId,
        asset: AssetIdOf<T>,
        reward_asset: AssetIdOf<T>,
    ) -> DispatchResult {
        let asset_pool_account = Self::pool_account_id(asset)?;

        UserRewardInfo::<T>::try_mutate(
            (who.clone(), asset, reward_asset),
            |user_reward_info| -> DispatchResult {
                let reward_info = user_reward_info
                    .as_mut()
                    .ok_or(Error::<T>::PoolDoesNotExist)?;

                let reward_amount = reward_info.reward_amount;
                if reward_amount > 0 {
                    T::Assets::transfer(
                        reward_asset,
                        &asset_pool_account,
                        &who,
                        reward_amount,
                        true,
                    )?;
                    reward_info.reward_amount = 0;

                    Self::deposit_event(Event::<T>::RewardPaid(who, reward_asset, reward_amount));
                }

                Ok(())
            },
        )?;

        Ok(())
    }

    fn transfer_all_reward_for_user(who: T::AccountId, asset: AssetIdOf<T>) -> DispatchResult {
        let pool = Pools::<T>::try_get(asset).map_err(|_err| Error::<T>::PoolDoesNotExist)?;

        for reward_token in pool.reward_tokens.clone() {
            Self::transfer_reward_for_user(who.clone(), asset, reward_token.clone())?;
        }

        Ok(())
    }

    pub fn pool_account_id(asset_id: AssetIdOf<T>) -> Result<T::AccountId, DispatchError> {
        let account_id: T::AccountId = T::PalletId::get().into_account();
        let entropy = (b"modlpy/liquidity", &[account_id], asset_id).using_encoded(blake2_256);
        Ok(T::AccountId::decode(&mut &entropy[..]).map_err(|_| Error::<T>::CodecError)?)
    }
}
