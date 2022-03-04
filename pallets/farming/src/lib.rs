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

mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod types;
pub mod weights;

use frame_support::{
    pallet_prelude::*,
    traits::{
        fungibles::{Inspect, Mutate, Transfer},
        Get, IsType,
    },
    transactional, Blake2_128Concat, PalletId,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use primitives::{Balance, CurrencyId, DecimalProvider};
use sp_io::hashing::blake2_256;
use sp_runtime::{
    traits::{AccountIdConversion, CheckedAdd, CheckedSub, Saturating, StaticLookup, Zero},
    ArithmeticError,
};
use sp_std::result::Result;

use crate::types::{PoolInfo, UserPosition};
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

        /// Decimal provider.
        type Decimal: DecimalProvider<CurrencyId>;

        /// The origin which can create new pools and add reward.
        type UpdateOrigin: EnsureOrigin<Self::Origin>;

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
        /// Pool is not active
        PoolIsNotActive,
        /// Pool is already in desire status
        PoolInStatus,
        /// Not a valid duration
        NotAValidDuration,
        /// Pool is in a target lock duration status
        PoolIsInTargetLockDuration,
        /// Not a valid amount
        NotAValidAmount,
        /// Deposit Balance must be greater than or equal to the withdraw amount
        DepositBalanceLow,
        /// Codec error
        CodecError,
        /// Excess max lock duration for lock pool
        ExcessMaxLockDuration,
        /// Excess max user lock item count
        ExcessMaxUserLockItemsCount,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Add new pool
        PoolAdded(AssetIdOf<T>, AssetIdOf<T>),
        /// Pool new status was set.
        PoolStatusChanged(AssetIdOf<T>, AssetIdOf<T>, bool),
        /// Pool new lock duration was set.
        PoolLockDurationChanged(AssetIdOf<T>, AssetIdOf<T>, T::BlockNumber),
        /// Deposited Assets in pool
        AssetsDeposited(T::AccountId, AssetIdOf<T>, AssetIdOf<T>, BalanceOf<T>),
        /// Withdrew Assets from pool
        AssetsWithdrew(T::AccountId, AssetIdOf<T>, AssetIdOf<T>, BalanceOf<T>),
        /// Redeem Assets from lock pool
        AssetsRedeem(T::AccountId, AssetIdOf<T>, AssetIdOf<T>, BalanceOf<T>),
        /// Reward Paid for user
        RewardPaid(T::AccountId, AssetIdOf<T>, AssetIdOf<T>, BalanceOf<T>),
        /// Reward added
        RewardAdded(AssetIdOf<T>, AssetIdOf<T>, BalanceOf<T>),
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Each pool is associated to a stake asset and reward asset pair
    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetIdOf<T>,
        Blake2_128Concat,
        AssetIdOf<T>,
        PoolInfo<T::BlockNumber, BalanceOf<T>>,
        OptionQuery,
    >;

    /// User position in pool which is associated to a stake asset and reward asset pair
    #[pallet::storage]
    #[pallet::getter(fn positions)]
    pub type Positions<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, AssetIdOf<T>>,
            NMapKey<Blake2_128Concat, AssetIdOf<T>>,
            NMapKey<Blake2_128Concat, T::AccountId>,
        ),
        UserPosition<
            BalanceOf<T>,
            BoundedVec<(BalanceOf<T>, T::BlockNumber), T::MaxUserLockItemsCount>,
        >,
        ValueQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create new pool from a privileged origin. Pool can be identified by a pair of asset and reward_asset.
        ///
        /// The origin must conform to `UpdateOrigin`.
        ///
        /// - `asset`: The identifier of the staking asset.
        /// - `reward_asset`: The identifier of the reward asset.
        /// - `lock_duration`: Lock block number after Withdraw.
        #[pallet::weight(T::WeightInfo::create())]
        #[transactional]
        pub fn create(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            lock_duration: T::BlockNumber,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;

            ensure!(
                !Pools::<T>::contains_key(&asset, &reward_asset),
                Error::<T>::PoolAlreadyExists
            );

            ensure!(
                lock_duration <= T::LockPoolMaxDuration::get(),
                Error::<T>::ExcessMaxLockDuration
            );

            let pool = PoolInfo {
                lock_duration,
                ..Default::default()
            };

            Pools::<T>::insert(&asset, &reward_asset, pool);
            Self::deposit_event(Event::<T>::PoolAdded(asset, reward_asset));
            Ok(())
        }

        /// Set pool active status
        ///
        /// The origin must conform to `UpdateOrigin`.
        ///
        /// - `asset`: The identifier of the staking asset.
        /// - `reward_asset`: The identifier of the reward asset.
        /// - `is_active`: new active status.
        #[pallet::weight(T::WeightInfo::set_pool_status())]
        #[transactional]
        pub fn set_pool_status(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            is_active: bool,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;

            Pools::<T>::mutate(asset, reward_asset, |pool_info| -> DispatchResult {
                let pool_info = pool_info.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;

                ensure!(pool_info.is_active != is_active, Error::<T>::PoolInStatus);

                pool_info.is_active = is_active;
                Self::deposit_event(Event::<T>::PoolStatusChanged(
                    asset,
                    reward_asset,
                    is_active,
                ));
                Ok(())
            })
        }

        /// Set pool lock duration
        ///
        /// The origin must conform to `UpdateOrigin`.
        ///
        /// - `asset`: The identifier of the staking asset.
        /// - `reward_asset`: The identifier of the reward asset.
        /// - `lock_duration`: new lock block number after Withdraw.
        #[pallet::weight(T::WeightInfo::set_pool_lock_duration())]
        #[transactional]
        pub fn set_pool_lock_duration(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            lock_duration: T::BlockNumber,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;

            ensure!(
                lock_duration <= T::LockPoolMaxDuration::get(),
                Error::<T>::ExcessMaxLockDuration
            );

            Pools::<T>::mutate(asset, reward_asset, |pool_info| -> DispatchResult {
                let pool_info = pool_info.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;

                ensure!(
                    pool_info.lock_duration != lock_duration,
                    Error::<T>::PoolIsInTargetLockDuration
                );

                pool_info.lock_duration = lock_duration;
                Self::deposit_event(Event::<T>::PoolLockDurationChanged(
                    asset,
                    reward_asset,
                    lock_duration,
                ));
                Ok(())
            })
        }

        /// Depositing Assets to reward Pool
        ///
        /// The origin must be Signed and the sender must have sufficient balance of staking asset.
        ///
        /// - `asset`: The identifier of the staking asset.
        /// - `reward_asset`: The identifier of the reward asset.
        /// - `amount`: the amount of staking asset want to deposit.
        #[pallet::weight(T::WeightInfo::deposit())]
        #[transactional]
        pub fn deposit(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Pools::<T>::contains_key(&asset, &reward_asset),
                Error::<T>::PoolDoesNotExist
            );
            ensure!(!amount.is_zero(), Error::<T>::NotAValidAmount);

            Self::update_reward(Some(who.clone()), asset, reward_asset)?;

            let asset_pool_account = Self::pool_account_id(asset)?;
            Pools::<T>::mutate(asset, reward_asset, |pool_info| -> DispatchResult {
                let pool_info = pool_info.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;

                ensure!(pool_info.is_active, Error::<T>::PoolIsNotActive);

                T::Assets::transfer(asset, &who, &asset_pool_account, amount, true)?;

                pool_info.total_deposited = pool_info
                    .total_deposited
                    .checked_add(amount)
                    .ok_or(ArithmeticError::Overflow)?;

                Positions::<T>::mutate(
                    (&asset, &reward_asset, &who),
                    |user_position| -> DispatchResult {
                        user_position.deposit_balance = user_position
                            .deposit_balance
                            .checked_add(amount)
                            .ok_or(ArithmeticError::Overflow)?;
                        Ok(())
                    },
                )?;

                Self::deposit_event(Event::<T>::AssetsDeposited(
                    who,
                    asset,
                    reward_asset,
                    amount,
                ));
                Ok(())
            })
        }

        /// Withdrawing Assets from reward Pool
        ///
        /// The origin must be Signed and the sender must have sufficient deposited balance.
        ///
        /// - `asset`: The identifier of the staking asset.
        /// - `reward_asset`: The identifier of the reward asset.
        /// - `amount`: the amount of staking asset want to withdraw.
        #[pallet::weight(T::WeightInfo::withdraw())]
        #[transactional]
        pub fn withdraw(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Pools::<T>::contains_key(&asset, &reward_asset),
                Error::<T>::PoolDoesNotExist
            );
            ensure!(!amount.is_zero(), Error::<T>::NotAValidAmount);

            let user_position = Positions::<T>::get((&asset, &reward_asset, &who));
            ensure!(
                user_position.deposit_balance >= amount,
                Error::<T>::DepositBalanceLow
            );

            Self::update_reward(Some(who.clone()), asset, reward_asset)?;

            let current_block_number = <frame_system::Pallet<T>>::block_number();
            Pools::<T>::mutate(asset, reward_asset, |pool_info| -> DispatchResult {
                let pool_info = pool_info.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;
                pool_info.total_deposited = pool_info
                    .total_deposited
                    .checked_sub(amount)
                    .ok_or(ArithmeticError::Overflow)?;

                Positions::<T>::mutate(
                    (&asset, &reward_asset, &who),
                    |user_position| -> DispatchResult {
                        user_position.deposit_balance = user_position
                            .deposit_balance
                            .checked_sub(amount)
                            .ok_or(ArithmeticError::Overflow)?;

                        if pool_info.lock_duration.is_zero() {
                            let asset_pool_account = Self::pool_account_id(asset)?;
                            T::Assets::transfer(asset, &asset_pool_account, &who, amount, true)?;
                        } else {
                            user_position
                                .lock_balance_items
                                .try_push((amount, current_block_number))
                                .map_err(|_| Error::<T>::ExcessMaxUserLockItemsCount)?;
                        }

                        Ok(())
                    },
                )?;

                Self::deposit_event(Event::<T>::AssetsWithdrew(who, asset, reward_asset, amount));
                Ok(())
            })
        }

        /// Redeem unlocked balance of staking asset from Pool
        ///
        /// Origin must be Signed.
        ///
        /// - `asset`: The identifier of the staking asset.
        /// - `reward_asset`: The identifier of the reward asset.
        #[pallet::weight(T::WeightInfo::redeem())]
        #[transactional]
        pub fn redeem(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let pool_info = Pools::<T>::try_get(&asset, &reward_asset)
                .map_err(|_err| Error::<T>::PoolDoesNotExist)?;

            let current_block_number = <frame_system::Pallet<T>>::block_number();
            Positions::<T>::mutate(
                (&asset, &reward_asset, &who),
                |user_position| -> DispatchResult {
                    let mut total_amount: BalanceOf<T> = 0;
                    for item in user_position.lock_balance_items.iter() {
                        let unlock_block = item.1.saturating_add(pool_info.lock_duration);
                        if current_block_number >= unlock_block {
                            total_amount = total_amount
                                .checked_add(item.0)
                                .ok_or(ArithmeticError::Overflow)?;
                        }
                    }

                    user_position.lock_balance_items.retain(|item| {
                        let unlock_block = item.1.saturating_add(pool_info.lock_duration);
                        current_block_number < unlock_block
                    });

                    if total_amount > 0 {
                        let asset_pool_account = Self::pool_account_id(asset)?;
                        T::Assets::transfer(asset, &asset_pool_account, &who, total_amount, true)?;

                        Self::deposit_event(Event::<T>::AssetsRedeem(
                            who.clone(),
                            asset,
                            reward_asset,
                            total_amount,
                        ));
                    }

                    Ok(())
                },
            )
        }

        /// Claim reward asset from pool
        ///
        /// Origin must be Signed.
        ///
        /// - `asset`: The identifier of the staking asset.
        /// - `reward_asset`: The identifier of the reward asset.
        #[pallet::weight(T::WeightInfo::claim())]
        #[transactional]
        pub fn claim(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Pools::<T>::contains_key(&asset, &reward_asset),
                Error::<T>::PoolDoesNotExist
            );

            Self::update_reward(Some(who.clone()), asset, reward_asset)?;

            let asset_pool_account = Self::pool_account_id(reward_asset)?;
            Positions::<T>::mutate(
                (&asset, &reward_asset, &who),
                |user_position| -> DispatchResult {
                    let reward_amount = user_position.reward_amount;
                    if reward_amount > 0 {
                        T::Assets::transfer(
                            reward_asset,
                            &asset_pool_account,
                            &who,
                            reward_amount,
                            true,
                        )?;
                        user_position.reward_amount = 0;

                        Self::deposit_event(Event::<T>::RewardPaid(
                            who.clone(),
                            asset,
                            reward_asset,
                            reward_amount,
                        ));
                    }

                    Ok(())
                },
            )
        }

        /// Dispatch reward asset with specified amount and duration
        ///
        /// The origin must conform to `UpdateOrigin`.
        ///
        /// - `asset`: The identifier of the staking asset.
        /// - `reward_asset`: The identifier of the reward asset.
        /// - `payer`: the payer of reward asset.
        /// - `amount`: the amount of reward asset to dispatch.
        /// - `duration`: the number of block this reward will last for.
        #[pallet::weight(T::WeightInfo::dispatch_reward())]
        #[transactional]
        pub fn dispatch_reward(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            payer: <T::Lookup as StaticLookup>::Source,
            amount: BalanceOf<T>,
            duration: T::BlockNumber,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            ensure!(
                Pools::<T>::contains_key(&asset, &reward_asset),
                Error::<T>::PoolDoesNotExist
            );
            ensure!(!duration.is_zero(), Error::<T>::NotAValidDuration);

            Self::update_reward(None, asset, reward_asset)?;

            let current_block_number = <frame_system::Pallet<T>>::block_number();
            Pools::<T>::mutate(asset, reward_asset, |pool_info| -> DispatchResult {
                let pool_info = pool_info.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;
                let duration_balance = pool_info.block_to_balance(duration);
                let reward_rate = if current_block_number >= pool_info.period_finish {
                    amount
                        .checked_div(duration_balance)
                        .ok_or(ArithmeticError::Overflow)?
                } else {
                    let remaining = pool_info
                        .period_finish
                        .checked_sub(&current_block_number)
                        .ok_or(ArithmeticError::Overflow)?;
                    let left_over = pool_info
                        .block_to_balance(remaining)
                        .checked_mul(pool_info.reward_rate)
                        .ok_or(ArithmeticError::Overflow)?;
                    let total = left_over
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Overflow)?;
                    total
                        .checked_div(duration_balance)
                        .ok_or(ArithmeticError::Overflow)?
                };

                let new_period_finish = current_block_number
                    .checked_add(&duration)
                    .ok_or(ArithmeticError::Overflow)?;

                pool_info.duration = duration;
                pool_info.period_finish = new_period_finish;
                pool_info.reward_rate = reward_rate;
                pool_info.last_update_block = current_block_number;

                if amount > 0 {
                    let asset_pool_account = Self::pool_account_id(reward_asset)?;
                    let payer = T::Lookup::lookup(payer)?;
                    T::Assets::transfer(reward_asset, &payer, &asset_pool_account, amount, true)?;
                }

                Self::deposit_event(Event::<T>::RewardAdded(asset, reward_asset, amount));
                Ok(())
            })
        }
    }
}

impl<T: Config> Pallet<T> {
    fn update_reward(
        who: Option<T::AccountId>,
        asset: AssetIdOf<T>,
        reward_asset: AssetIdOf<T>,
    ) -> DispatchResult {
        let current_block_number = <frame_system::Pallet<T>>::block_number();

        //1, update pool reward info
        Pools::<T>::mutate(asset, reward_asset, |pool_info| -> DispatchResult {
            let pool_info = pool_info.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;

            pool_info.update_reward_per_share(current_block_number)?;

            //2, update user reward info
            if let Some(who) = who {
                Positions::<T>::mutate(
                    (&asset, &reward_asset, &who),
                    |user_position| -> DispatchResult {
                        let diff = pool_info
                            .reward_per_share(current_block_number)?
                            .checked_sub(user_position.reward_per_share_paid)
                            .ok_or(ArithmeticError::Overflow)?;

                        let earned = user_position
                            .deposit_balance
                            .checked_mul(diff)
                            .and_then(|r| r.checked_div(pool_info.amount_per_share()))
                            .and_then(|r| r.checked_add(user_position.reward_amount))
                            .ok_or(ArithmeticError::Overflow)?;

                        user_position.reward_amount = earned;
                        user_position.reward_per_share_paid = pool_info.reward_per_share_stored;

                        Ok(())
                    },
                )?
            }
            Ok(())
        })
    }

    fn pool_account_id(asset_id: AssetIdOf<T>) -> Result<T::AccountId, DispatchError> {
        let account_id: T::AccountId = T::PalletId::get().into_account();
        let entropy = (b"modlpy/liquidity", &[account_id], asset_id).using_encoded(blake2_256);
        Ok(T::AccountId::decode(&mut &entropy[..]).map_err(|_| Error::<T>::CodecError)?)
    }
}
