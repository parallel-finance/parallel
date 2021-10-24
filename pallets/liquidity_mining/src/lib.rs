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

use codec::{Decode, Encode};
use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{
        fungibles::{Inspect, Mutate, Transfer},
        Get, Hooks, IsType,
    },
    transactional, Blake2_128Concat, PalletId, Twox64Concat,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor, RawOrigin};
pub use pallet::*;
use primitives::{Balance, CurrencyId, Rate};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_io::hashing::blake2_256;
use sp_runtime::{
    traits::{
        AccountIdConversion, CheckedDiv, IntegerSquareRoot, One, StaticLookup, UniqueSaturatedInto,
        Zero,
    },
    ArithmeticError, DispatchError, FixedU128, Perbill, SaturatedConversion,
};

pub type AssetIdOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type BalanceOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_system::ensure_root;

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

        /// The origin which can create new pools.
        type CreateOrigin: EnsureOrigin<Self::Origin>;

        /// Specifies how many reward tokens can be manipulated by a pool
        #[pallet::constant]
        type MaxRewardTokens: Get<u32>;
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Per block and rewards are not same size
        PerBlockAndRewardsAreNotSameSize,
        /// Pool associacted with asset already exists
        PoolAlreadyExists,
        /// Not a newly created asset
        NotANewlyCreatedAsset,
        /// Start block number is less than current block number
        StartBlockNumberLessThanCurrentBlockNumber,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// Add new pool
        /// [sender, asset_id]
        PoolAdded(T::AccountId, AssetIdOf<T, I>),
    }

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<T::BlockNumber> for Pallet<T, I> {}

    #[pallet::pallet]
    pub struct Pallet<T, I = ()>(_);

    #[derive(
        Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo,
    )]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct Pool<BlockNumber, Balance, AssetId, BoundedTokens> {
        /// When the liquidity program starts
        start: BlockNumber,
        /// When the liquidity program stops
        end: BlockNumber,
        /// How much is vested into the pool every block, will be shared among
        /// all participants
        per_block: Balance,
        /// Which assets we use to send rewards
        rewards: BoundedTokens,
        /// Which asset we use to represent shares of the pool
        shares: AssetId,
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
            BalanceOf<T, I>,
            AssetIdOf<T, I>,
            BoundedVec<AssetIdOf<T, I>, T::MaxRewardTokens>,
        >,
        OptionQuery,
    >;

    /// ## Tracking Contributions
    ///
    /// Contributions can be tracked by user's account id and the asset of the pool they are depositing into.
    #[pallet::storage]
    #[pallet::getter(fn deposits)]
    pub type Deposits<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        AssetIdOf<T, I>,
        BalanceOf<T, I>,
    >;

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Create new pool, associated with a unique asset id
        #[pallet::weight(10000)]
        #[transactional]
        pub fn create(
            origin: OriginFor<T>,
            asset: AssetIdOf<T, I>,
            stash: <T::Lookup as StaticLookup>::Source,
            start: T::BlockNumber,
            end: T::BlockNumber,
            per_block: BoundedVec<BalanceOf<T, I>, T::MaxRewardTokens>,
            rewards: BoundedVec<AssetIdOf<T, I>, T::MaxRewardTokens>,
            shares: AssetIdOf<T, I>,
        ) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    pub fn pool_account_id(asset_id: u16) -> T::AccountId {
        let account_id: T::AccountId = T::PalletId::get().into_account();
        let entropy = (b"modlpy/liquidity", &[account_id], asset_id).using_encoded(blake2_256);
        T::AccountId::decode(&mut &entropy[..]).unwrap_or_default()
    }
}
