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

//! # Stable Swap
//!
//! Given any [X, Y] asset pair, "base" is the `X` asset while "quote" is the `Y` asset.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

use frame_support::{
    log,
    traits::{
        fungibles::{Inspect, Mutate, Transfer},
        Get, IsType,
    },
    PalletId,
};
use primitives::{Balance, ConvertToBigUint, CurrencyId, StableSwap, AMM};
use sp_runtime::{ArithmeticError, DispatchError};
use sp_std::result::Result;

pub use pallet::*;
pub use weights::WeightInfo;

use num_traits::{CheckedDiv, CheckedMul};

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type AssetIdOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type BalanceOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    pub type Amounts<T, I> = sp_std::vec::Vec<BalanceOf<T, I>>;

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

        /// Currency type for deposit/withdraw assets to/from amm
        /// module
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Specify all the AMMs we are routing between
        type AMM: AMM<AccountIdOf<Self>, AssetIdOf<Self, I>, BalanceOf<Self, I>>;

        #[pallet::constant]
        type NumTokens: Get<u8>;

        /// Precision
        #[pallet::constant]
        type Precision: Get<u128>;

        /// Optimal Amplification Coefficient
        #[pallet::constant]
        type AmplificationCoefficient: Get<u8>;
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Conversion failure to u128
        ConversionToU128Failed,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// Delta Calculated
        DeltaCalculated(AssetIdOf<T, I>, AssetIdOf<T, I>, u128),
    }

    #[pallet::pallet]
    pub struct Pallet<T, I = ()>(_);

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        // https://miguelmota.com/blog/understanding-stableswap-curve/
        // https://github.com/curvefi/curve-contract/blob/master/contracts/pool-templates/base/SwapTemplateBase.vy
        // https://github.com/parallel-finance/amm-formula/blob/master/src/formula.rs
        // https://curve.fi/files/stableswap-paper.pdf
        pub(crate) fn do_get_delta(
            (asset_in, asset_out): (AssetIdOf<T, I>, AssetIdOf<T, I>),
        ) -> Result<BalanceOf<T, I>, DispatchError> {
            let (x, y) = T::AMM::get_reserves(asset_in, asset_out).unwrap();

            let total_reserves = x.checked_add(y).ok_or(ArithmeticError::Overflow)?;

            let a: u128 = (T::AmplificationCoefficient::get() as u128)
                .checked_mul(T::Precision::get())
                .ok_or(ArithmeticError::Overflow)?;

            let mut prev_d: u128;

            let mut d = total_reserves;
            let n_a = a.checked_mul(2u128).ok_or(ArithmeticError::Overflow)?;

            // 255 is a max number of loops
            // should throw error if does not converge
            for _ in 0..255 {
                let mut dp = d;

                // repeat twice instead of a loop since we only
                // support two pools

                // dp = (dp * d) / (x * n_t);
                dp = dp
                    .checked_mul(d)
                    .ok_or(ArithmeticError::Overflow)?
                    .checked_div(
                        x.checked_mul(T::NumTokens::get().into())
                            .ok_or(ArithmeticError::Overflow)?,
                    )
                    .ok_or(ArithmeticError::Underflow)?;

                // dp = (dp * d) / (y * n_t);
                dp = dp
                    .checked_mul(d)
                    .ok_or(ArithmeticError::Overflow)?
                    .checked_div(
                        y.checked_mul(T::NumTokens::get().into())
                            .ok_or(ArithmeticError::Overflow)?,
                    )
                    .ok_or(ArithmeticError::Underflow)?;

                prev_d = d;

                // d = ((((n_a * s) / a_precision) + (dp * n_t)) * d)
                //     / ((((n_a - a_precision) * d) / a_precision) + ((n_t + 1) * dp));
                let k = dp
                    .checked_mul(T::NumTokens::get().into())
                    .ok_or(ArithmeticError::Overflow)?;

                let m = n_a
                    .checked_mul(total_reserves)
                    .ok_or(ArithmeticError::Overflow)?
                    .checked_div(T::Precision::get())
                    .ok_or(ArithmeticError::Underflow)?
                    .checked_add(k)
                    .ok_or(ArithmeticError::Overflow)?;

                let n = n_a
                    .checked_sub(T::Precision::get())
                    .ok_or(ArithmeticError::Underflow)?
                    .checked_mul(d)
                    .ok_or(ArithmeticError::Overflow)?;

                let u = n
                    .checked_div(T::Precision::get())
                    .ok_or(ArithmeticError::Underflow)?;

                let l = (T::NumTokens::get() as u128)
                    .checked_add(1u128)
                    .ok_or(ArithmeticError::Overflow)?
                    .checked_mul(dp)
                    .ok_or(ArithmeticError::Overflow)?;

                let _denom = u.checked_add(l).ok_or(ArithmeticError::Overflow)?;

                d = m
                    .checked_mul(d)
                    .ok_or(ArithmeticError::Overflow)?
                    .checked_div(_denom)
                    .ok_or(ArithmeticError::Underflow)?;

                // check if difference is less than 1
                if d > prev_d {
                    if d - prev_d < 1 {
                        break;
                    }
                } else if prev_d - d < 1 {
                    break;
                }
            }
            // throw new Error('D does not converge')
            Self::deposit_event(Event::<T, I>::DeltaCalculated(asset_in, asset_out, d));
            Ok(d)
        }

        pub(crate) fn do_get_alternative_var(
            mut autonomous_var: BalanceOf<T, I>,
            (asset_in, asset_out): (AssetIdOf<T, I>, AssetIdOf<T, I>),
        ) -> Result<BalanceOf<T, I>, DispatchError> {
            let (resx, _) = T::AMM::get_reserves(asset_in, asset_out).unwrap();
            autonomous_var += resx;

            // passes asset in and asset out
            let d = Self::get_d((asset_in, asset_out)).unwrap();

            let mut c = d;
            let mut s = 0u128;

            let a = (T::AmplificationCoefficient::get() as u128)
                .checked_mul(T::Precision::get())
                .ok_or(ArithmeticError::Underflow)?;

            let n_a = (T::NumTokens::get() as u128)
                .checked_mul(a)
                .ok_or(ArithmeticError::Overflow)?;

            let _x = 0u128;

            s = s
                .checked_add(autonomous_var)
                .ok_or(ArithmeticError::Overflow)?;

            c = (c.checked_mul(d).ok_or(ArithmeticError::Underflow)?)
                .checked_div(
                    autonomous_var
                        .checked_mul(T::NumTokens::get().into())
                        .ok_or(ArithmeticError::Underflow)?,
                )
                .ok_or(ArithmeticError::Underflow)?;

            c = (c
                .checked_mul(d)
                .ok_or(ArithmeticError::Underflow)?
                .checked_mul(T::Precision::get())
                .ok_or(ArithmeticError::Underflow)?)
            .checked_div(
                n_a.checked_mul(T::NumTokens::get() as u128)
                    .ok_or(ArithmeticError::Underflow)?,
            )
            .ok_or(ArithmeticError::Underflow)?;

            let b = s
                .checked_add(
                    d.checked_mul(T::Precision::get())
                        .ok_or(ArithmeticError::Underflow)?
                        .checked_div(n_a)
                        .ok_or(ArithmeticError::Underflow)?,
                )
                .ok_or(ArithmeticError::Underflow)?;

            let mut y_prev;
            let mut y = d;
            // 255 is a max number of loops
            // should throw error if does not converge
            for _ in 0..255 {
                y_prev = y;

                y = (y
                    .checked_mul(y)
                    .ok_or(ArithmeticError::Underflow)?
                    .checked_add(c)
                    .ok_or(ArithmeticError::Underflow)?)
                .checked_div(
                    (y.checked_mul(2u128)
                        .ok_or(ArithmeticError::Underflow)?
                        .checked_add(b)
                        .ok_or(ArithmeticError::Underflow)?)
                    .checked_sub(d)
                    .ok_or(ArithmeticError::Underflow)?,
                )
                .ok_or(ArithmeticError::Underflow)?;

                if y.eq(&y_prev) {
                    break;
                }
            }

            Ok(y)
            // throw new Error('Approximation did not converge')
        }

        // ****************************************************************************************
    }
}

// For Parallel Router
impl<T: Config<I>, I: 'static>
    primitives::StableSwap<AccountIdOf<T>, AssetIdOf<T, I>, BalanceOf<T, I>> for Pallet<T, I>
{
    fn get_d(pair: (AssetIdOf<T, I>, AssetIdOf<T, I>)) -> Result<u128, DispatchError> {
        let d = Self::do_get_delta(pair)?;
        Ok(d)
    }

    fn get_alternative_var(
        autonomous_var: BalanceOf<T, I>,
        pair: (AssetIdOf<T, I>, AssetIdOf<T, I>),
    ) -> Result<u128, DispatchError> {
        let alternative_var = Self::do_get_alternative_var(autonomous_var, pair)?;
        Ok(alternative_var)
    }
}
