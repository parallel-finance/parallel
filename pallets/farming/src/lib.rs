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
use num_traits::{cast::ToPrimitive, CheckedDiv, CheckedMul};
use pallet_traits::{ConvertToBigUint, DecimalProvider};
use primitives::{Balance, CurrencyId};
use sp_io::hashing::blake2_256;
use sp_runtime::{
    traits::{
        AccountIdConversion, CheckedAdd, CheckedSub, SaturatedConversion, Saturating, StaticLookup,
        Zero,
    },
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
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

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
        type UpdateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Specifies max amount lock item for a user
        #[pallet::constant]
        type MaxUserLockItemsCount: Get<u32>;

        /// Specifies upper limit of lock duration for lock pool
        #[pallet::constant]
        type LockPoolMaxDuration: Get<Self::BlockNumber>;

        /// Specifies upper limit of cool down duration for pool
        #[pallet::constant]
        type CoolDownMaxDuration: Get<Self::BlockNumber>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Pool does not exist
        PoolDoesNotExist,
        /// Pool associated with asset already exists
        PoolAlreadyExists,
        /// Pool is not active
        PoolIsNotActive,
        /// Pool is already in desire status
        PoolInStatus,
        /// Not a valid duration
        NotAValidDuration,
        /// Pool is in a target cool down duration status
        PoolIsInTargetCoolDownDuration,
        /// Not a valid amount
        NotAValidAmount,
        /// Pool is in lock status, withdraw is not allowed.
        PoolUnderLock,
        /// Deposit Balance must be greater than or equal to the withdraw amount
        DepositBalanceLow,
        /// Codec error
        CodecError,
        /// Excess max lock duration for lock pool
        ExcessMaxLockDuration,
        /// Excess max cool down duration for pool
        ExcessMaxCoolDownDuration,
        /// Excess max user lock item count
        ExcessMaxUserLockItemsCount,
        /// Last reward is not finish
        RewardNotFinish,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Add new pool
        PoolAdded(AssetIdOf<T>, AssetIdOf<T>, T::BlockNumber),
        /// Pool new status was set.
        PoolStatusChanged(AssetIdOf<T>, AssetIdOf<T>, T::BlockNumber, bool),
        /// Pool new cool down duration was set.
        PoolCoolDownDurationChanged(AssetIdOf<T>, AssetIdOf<T>, T::BlockNumber, T::BlockNumber),
        /// Pool unlock height was reset.
        PoolUnlockHeightReset(AssetIdOf<T>, AssetIdOf<T>, T::BlockNumber, T::BlockNumber),
        /// Deposited Assets in pool
        AssetsDeposited(
            T::AccountId,
            AssetIdOf<T>,
            AssetIdOf<T>,
            T::BlockNumber,
            BalanceOf<T>,
        ),
        /// Withdrew Assets from pool
        AssetsWithdrew(
            T::AccountId,
            AssetIdOf<T>,
            AssetIdOf<T>,
            T::BlockNumber,
            BalanceOf<T>,
        ),
        /// Redeem Assets from lock pool
        AssetsRedeem(
            T::AccountId,
            AssetIdOf<T>,
            AssetIdOf<T>,
            T::BlockNumber,
            BalanceOf<T>,
        ),
        /// Reward Paid for user
        RewardPaid(
            T::AccountId,
            AssetIdOf<T>,
            AssetIdOf<T>,
            T::BlockNumber,
            BalanceOf<T>,
        ),
        /// Reward added
        RewardAdded(AssetIdOf<T>, AssetIdOf<T>, T::BlockNumber, BalanceOf<T>),
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Each pool is associated to a stake asset and reward asset pair
    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, AssetIdOf<T>>,
            NMapKey<Blake2_128Concat, AssetIdOf<T>>,
            NMapKey<Blake2_128Concat, T::BlockNumber>,
        ),
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
            NMapKey<Blake2_128Concat, T::BlockNumber>,
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
        /// - `lock_duration`: Lock block number after Deposit.
        /// - `cool_down_duration`: Lock block number after Withdraw.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create())]
        #[transactional]
        pub fn create(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            lock_duration: T::BlockNumber,
            cool_down_duration: T::BlockNumber,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;

            ensure!(
                !Pools::<T>::contains_key((&asset, &reward_asset, &lock_duration)),
                Error::<T>::PoolAlreadyExists
            );

            ensure!(
                lock_duration <= T::LockPoolMaxDuration::get(),
                Error::<T>::ExcessMaxLockDuration
            );

            ensure!(
                cool_down_duration <= T::CoolDownMaxDuration::get(),
                Error::<T>::ExcessMaxCoolDownDuration
            );

            let pool = PoolInfo {
                cool_down_duration,
                ..Default::default()
            };

            Pools::<T>::insert((&asset, &reward_asset, &lock_duration), pool);
            Self::deposit_event(Event::<T>::PoolAdded(asset, reward_asset, lock_duration));
            Ok(())
        }

        /// Set pool active status
        ///
        /// The origin must conform to `UpdateOrigin`.
        ///
        /// - `asset`: The identifier of the staking asset.
        /// - `reward_asset`: The identifier of the reward asset.
        /// - `lock_duration`: Lock block number after Deposit.
        /// - `is_active`: new active status.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::set_pool_status())]
        #[transactional]
        pub fn set_pool_status(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            lock_duration: T::BlockNumber,
            is_active: bool,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;

            Pools::<T>::mutate(
                (asset, reward_asset, lock_duration),
                |pool_info| -> DispatchResult {
                    let pool_info = pool_info.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;

                    ensure!(pool_info.is_active != is_active, Error::<T>::PoolInStatus);

                    pool_info.is_active = is_active;
                    Self::deposit_event(Event::<T>::PoolStatusChanged(
                        asset,
                        reward_asset,
                        lock_duration,
                        is_active,
                    ));
                    Ok(())
                },
            )
        }

        /// Set pool cool down duration
        ///
        /// The origin must conform to `UpdateOrigin`.
        ///
        /// - `asset`: The identifier of the staking asset.
        /// - `reward_asset`: The identifier of the reward asset.
        /// - `lock_duration`: Lock block number after Deposit.
        /// - `cool_down_duration`: new lock block number after Withdraw.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::set_pool_cool_down_duration())]
        #[transactional]
        pub fn set_pool_cool_down_duration(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            lock_duration: T::BlockNumber,
            cool_down_duration: T::BlockNumber,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;

            ensure!(
                cool_down_duration <= T::CoolDownMaxDuration::get(),
                Error::<T>::ExcessMaxCoolDownDuration
            );

            Pools::<T>::mutate(
                (asset, reward_asset, lock_duration),
                |pool_info| -> DispatchResult {
                    let pool_info = pool_info.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;

                    ensure!(
                        pool_info.cool_down_duration != cool_down_duration,
                        Error::<T>::PoolIsInTargetCoolDownDuration
                    );

                    pool_info.cool_down_duration = cool_down_duration;
                    Self::deposit_event(Event::<T>::PoolCoolDownDurationChanged(
                        asset,
                        reward_asset,
                        lock_duration,
                        cool_down_duration,
                    ));
                    Ok(())
                },
            )
        }

        /// Reset pool unlock height
        ///
        /// The origin must conform to `UpdateOrigin`.
        ///
        /// - `asset`: The identifier of the staking asset.
        /// - `reward_asset`: The identifier of the reward asset.
        /// - `lock_duration`: Lock block number after Deposit.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::reset_pool_unlock_height())]
        #[transactional]
        pub fn reset_pool_unlock_height(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            lock_duration: T::BlockNumber,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;

            Pools::<T>::mutate(
                (asset, reward_asset, lock_duration),
                |pool_info| -> DispatchResult {
                    let pool_info = pool_info.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;

                    let current_block_number = <frame_system::Pallet<T>>::block_number();
                    ensure!(
                        pool_info.unlock_height < current_block_number,
                        Error::<T>::PoolUnderLock
                    );

                    let new_unlock_height = current_block_number + lock_duration;
                    pool_info.unlock_height = new_unlock_height;
                    Self::deposit_event(Event::<T>::PoolUnlockHeightReset(
                        asset,
                        reward_asset,
                        lock_duration,
                        new_unlock_height,
                    ));
                    Ok(())
                },
            )
        }

        /// Depositing Assets to reward Pool
        ///
        /// The origin must be Signed and the sender must have sufficient balance of staking asset.
        ///
        /// - `asset`: The identifier of the staking asset.
        /// - `reward_asset`: The identifier of the reward asset.
        /// - `lock_duration`: Lock block number after Deposit.
        /// - `amount`: the amount of staking asset want to deposit.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::deposit())]
        #[transactional]
        pub fn deposit(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            lock_duration: T::BlockNumber,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Pools::<T>::contains_key((&asset, &reward_asset, &lock_duration)),
                Error::<T>::PoolDoesNotExist
            );
            ensure!(!amount.is_zero(), Error::<T>::NotAValidAmount);

            Self::update_reward(Some(who.clone()), asset, reward_asset, lock_duration)?;

            let asset_pool_account = Self::pool_account_id(asset)?;
            Pools::<T>::mutate(
                (asset, reward_asset, lock_duration),
                |pool_info| -> DispatchResult {
                    let pool_info = pool_info.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;

                    ensure!(pool_info.is_active, Error::<T>::PoolIsNotActive);

                    T::Assets::transfer(asset, &who, &asset_pool_account, amount, false)?;

                    pool_info.total_deposited = pool_info
                        .total_deposited
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Overflow)?;

                    Positions::<T>::mutate(
                        (&asset, &reward_asset, &lock_duration, &who),
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
                        lock_duration,
                        amount,
                    ));
                    Ok(())
                },
            )
        }

        /// Withdrawing Assets from reward Pool
        ///
        /// The origin must be Signed and the sender must have sufficient deposited balance.
        ///
        /// - `asset`: The identifier of the staking asset.
        /// - `reward_asset`: The identifier of the reward asset.
        /// - `lock_duration`: Lock block number after Deposit.
        /// - `amount`: the amount of staking asset want to withdraw.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::withdraw())]
        #[transactional]
        pub fn withdraw(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            lock_duration: T::BlockNumber,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Pools::<T>::contains_key((&asset, &reward_asset, &lock_duration)),
                Error::<T>::PoolDoesNotExist
            );
            ensure!(!amount.is_zero(), Error::<T>::NotAValidAmount);

            let user_position = Positions::<T>::get((&asset, &reward_asset, &lock_duration, &who));
            ensure!(
                user_position.deposit_balance >= amount,
                Error::<T>::DepositBalanceLow
            );

            Self::update_reward(Some(who.clone()), asset, reward_asset, lock_duration)?;

            Pools::<T>::mutate(
                (asset, reward_asset, lock_duration),
                |pool_info| -> DispatchResult {
                    let pool_info = pool_info.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;

                    let current_block_number = <frame_system::Pallet<T>>::block_number();
                    ensure!(
                        current_block_number >= pool_info.unlock_height,
                        Error::<T>::PoolUnderLock
                    );

                    pool_info.total_deposited = pool_info
                        .total_deposited
                        .checked_sub(amount)
                        .ok_or(ArithmeticError::Overflow)?;

                    Positions::<T>::mutate(
                        (&asset, &reward_asset, &lock_duration, &who),
                        |user_position| -> DispatchResult {
                            user_position.deposit_balance = user_position
                                .deposit_balance
                                .checked_sub(amount)
                                .ok_or(ArithmeticError::Overflow)?;

                            if pool_info.cool_down_duration.is_zero() {
                                let asset_pool_account = Self::pool_account_id(asset)?;
                                T::Assets::transfer(
                                    asset,
                                    &asset_pool_account,
                                    &who,
                                    amount,
                                    false,
                                )?;
                            } else {
                                user_position
                                    .lock_balance_items
                                    .try_push((amount, current_block_number))
                                    .map_err(|_| Error::<T>::ExcessMaxUserLockItemsCount)?;
                            }

                            Ok(())
                        },
                    )?;

                    Self::deposit_event(Event::<T>::AssetsWithdrew(
                        who,
                        asset,
                        reward_asset,
                        lock_duration,
                        amount,
                    ));
                    Ok(())
                },
            )
        }

        /// Redeem unlocked balance of staking asset from Pool
        ///
        /// Origin must be Signed.
        ///
        /// - `asset`: The identifier of the staking asset.
        /// - `reward_asset`: The identifier of the reward asset.
        /// - `lock_duration`: Lock block number after Deposit.
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::redeem())]
        #[transactional]
        pub fn redeem(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            lock_duration: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let pool_info = Pools::<T>::try_get((&asset, &reward_asset, &lock_duration))
                .map_err(|_err| Error::<T>::PoolDoesNotExist)?;

            let current_block_number = <frame_system::Pallet<T>>::block_number();
            Positions::<T>::mutate(
                (&asset, &reward_asset, &lock_duration, &who),
                |user_position| -> DispatchResult {
                    let mut total_amount: BalanceOf<T> = 0;
                    for item in user_position.lock_balance_items.iter() {
                        let unlock_block = item.1.saturating_add(pool_info.cool_down_duration);
                        if current_block_number >= unlock_block {
                            total_amount = total_amount
                                .checked_add(item.0)
                                .ok_or(ArithmeticError::Overflow)?;
                        }
                    }

                    user_position.lock_balance_items.retain(|item| {
                        let unlock_block = item.1.saturating_add(pool_info.cool_down_duration);
                        current_block_number < unlock_block
                    });

                    if total_amount > 0 {
                        let asset_pool_account = Self::pool_account_id(asset)?;
                        T::Assets::transfer(asset, &asset_pool_account, &who, total_amount, false)?;
                    }

                    Self::deposit_event(Event::<T>::AssetsRedeem(
                        who.clone(),
                        asset,
                        reward_asset,
                        lock_duration,
                        total_amount,
                    ));
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
        /// - `lock_duration`: Lock block number after Deposit.
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::claim())]
        #[transactional]
        pub fn claim(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            lock_duration: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Pools::<T>::contains_key((&asset, &reward_asset, &lock_duration)),
                Error::<T>::PoolDoesNotExist
            );

            Self::update_reward(Some(who.clone()), asset, reward_asset, lock_duration)?;

            let asset_pool_account = Self::pool_account_id(reward_asset)?;
            Positions::<T>::mutate(
                (&asset, &reward_asset, &lock_duration, &who),
                |user_position| -> DispatchResult {
                    let reward_amount = user_position.reward_amount;
                    if reward_amount > 0 {
                        T::Assets::transfer(
                            reward_asset,
                            &asset_pool_account,
                            &who,
                            reward_amount,
                            false,
                        )?;
                        user_position.reward_amount = 0;
                    }

                    Self::deposit_event(Event::<T>::RewardPaid(
                        who.clone(),
                        asset,
                        reward_asset,
                        lock_duration,
                        reward_amount,
                    ));
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
        /// - `lock_duration`: Lock block number after Deposit.
        /// - `payer`: the payer of reward asset.
        /// - `amount`: the amount of reward asset to dispatch.
        /// - `duration`: the number of block this reward will last for.
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::dispatch_reward())]
        #[transactional]
        pub fn dispatch_reward(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            reward_asset: AssetIdOf<T>,
            lock_duration: T::BlockNumber,
            payer: <T::Lookup as StaticLookup>::Source,
            amount: BalanceOf<T>,
            reward_duration: T::BlockNumber,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            ensure!(
                Pools::<T>::contains_key((&asset, &reward_asset, &lock_duration)),
                Error::<T>::PoolDoesNotExist
            );
            ensure!(!reward_duration.is_zero(), Error::<T>::NotAValidDuration);

            Self::update_reward(None, asset, reward_asset, lock_duration)?;

            let current_block_number = <frame_system::Pallet<T>>::block_number();
            Pools::<T>::mutate(
                (asset, reward_asset, lock_duration),
                |pool_info| -> DispatchResult {
                    let pool_info = pool_info.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;
                    let duration_balance = pool_info.block_to_balance(reward_duration);
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
                        .checked_add(&reward_duration)
                        .ok_or(ArithmeticError::Overflow)?;

                    pool_info.reward_duration = reward_duration;
                    pool_info.period_finish = new_period_finish;
                    pool_info.reward_rate = reward_rate;
                    pool_info.last_update_block = current_block_number;

                    if amount > 0 {
                        let asset_pool_account = Self::pool_account_id(reward_asset)?;
                        let payer = T::Lookup::lookup(payer)?;
                        T::Assets::transfer(
                            reward_asset,
                            &payer,
                            &asset_pool_account,
                            amount,
                            false,
                        )?;
                    }

                    Self::deposit_event(Event::<T>::RewardAdded(
                        asset,
                        reward_asset,
                        lock_duration,
                        amount,
                    ));
                    Ok(())
                },
            )
        }
    }
}

impl<T: Config> Pallet<T> {
    fn update_reward(
        who: Option<T::AccountId>,
        asset: AssetIdOf<T>,
        reward_asset: AssetIdOf<T>,
        lock_duration: T::BlockNumber,
    ) -> DispatchResult {
        let current_block_number = <frame_system::Pallet<T>>::block_number();

        //1, update pool reward info
        Pools::<T>::mutate(
            (asset, reward_asset, lock_duration),
            |pool_info| -> DispatchResult {
                let pool_info = pool_info.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;

                pool_info.update_reward_per_share(current_block_number)?;

                //2, update user reward info
                if let Some(who) = who {
                    Positions::<T>::mutate(
                        (&asset, &reward_asset, &lock_duration, &who),
                        |user_position| -> DispatchResult {
                            let diff = pool_info
                                .reward_per_share(current_block_number)?
                                .checked_sub(user_position.reward_per_share_paid)
                                .ok_or(ArithmeticError::Overflow)?;

                            let earned = user_position
                                .deposit_balance
                                .get_big_uint()
                                .checked_mul(&diff.get_big_uint())
                                .and_then(|r| {
                                    r.checked_div(&pool_info.amount_per_share().get_big_uint())
                                })
                                .and_then(|r| {
                                    r.checked_add(&user_position.reward_amount.get_big_uint())
                                })
                                .and_then(|r| r.to_u128())
                                .ok_or(ArithmeticError::Overflow)?;

                            user_position.reward_amount = BalanceOf::<T>::saturated_from(earned);
                            user_position.reward_per_share_paid = pool_info.reward_per_share_stored;

                            Ok(())
                        },
                    )?
                }
                Ok(())
            },
        )
    }

    fn pool_account_id(asset_id: AssetIdOf<T>) -> Result<T::AccountId, DispatchError> {
        let account_id: T::AccountId = T::PalletId::get().into_account_truncating();
        let entropy = (b"modlpy/liquidity", &[account_id], asset_id).using_encoded(blake2_256);
        Ok(T::AccountId::decode(&mut &entropy[..]).map_err(|_| Error::<T>::CodecError)?)
    }
}
