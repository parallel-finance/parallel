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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod mock;
mod pool;
mod tests;
mod y_evaluation;

pub use pallet::*;
use pool::Pool;
use y_evaluation::YEvaluation;

#[frame_support::pallet]
mod pallet {
    use crate::{Pool, YEvaluation};
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::{StorageMap, StorageValue, ValueQuery},
        traits::{GenesisBuild, Get, Hooks, IsType},
        Blake2_128Concat, PalletId, Parameter,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use orml_traits::{MultiCurrency, MultiCurrencyExtended};
    use parallel_primitives::{Amount, Balance, CurrencyId, Rate};
    use sp_arithmetic::traits::BaseArithmetic;
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedAdd, Saturating},
        ArithmeticError, DispatchError, FixedPointNumber,
    };

    // Amplification Coefficient Weight.
    //
    // In this pallet, the actual amplification coefficient will be `exchange_rate` * `ACW`.
    const ACW: Rate = Rate::from_inner(Rate::DIV / 100 * 50); // 50%

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        #[frame_support::transactional]
        pub fn create_pool(origin: OriginFor<T>, pool: Pool) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let pool_id = Self::increase_pools_counter()?;
            let pool_account = Self::pool_account(&pool_id);
            T::Currency::transfer(pool.asset_base, &who, &pool_account, pool.amount_base)?;
            T::Currency::transfer(pool.asset_quote, &who, &pool_account, pool.amount_quote)?;
            <Pools<T>>::insert(pool_id, pool);
            Ok(())
        }

        /// In an AMM context, buying `X` (base) means swapping `Y` (quote) for `X` (base) while the opposite
        /// (selling) means swapping `X` (base) for `Y` (quote).
        #[pallet::weight(0)]
        #[frame_support::transactional]
        pub fn sell_with_exact_amount(
            origin: OriginFor<T>,
            amount_base: Balance,
            pool_id: T::PoolId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut pool = Self::pool(&pool_id)?;
            let x_opt = pool.amount_base.checked_sub(amount_base);
            let x = x_opt.ok_or(ArithmeticError::Underflow)?;
            let YEvaluation { y, y_diff } = Self::calculate_y(x, &pool)?;
            pool.amount_quote = y;
            let pool_account = Self::pool_account(&pool_id);
            T::Currency::transfer(pool.asset_base, &who, &pool_account, amount_base)?;
            T::Currency::transfer(pool.asset_quote, &pool_account, &who, y_diff)?;
            Self::mutate_pool(&pool_id, |previous_pool| *previous_pool = pool)?;
            Ok(())
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currency: MultiCurrencyExtended<
            Self::AccountId,
            CurrencyId = CurrencyId,
            Balance = Balance,
            Amount = Amount,
        >;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type PalletId: Get<PalletId>;
        type PoolId: Default + BaseArithmetic + Parameter;
    }

    #[pallet::error]
    pub enum Error<T> {
        PoolDoesNotExist,
    }

    #[pallet::event]
    pub enum Event<T>
    where
        T: Config, {}

    #[pallet::genesis_config]
    pub struct GenesisConfig<T> {
        pub exchange_rate: Rate,
        pub phantom: PhantomData<T>,
    }

    #[cfg(feature = "std")]
    impl<T> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig {
                exchange_rate: Default::default(),
                phantom: PhantomData,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            ExchangeRate::<T>::put(self.exchange_rate);
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    impl<T> Pallet<T>
    where
        T: Config,
    {
        // Each pool has an account derived from the pool id.
        pub(crate) fn pool_account(pool_id: &T::PoolId) -> T::AccountId {
            T::PalletId::get().into_sub_account(pool_id)
        }

        // Multiplies an arbitrary coefficient value with the current amplification coefficient.
        fn amplification_coeficient_mul(n: u128) -> Option<u128> {
            let exchange_rate = ExchangeRate::<T>::get();
            // Saturates because a very large amplification coefficient
            // will simply shape the curve as a constant sum equation.
            let amplif_coefficient = ACW.saturating_add(exchange_rate);
            amplif_coefficient.checked_mul_int(n)
        }

        // Related to the underlying formula used to evaluate swaps. Calculates `y` (quote) amount
        // and the different between the previous `y` amount based on the provided amount of
        // `x` (base).
        //
        // let y = (k * (4*A*k + k - 4*A*x)) / (4 * (A*k + x))
        fn calculate_y(x: Balance, pool: &Pool) -> Result<YEvaluation, DispatchError> {
            let k = Self::total_assets(pool)?;
            let f = || {
                let ak = Self::amplification_coeficient_mul(k)?;
                let _4ax = 4u128.checked_mul(Self::amplification_coeficient_mul(x)?)?;
                let _4ak = 4u128.checked_mul(ak)?;
                let dividend = k.checked_mul(_4ak.checked_add(k)?.checked_sub(_4ax)?)?;
                let divisor = 4u128.checked_mul(ak.checked_add(x)?)?;
                dividend.checked_div(divisor)
            };
            let y = f().ok_or_else::<DispatchError, _>(|| ArithmeticError::Overflow.into())?;
            let [greater, lesser] = if pool.amount_quote > y {
                [pool.amount_quote, y]
            } else {
                [y, pool.amount_quote]
            };
            let y_diff = greater
                .checked_sub(lesser)
                .ok_or_else::<DispatchError, _>(|| ArithmeticError::Underflow.into())?;
            Ok(YEvaluation { y, y_diff })
        }

        // Increases the internal pool counter by 1 and returns the id that should
        // be attached to a new stored pool.
        fn increase_pools_counter() -> Result<T::PoolId, DispatchError> {
            let curr = <PoolsCounter<T>>::get();
            <PoolsCounter<T>>::try_mutate(|n| {
                let opt = n.checked_add(&T::PoolId::from(1u8));
                *n = opt.ok_or_else::<DispatchError, _>(|| ArithmeticError::Overflow.into())?;
                Ok::<_, DispatchError>(())
            })?;
            Ok(curr)
        }

        fn mutate_pool<F>(pool_id: &T::PoolId, cb: F) -> Result<(), DispatchError>
        where
            F: FnOnce(&mut Pool),
        {
            Pools::<T>::try_mutate(pool_id, |opt| {
                if let Some(market) = opt {
                    cb(market);
                    return Ok(());
                }
                Err(Error::<T>::PoolDoesNotExist.into())
            })
        }

        fn pool(pool_id: &T::PoolId) -> Result<Pool, DispatchError> {
            Pools::<T>::get(pool_id).ok_or_else(|| Error::<T>::PoolDoesNotExist.into())
        }

        // Sum of the two pool assets
        fn total_assets(pool: &Pool) -> Result<Balance, DispatchError> {
            pool.amount_base
                .checked_add(pool.amount_quote)
                .ok_or_else(|| ArithmeticError::Overflow.into())
        }
    }

    #[pallet::storage]
    pub type ExchangeRate<T: Config> = StorageValue<_, Rate, ValueQuery>;

    #[pallet::storage]
    pub type Pools<T: Config> = StorageMap<_, Blake2_128Concat, T::PoolId, Pool>;

    #[pallet::storage]
    pub type PoolsCounter<T: Config> = StorageValue<_, T::PoolId, ValueQuery>;
}
