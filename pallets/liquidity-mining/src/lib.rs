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

//! # Pure Liquidity Mining (PLM)
//!
//! pallet-liquidity-mining is in charge of creating a governance-controlled incentivization program for our different products.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod benchmarking;

pub mod weights;

use codec::{Decode, Encode};
use frame_support::{
    pallet_prelude::*,
    traits::{
        fungibles::{Inspect, Mutate, Transfer},
        Get, IsType,
    },
    transactional, Blake2_128Concat, PalletId,
};
use frame_system::ensure_signed;
use frame_system::pallet_prelude::OriginFor;
pub use pallet::*;
use primitives::{Balance, CurrencyId};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_io::hashing::blake2_256;
use sp_runtime::{
    traits::{AccountIdConversion, Saturating, StaticLookup, Zero},
    ArithmeticError, SaturatedConversion,
};
pub use weights::WeightInfo;

pub type AssetIdOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type BalanceOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config<I: 'static = ()>:
        frame_system::Config
        + pallet_assets::Config<AssetId = AssetIdOf<Self, I>, Balance = BalanceOf<Self, I>>
    {
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

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

        /// Specifies how many reward tokens can be manipulated by a pool
        #[pallet::constant]
        type MaxRewardTokens: Get<u32>;
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Pool does not exist
        PoolDoesNotExist,
        /// Pool associacted with asset already exists
        PoolAlreadyExists,
        /// Not a newly created asset
        NotANewlyCreatedAsset,
        /// Not a valid duration
        NotAValidDuration,
        /// Not a valid amount
        NotAValidAmount,
        /// The end block is smaller than start block
        SmallerThanEndBlock,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// Add new pool
        /// [asset_id]
        PoolAdded(AssetIdOf<T, I>),
        /// Deposited Assets in pool
        /// [sender, asset_id]
        AssetsDeposited(T::AccountId, AssetIdOf<T, I>),
        /// Withdrew Assets from pool
        /// [sender, asset_id]
        AssetsWithdrew(T::AccountId, AssetIdOf<T, I>),
    }

    #[pallet::pallet]
    pub struct Pallet<T, I = ()>(_);

    #[derive(
        Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo,
    )]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct Pool<BlockNumber, AssetId, BoundedTokens> {
        /// When the liquidity program starts
        start: BlockNumber,
        /// When the liquidity program stops
        end: BlockNumber,
        /// Which assets we use to send rewards
        rewards: BoundedTokens,
        /// Which asset we use to represent shares of the pool
        asset_id: AssetId,
    }

    /// Each pool is associated to a unique AssetId (not be mixed with the reward asset)
    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        AssetIdOf<T, I>,
        Pool<
            T::BlockNumber,
            AssetIdOf<T, I>,
            BoundedVec<(BalanceOf<T, I>, AssetIdOf<T, I>), T::MaxRewardTokens>,
        >,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Create new pool, associated with a unique asset id
        #[pallet::weight(<T as Config<I>>::WeightInfo::create())]
        #[transactional]
        pub fn create(
            origin: OriginFor<T>,
            asset: AssetIdOf<T, I>,
            stash: <T::Lookup as StaticLookup>::Source,
            start: T::BlockNumber,
            end: T::BlockNumber,
            rewards: BoundedVec<(BalanceOf<T, I>, AssetIdOf<T, I>), T::MaxRewardTokens>,
            asset_id: AssetIdOf<T, I>,
        ) -> DispatchResultWithPostInfo {
            T::CreateOrigin::ensure_origin(origin)?;

            ensure!(end > start, Error::<T, I>::SmallerThanEndBlock);

            ensure!(
                !Pools::<T, I>::contains_key(&asset),
                Error::<T, I>::PoolAlreadyExists
            );

            ensure!(
                T::Assets::total_issuance(asset_id).is_zero(),
                Error::<T, I>::NotANewlyCreatedAsset
            );

            let current_block_number = <frame_system::Pallet<T>>::block_number();
            ensure!(
                start >= current_block_number,
                Error::<T, I>::NotAValidDuration
            );

            let stash = T::Lookup::lookup(stash)?;

            let asset_pool_account = Self::pool_account_id(asset);

            for (per_block, reward_token) in rewards.clone() {
                let total_rewards =
                    Self::block_to_balance(end.saturating_sub(start)).saturating_mul(per_block);
                T::Assets::transfer(
                    reward_token,
                    &stash,
                    &asset_pool_account,
                    total_rewards,
                    true,
                )?;
            }

            let pool = Pool {
                start,
                end,
                rewards,
                asset_id,
            };

            Pools::<T, I>::insert(&asset, pool);
            Self::deposit_event(Event::<T, I>::PoolAdded(asset));
            Ok(().into())
        }

        /// Depositing Assets in a Pool
        #[pallet::weight(<T as Config<I>>::WeightInfo::deposit())]
        #[transactional]
        pub fn deposit(
            origin: OriginFor<T>,
            asset: AssetIdOf<T, I>,
            #[pallet::compact] amount: BalanceOf<T, I>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(amount != Zero::zero(), Error::<T, I>::NotAValidAmount);

            let asset_pool_account = Self::pool_account_id(asset);
            Pools::<T, I>::try_mutate(asset, |liquidity_pool| -> DispatchResult {
                let pool = liquidity_pool
                    .as_mut()
                    .ok_or(Error::<T, I>::PoolDoesNotExist)?;

                let current_block_number = <frame_system::Pallet<T>>::block_number();
                ensure!(
                    current_block_number >= pool.start && current_block_number <= pool.end,
                    Error::<T, I>::NotAValidDuration
                );

                T::Assets::transfer(asset, &who, &asset_pool_account, amount, true)?;

                T::Assets::mint_into(pool.asset_id, &who, amount)?;

                Self::deposit_event(Event::<T, I>::AssetsDeposited(who, asset));
                Ok(())
            })
        }

        /// Claiming Rewards or Withdrawing Assets from a Pool
        #[pallet::weight(<T as Config<I>>::WeightInfo::withdraw())]
        #[transactional]
        pub fn withdraw(
            origin: OriginFor<T>,
            asset: AssetIdOf<T, I>,
            #[pallet::compact] amount: BalanceOf<T, I>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(amount != Zero::zero(), Error::<T, I>::NotAValidAmount);

            let asset_pool_account = Self::pool_account_id(asset);

            Pools::<T, I>::try_mutate(asset, |liquidity_pool| -> DispatchResultWithPostInfo {
                let pool = liquidity_pool
                    .as_mut()
                    .ok_or(Error::<T, I>::PoolDoesNotExist)?;

                for (per_block, reward_token) in pool.rewards.clone() {
                    let duration: T::BlockNumber =
                        <frame_system::Pallet<T>>::block_number().saturating_sub(pool.start);
                    let percent = amount
                        .checked_div(T::Assets::total_issuance(pool.asset_id))
                        .ok_or(ArithmeticError::Overflow)?;
                    let total_rewards = percent
                        .saturating_mul(Self::block_to_balance(duration).saturating_mul(per_block));

                    T::Assets::transfer(
                        reward_token,
                        &asset_pool_account,
                        &who,
                        total_rewards,
                        true,
                    )?;
                }

                T::Assets::transfer(asset, &asset_pool_account, &who, amount, true)?;
                T::Assets::burn_from(pool.asset_id, &who, amount)?;

                Self::deposit_event(Event::<T, I>::AssetsWithdrew(who, asset));
                Ok(().into())
            })
        }
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    pub fn pool_account_id(asset_id: AssetIdOf<T, I>) -> T::AccountId {
        let account_id: T::AccountId = T::PalletId::get().into_account();
        let entropy = (b"modlpy/liquidity", &[account_id], asset_id).using_encoded(blake2_256);
        T::AccountId::decode(&mut &entropy[..]).unwrap_or_default()
    }

    fn block_to_balance(input: T::BlockNumber) -> T::Balance {
        input.saturated_into::<u128>()
    }
}
