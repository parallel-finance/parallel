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
//! Provide low slippage and low fees when trading stablecoins

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

use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type AssetIdOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type BalanceOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use num_traits::ToPrimitive;

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        type WeightInfo: WeightInfo;

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
        ) -> Result<Balance, DispatchError> {
            let (x, y) = T::AMM::get_reserves(asset_in, asset_out).unwrap();

            let total_reserves = x
                .get_big_uint()
                .checked_add(&y.get_big_uint())
                .ok_or(Error::<T, I>::ConversionToU128Failed)?
                .to_u128()
                .ok_or(ArithmeticError::Underflow)?;

            let a: u128 = (T::AmplificationCoefficient::get() as u128)
                .get_big_uint()
                .checked_mul(&T::Precision::get().get_big_uint())
                .ok_or(Error::<T, I>::ConversionToU128Failed)?
                .to_u128()
                .ok_or(ArithmeticError::Underflow)?;

            let mut prev_d: u128;

            let mut d = total_reserves;

            let n_a = a.checked_mul(2u128).ok_or(ArithmeticError::Overflow)?;

            // 255 is a max number of loops
            // should throw error if does not converge
            for _ in 0..255 {
                let mut dp = d;

                dp = dp
                    .get_big_uint()
                    .checked_mul(&d.get_big_uint())
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Underflow)?
                    .checked_div(
                        x.get_big_uint()
                            .checked_mul(&(T::NumTokens::get() as u128).get_big_uint())
                            .ok_or(Error::<T, I>::ConversionToU128Failed)?
                            .to_u128()
                            .ok_or(ArithmeticError::Underflow)?,
                    )
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Underflow)?;

                dp = dp
                    .get_big_uint()
                    .checked_mul(&d.get_big_uint())
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Underflow)?
                    .checked_div(
                        y.get_big_uint()
                            .checked_mul(&(T::NumTokens::get() as u128).get_big_uint())
                            .ok_or(Error::<T, I>::ConversionToU128Failed)?
                            .to_u128()
                            .ok_or(ArithmeticError::Underflow)?,
                    )
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Underflow)?;

                prev_d = d;

                // d = ((((n_a * s) / a_precision) + (dp * n_t)) * d)
                //     / ((((n_a - a_precision) * d) / a_precision) + ((n_t + 1) * dp));
                let k = dp
                    .checked_mul(T::NumTokens::get().into())
                    .ok_or(ArithmeticError::Overflow)?;

                let m = n_a
                    .get_big_uint()
                    .checked_mul(&total_reserves.get_big_uint())
                    .and_then(|r| r.checked_div(&T::Precision::get().get_big_uint()))
                    .and_then(|r| r.checked_add(&k.get_big_uint()))
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Underflow)?;

                let n = n_a
                    .get_big_uint()
                    .checked_sub(&T::Precision::get().get_big_uint())
                    .and_then(|r| r.checked_mul(&d.get_big_uint()))
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Underflow)?;

                let u = n
                    .get_big_uint()
                    .checked_div(&T::Precision::get().get_big_uint())
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Underflow)?;

                let l = (T::NumTokens::get() as u128)
                    .checked_add(1u128)
                    .ok_or(ArithmeticError::Overflow)?
                    .get_big_uint()
                    .checked_mul(&dp.get_big_uint())
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Underflow)?;

                let _denom = u
                    .get_big_uint()
                    .checked_add(&l.get_big_uint())
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Underflow)?;

                d = m
                    .get_big_uint()
                    .checked_mul(&d.get_big_uint())
                    .and_then(|r| r.checked_div(&_denom.get_big_uint()))
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
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

            log::trace!(
                target: "stableSwap::do_get_delta",
                "asset_in: {:?}, asset_out: {:?}, delta: {:?}",
                &asset_in,
                &asset_out,
                &d
            );

            Ok(d)
        }

        pub(crate) fn do_get_alternative_var(
            mut autonomous_var: BalanceOf<T, I>,
            (asset_in, asset_out): (AssetIdOf<T, I>, AssetIdOf<T, I>),
        ) -> Result<Balance, DispatchError> {
            let (resx, _) = T::AMM::get_reserves(asset_in, asset_out).unwrap();
            autonomous_var = autonomous_var
                .get_big_uint()
                .checked_add(&resx.get_big_uint())
                .ok_or(Error::<T, I>::ConversionToU128Failed)?
                .to_u128()
                .ok_or(ArithmeticError::Overflow)?;

            // passes asset in and asset out
            let d = Self::get_d((asset_in, asset_out)).unwrap();

            let mut c = d;
            let mut s = 0u128;

            let a = (T::AmplificationCoefficient::get() as u128)
                .get_big_uint()
                .checked_mul(&T::Precision::get().get_big_uint())
                .ok_or(Error::<T, I>::ConversionToU128Failed)?
                .to_u128()
                .ok_or(ArithmeticError::Underflow)?;

            let n_a = (T::NumTokens::get() as u128)
                .get_big_uint()
                .checked_mul(&a.get_big_uint())
                .ok_or(Error::<T, I>::ConversionToU128Failed)?
                .to_u128()
                .ok_or(ArithmeticError::Underflow)?;

            let _x = 0u128;

            s = s
                .get_big_uint()
                .checked_add(&autonomous_var.get_big_uint())
                .ok_or(Error::<T, I>::ConversionToU128Failed)?
                .to_u128()
                .ok_or(ArithmeticError::Underflow)?;

            c = (c.get_big_uint().checked_mul(&d.get_big_uint()))
                .ok_or(Error::<T, I>::ConversionToU128Failed)?
                .to_u128()
                .ok_or(ArithmeticError::Underflow)?
                .checked_div(
                    autonomous_var
                        .get_big_uint()
                        .checked_mul(&(T::NumTokens::get() as u128).get_big_uint())
                        .ok_or(Error::<T, I>::ConversionToU128Failed)?
                        .to_u128()
                        .ok_or(ArithmeticError::Underflow)?,
                )
                .ok_or(Error::<T, I>::ConversionToU128Failed)?
                .to_u128()
                .ok_or(ArithmeticError::Underflow)?;

            c = (c
                .get_big_uint()
                .checked_mul(&d.get_big_uint())
                .and_then(|r| r.checked_mul(&T::Precision::get().get_big_uint())))
            .ok_or(Error::<T, I>::ConversionToU128Failed)?
            .to_u128()
            .ok_or(ArithmeticError::Underflow)?
            .checked_div(
                n_a.get_big_uint()
                    .checked_mul(&(T::NumTokens::get() as u128).get_big_uint())
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Underflow)?,
            )
            .ok_or(Error::<T, I>::ConversionToU128Failed)?
            .to_u128()
            .ok_or(ArithmeticError::Underflow)?;

            let b = s
                .get_big_uint()
                .checked_add(
                    &d.get_big_uint()
                        .checked_mul(&T::Precision::get().get_big_uint())
                        .and_then(|r| r.checked_div(&n_a.get_big_uint()))
                        .ok_or(Error::<T, I>::ConversionToU128Failed)?,
                )
                .ok_or(Error::<T, I>::ConversionToU128Failed)?
                .to_u128()
                .ok_or(ArithmeticError::Underflow)?;

            let mut y_prev;
            let mut y = d;
            // 255 is a max number of loops
            // should throw error if does not converge
            for _ in 0..255 {
                y_prev = y;

                y = (y
                    .get_big_uint()
                    .checked_mul(&y.get_big_uint())
                    .and_then(|r| r.checked_add(&c.get_big_uint()))
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Underflow)?)
                .checked_div(
                    y.get_big_uint()
                        .checked_mul(&2u128.get_big_uint())
                        .and_then(|r| r.checked_add(&b.get_big_uint()))
                        .and_then(|r| r.checked_sub(&d.get_big_uint()))
                        .ok_or(Error::<T, I>::ConversionToU128Failed)?
                        .to_u128()
                        .ok_or(ArithmeticError::Underflow)?,
                )
                .ok_or(Error::<T, I>::ConversionToU128Failed)?
                .to_u128()
                .ok_or(ArithmeticError::Underflow)?;

                if y.eq(&y_prev) {
                    break;
                }
            }

            Ok(y)
            // throw new Error('Approximation did not converge')
        }
    }
}

// For Parallel Router
impl<T: Config<I>, I: 'static>
    primitives::StableSwap<AccountIdOf<T>, AssetIdOf<T, I>, BalanceOf<T, I>> for Pallet<T, I>
{
    fn get_d(pair: (AssetIdOf<T, I>, AssetIdOf<T, I>)) -> Result<Balance, DispatchError> {
        let d = Self::do_get_delta(pair)?;
        Ok(d)
    }

    fn get_alternative_var(
        autonomous_var: BalanceOf<T, I>,
        pair: (AssetIdOf<T, I>, AssetIdOf<T, I>),
    ) -> Result<Balance, DispatchError> {
        let alternative_var = Self::do_get_alternative_var(autonomous_var, pair)?;
        Ok(alternative_var)
    }
}
