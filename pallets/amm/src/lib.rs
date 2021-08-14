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

//! # Automatic Market Maker (AMM)
//!
//! Given any [X, Y] asset pair, "base" is the `X` asset while "quote" is the `Y` asset.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod pool_structs;

pub use pallet::*;
use pool_structs::{AMMCurve, LiquidityProviderAmounts, Pool, StabilityPool};

#[frame_support::pallet]
mod pallet {
    use crate::{AMMCurve, LiquidityProviderAmounts, Pool, StabilityPool};
    use core::marker::PhantomData;
    use frame_support::{
        pallet_prelude::{StorageDoubleMap, StorageMap, StorageValue, ValueQuery},
        traits::{EnsureOrigin, GenesisBuild, Get, Hooks, IsType},
        Blake2_128Concat, PalletId, Parameter,
    };
    use orml_traits::MultiCurrencyExtended;
    use parallel_primitives::{Amount, Balance, CurrencyId, Rate};
    use sp_arithmetic::traits::BaseArithmetic;
    use sp_runtime::Perbill;

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {}

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        type Currency: MultiCurrencyExtended<
            Self::AccountId,
            CurrencyId = CurrencyId,
            Balance = Balance,
            Amount = Amount,
        >;
        type Curve: AMMCurve;
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;
        type LpFee: Get<Perbill>;
        type PalletId: Get<PalletId>;
        type PoolId: Default + BaseArithmetic + Parameter;
        type PoolManager: EnsureOrigin<Self::Origin>;
        type StabilityPool: StabilityPool;
        type TreasuryAccount: Get<Self::AccountId>;
        type TreasuryFee: Get<Perbill>;
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {}

    #[pallet::event]
    pub enum Event<T: Config<I>, I: 'static = ()> {}

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
        pub exchange_rate: Rate,
        pub phantom: PhantomData<(T, I)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
        fn default() -> Self {
            GenesisConfig {
                exchange_rate: Default::default(),
                phantom: PhantomData,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
        fn build(&self) {
            ExchangeRate::<T, I>::put(self.exchange_rate);
        }
    }

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<T::BlockNumber> for Pallet<T, I> {}

    #[pallet::pallet]

    pub struct Pallet<T, I = ()>(_);

    /// The exchange rate from the underlying to the internal collateral
    #[pallet::storage]
    pub type ExchangeRate<T, I = ()> = StorageValue<_, Rate, ValueQuery>;

    /// Accounts that deposits and withdraw assets in one or more pools
    #[pallet::storage]
    pub type LiquidityProviders<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::PoolId,
        Blake2_128Concat,
        T::AccountId,
        LiquidityProviderAmounts,
    >;

    /// A bag of liquidity composed by two different assets
    #[pallet::storage]
    pub type Pools<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::PoolId, Pool>;

    /// Auxiliary storage used to track pool ids
    #[pallet::storage]
    pub type PoolsCounter<T: Config<I>, I: 'static = ()> = StorageValue<_, T::PoolId, ValueQuery>;
}
