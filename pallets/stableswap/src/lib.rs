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
pub use pallet::*;
use types::Pool;
extern crate alloc;

mod helpers;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod types;
pub mod weights;

use frame_support::{
    log,
    pallet_prelude::*,
    require_transactional,
    traits::{
        fungibles::{Inspect, Mutate, Transfer},
        Get, IsType,
    },
    transactional, Blake2_128Concat, PalletId,
};

use pallet_traits::ConvertToBigUint;
use primitives::{Balance, CurrencyId, Ratio};
use sp_runtime::{
    traits::{AccountIdConversion, CheckedAdd, CheckedSub, One, Saturating, Zero},
    ArithmeticError, DispatchError, FixedPointNumber, FixedU128, SaturatedConversion,
};
use sp_std::{cmp::min, ops::Div, result::Result, vec::Vec};

use crate::helpers::{compute_base, compute_d};
use num_traits::{CheckedDiv, CheckedMul, ToPrimitive};

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type AssetIdOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type BalanceOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::ensure;
    use frame_support::pallet_prelude::DispatchResultWithPostInfo;
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};

    pub type Amounts<T, I> = sp_std::vec::Vec<BalanceOf<T, I>>;

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        type RuntimeEvent: From<Event<Self, I>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        type WeightInfo: WeightInfo;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        #[pallet::constant]
        type ProtocolFeeReceiver: Get<Self::AccountId>;

        #[pallet::constant]
        type LpFee: Get<Ratio>;

        #[pallet::constant]
        type LockAccountId: Get<Self::AccountId>;

        /// How much the protocol is taking out of each trade.
        #[pallet::constant]
        type ProtocolFee: Get<Ratio>;

        #[pallet::constant]
        type MinimumLiquidity: Get<BalanceOf<Self, I>>;

        #[pallet::constant]
        type NumTokens: Get<u8>;

        /// Precision
        #[pallet::constant]
        type Precision: Get<u128>;

        /// Optimal Amplification Coefficient
        #[pallet::constant]
        type AmplificationCoefficient: Get<u8>;

        /// Specify which origin is allowed to create new pools.
        type CreatePoolOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Pool does not exist
        PoolDoesNotExist,
        /// Conversion failure to u128
        ConversionToU128Failed,
        /// Insufficient supply out.
        InsufficientSupplyOut,
        /// Insufficient liquidity
        InsufficientLiquidity,
        /// Insufficient amount out
        InsufficientAmountOut,
        /// Insufficient amount in
        InsufficientAmountIn,
        /// Invariant Error
        InvalidInvariant,
        /// LP token has already been minted
        LpTokenAlreadyExists,
        /// Pool does not exist
        PoolAlreadyExists,
        /// Identical assets
        IdenticalAssets,
        /// Not an ideal price ratio
        NotAnIdealPrice,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// Delta Calculated
        DeltaCalculated(AssetIdOf<T, I>, AssetIdOf<T, I>, u128),

        Traded(
            T::AccountId,
            AssetIdOf<T, I>,
            AssetIdOf<T, I>,
            BalanceOf<T, I>,
            BalanceOf<T, I>,
            AssetIdOf<T, I>,
            BalanceOf<T, I>,
            BalanceOf<T, I>,
        ),
        PoolCreated(
            T::AccountId,
            AssetIdOf<T, I>,
            AssetIdOf<T, I>,
            AssetIdOf<T, I>,
        ),
        /// Add liquidity into pool
        /// [sender, base_currency_id, quote_currency_id, base_amount_added, quote_amount_added, lp_token_id, new_base_amount, new_quote_amount]
        LiquidityAdded(
            T::AccountId,
            AssetIdOf<T, I>,
            AssetIdOf<T, I>,
            BalanceOf<T, I>,
            BalanceOf<T, I>,
            AssetIdOf<T, I>,
            BalanceOf<T, I>,
            BalanceOf<T, I>,
        ),
        /// Remove liquidity from pool
        /// [sender, base_currency_id, quote_currency_id, liquidity, base_amount_removed, quote_amount_removed, lp_token_id, new_base_amount, new_quote_amount]
        LiquidityRemoved(
            T::AccountId,
            AssetIdOf<T, I>,
            AssetIdOf<T, I>,
            BalanceOf<T, I>,
            BalanceOf<T, I>,
            BalanceOf<T, I>,
            AssetIdOf<T, I>,
            BalanceOf<T, I>,
            BalanceOf<T, I>,
        ),
    }

    #[pallet::pallet]
    pub struct Pallet<T, I = ()>(_);

    /// A bag of liquidity composed by two different assets
    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetIdOf<T, I>,
        Blake2_128Concat,
        AssetIdOf<T, I>,
        Pool<AssetIdOf<T, I>, BalanceOf<T, I>, T::BlockNumber>,
        OptionQuery,
    >;

    // No Extrinsic Calls
    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        // Allow users to add liquidity to a given pool
        ///
        /// - `pool`: Currency pool, in which liquidity will be added
        /// - `liquidity_amounts`: Liquidity amounts to be added in pool
        /// - `minimum_amounts`: specifying its "worst case" ratio when pool already exists
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::add_liquidity())]
        #[transactional]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            pair: (AssetIdOf<T, I>, AssetIdOf<T, I>),
            desired_amounts: (BalanceOf<T, I>, BalanceOf<T, I>),
            minimum_amounts: (BalanceOf<T, I>, BalanceOf<T, I>),
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let (is_inverted, base_asset, quote_asset) = Self::sort_assets(pair)?;

            let (base_amount, quote_amount) = if is_inverted {
                (desired_amounts.1, desired_amounts.0)
            } else {
                (desired_amounts.0, desired_amounts.1)
            };

            let (minimum_base_amount, minimum_quote_amount) = if is_inverted {
                (minimum_amounts.1, minimum_amounts.0)
            } else {
                (minimum_amounts.0, minimum_amounts.1)
            };

            Pools::<T, I>::try_mutate(
                base_asset,
                quote_asset,
                |pool| -> DispatchResultWithPostInfo {
                    let pool = pool.as_mut().ok_or(Error::<T, I>::PoolDoesNotExist)?;

                    let (ideal_base_amount, ideal_quote_amount) =
                        Self::get_ideal_amounts(pool, (base_amount, quote_amount))?;

                    ensure!(
                        ideal_base_amount <= base_amount && ideal_quote_amount <= quote_amount,
                        Error::<T, I>::InsufficientAmountIn
                    );

                    ensure!(
                        ideal_base_amount >= minimum_base_amount
                            && ideal_quote_amount >= minimum_quote_amount,
                        Error::<T, I>::NotAnIdealPrice
                    );

                    Self::do_mint_protocol_fee(pool)?;

                    // Adds liquidity
                    Self::do_add_liquidity(
                        &who,
                        pool,
                        (ideal_base_amount, ideal_quote_amount),
                        (base_asset, quote_asset),
                    )?;

                    log::trace!(
                        target: "stableswap::add_liquidity",
                        "who: {:?}, base_asset: {:?}, quote_asset: {:?}, ideal_amounts: {:?},\
                        desired_amounts: {:?}, minimum_amounts: {:?}",
                        &who,
                        &base_asset,
                        &quote_asset,
                        &(ideal_base_amount, ideal_quote_amount),
                        &desired_amounts,
                        &minimum_amounts
                    );

                    Self::deposit_event(Event::<T, I>::LiquidityAdded(
                        who,
                        base_asset,
                        quote_asset,
                        ideal_base_amount,
                        ideal_quote_amount,
                        pool.lp_token_id,
                        pool.base_amount,
                        pool.quote_amount,
                    ));

                    Ok(().into())
                },
            )
        }
        /// Allow users to remove liquidity from a given pool
        ///
        /// - `pair`: Currency pool, in which liquidity will be removed
        /// - `liquidity`: liquidity to be removed from user's liquidity
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::remove_liquidity())]
        #[transactional]
        pub fn remove_liquidity(
            origin: OriginFor<T>,
            pair: (AssetIdOf<T, I>, AssetIdOf<T, I>),
            #[pallet::compact] liquidity: BalanceOf<T, I>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let (_, base_asset, quote_asset) = Self::sort_assets(pair)?;

            Pools::<T, I>::try_mutate(base_asset, quote_asset, |pool| -> DispatchResult {
                let pool = pool.as_mut().ok_or(Error::<T, I>::PoolDoesNotExist)?;

                Self::do_mint_protocol_fee(pool)?;

                let (base_amount_removed, quote_amount_removed) =
                    Self::do_remove_liquidity(&who, pool, liquidity, (base_asset, quote_asset))?;

                log::trace!(
                    target: "stableswap::remove_liquidity",
                    "who: {:?}, base_asset: {:?}, quote_asset: {:?}, liquidity: {:?}",
                    &who,
                    &base_asset,
                    &quote_asset,
                    &liquidity
                );

                Self::deposit_event(Event::<T, I>::LiquidityRemoved(
                    who,
                    base_asset,
                    quote_asset,
                    liquidity,
                    base_amount_removed,
                    quote_amount_removed,
                    pool.lp_token_id,
                    pool.base_amount,
                    pool.quote_amount,
                ));

                Ok(())
            })
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::create_pool())]
        #[transactional]
        pub fn create_pool(
            origin: OriginFor<T>,
            pair: (AssetIdOf<T, I>, AssetIdOf<T, I>),
            liquidity_amounts: (BalanceOf<T, I>, BalanceOf<T, I>),
            lptoken_receiver: T::AccountId,
            lp_token_id: AssetIdOf<T, I>,
        ) -> DispatchResultWithPostInfo {
            T::CreatePoolOrigin::ensure_origin(origin)?;

            let (is_inverted, base_asset, quote_asset) = Self::sort_assets(pair)?;
            ensure!(
                !Pools::<T, I>::contains_key(&base_asset, &quote_asset),
                Error::<T, I>::PoolAlreadyExists
            );

            let (base_amount, quote_amount) = if is_inverted {
                (liquidity_amounts.1, liquidity_amounts.0)
            } else {
                (liquidity_amounts.0, liquidity_amounts.1)
            };

            // check that this is a new asset to avoid using an asset that
            // already has tokens minted
            frame_support::ensure!(
                T::Assets::total_issuance(lp_token_id).is_zero(),
                Error::<T, I>::LpTokenAlreadyExists
            );

            let mut pool = Pool::new(lp_token_id);

            Self::deposit_event(Event::<T, I>::PoolCreated(
                lptoken_receiver.clone(),
                base_asset,
                quote_asset,
                lp_token_id,
            ));

            Self::do_add_liquidity(
                &lptoken_receiver,
                &mut pool,
                (base_amount, quote_amount),
                (base_asset, quote_asset),
            )?;

            Pools::<T, I>::insert(&base_asset, &quote_asset, pool);

            log::trace!(
                target: "stableswap::create_pool",
                "lptoken_receiver: {:?}, base_asset: {:?}, quote_asset: {:?}, base_amount: {:?}, quote_amount: {:?},\
                 liquidity_amounts: {:?}",
                &lptoken_receiver,
                &base_asset,
                &quote_asset,
                &base_amount,
                &quote_amount,
                &liquidity_amounts
            );

            Self::deposit_event(Event::<T, I>::LiquidityAdded(
                lptoken_receiver,
                base_asset,
                quote_asset,
                base_amount,
                quote_amount,
                pool.lp_token_id,
                pool.base_amount,
                pool.quote_amount,
            ));

            Ok(().into())
        }
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    // given an output amount and a vector of assets, return a vector of required input
    // amounts to return the expected output amount
    fn get_amounts_in(
        amount_out: BalanceOf<T, I>,
        path: Vec<AssetIdOf<T, I>>,
    ) -> Result<Amounts<T, I>, DispatchError> {
        let mut amounts_in: Amounts<T, I> = Vec::new();
        amounts_in.resize(path.len(), 0u128);
        let amount_len = amounts_in.len();

        amounts_in[amount_len - 1] = amount_out;
        for i in (1..(path.len())).rev() {
            let (reserve_in, reserve_out) = Self::get_reserves(path[i - 1], path[i])?;
            let amount_in = Self::get_amount_in(amounts_in[i], reserve_in, reserve_out)?;
            amounts_in[i - 1] = amount_in;
        }

        Ok(amounts_in)
    }

    fn get_amount_out(
        amount_in: BalanceOf<T, I>,
        pool_base_aum: BalanceOf<T, I>,
        pool_quote_aum: BalanceOf<T, I>,
    ) -> Result<BalanceOf<T, I>, DispatchError> {
        let fees = T::LpFee::get()
            .checked_add(&T::ProtocolFee::get())
            .map(|r| r.mul_floor(amount_in))
            .ok_or(ArithmeticError::Overflow)?;

        let amount_in = amount_in
            .checked_sub(fees)
            .ok_or(ArithmeticError::Underflow)?;

        let amp = T::AmplificationCoefficient::get() as u128;
        // d = 2000000
        // poolbaseamount = 1000000
        // amountin = 997
        // new quote amount = 1000000 + 997
        let d = Self::delta_util(pool_base_aum, pool_quote_aum)?;
        let new_quote_amount = pool_quote_aum
            .checked_add(amount_in)
            .ok_or(ArithmeticError::Overflow)?;

        let new_base_amount = Self::get_base(new_quote_amount, amp, d)?;

        // pool base amount = 1000000
        // new base amount =  999003

        // TODO: Have a check here
        let amount_out = pool_base_aum
            .checked_sub(new_base_amount)
            .ok_or(ArithmeticError::Underflow)?;

        Ok(amount_out)
    }

    // given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset
    //
    // reserveIn * reserveOut = (reserveIn + amountIn) * (reserveOut - amountOut)
    // reserveIn * reserveOut = reserveIn * reserveOut + amountIn * reserveOut - (reserveIn + amountIn) * amountOut
    // amountIn * reserveOut = (reserveIn + amountIn) * amountOut
    //
    // amountOut = amountIn * reserveOut / (reserveIn + amountIn)
    // amountOut  = amountOut * (1 - fee_percent)
    #[allow(dead_code)]
    fn get_amount_out_1(
        amount_in: BalanceOf<T, I>,
        reserve_in: BalanceOf<T, I>,
        reserve_out: BalanceOf<T, I>,
    ) -> Result<BalanceOf<T, I>, DispatchError> {
        let fees = T::LpFee::get()
            .checked_add(&T::ProtocolFee::get())
            .map(|r| r.mul_floor(amount_in))
            .ok_or(ArithmeticError::Overflow)?;

        let amount_in = amount_in
            .checked_sub(fees)
            .ok_or(ArithmeticError::Underflow)?;
        let numerator = amount_in
            .checked_mul(reserve_out)
            .ok_or(ArithmeticError::Overflow)?;

        let denominator = reserve_in
            .checked_add(amount_in)
            .ok_or(ArithmeticError::Overflow)?;

        let amount_out = numerator
            .checked_div(denominator)
            .ok_or(ArithmeticError::Underflow)?;

        log::trace!(
            target: "stableswap::get_amount_out",
            "amount_in: {:?}, reserve_in: {:?}, reserve_out: {:?}, fees: {:?}, numerator: {:?}, denominator: {:?},\
             amount_out: {:?}",
            &amount_in,
            &reserve_in,
            &reserve_out,
            &fees,
            &numerator,
            &denominator,
            &amount_out
        );

        Ok(amount_out)
    }

    fn quote(
        base_amount: BalanceOf<T, I>,
        base_pool: BalanceOf<T, I>,
        quote_pool: BalanceOf<T, I>,
    ) -> Result<BalanceOf<T, I>, DispatchError> {
        log::trace!(
            target: "stableswap::quote",
            "base_amount: {:?}, base_pool: {:?}, quote_pool: {:?}",
            &base_amount,
            &base_pool,
            &quote_pool
        );

        Ok(base_amount
            .checked_mul(quote_pool)
            .and_then(|r| r.checked_div(base_pool))
            .ok_or(ArithmeticError::Underflow)?)
    }

    fn sort_assets(
        (curr_a, curr_b): (AssetIdOf<T, I>, AssetIdOf<T, I>),
    ) -> Result<(bool, AssetIdOf<T, I>, AssetIdOf<T, I>), DispatchError> {
        if curr_a > curr_b {
            return Ok((false, curr_a, curr_b));
        }

        if curr_a < curr_b {
            return Ok((true, curr_b, curr_a));
        }

        log::trace!(
            target: "stableswap::sort_assets",
            "pair: {:?}",
            &(curr_a, curr_b)
        );

        Err(Error::<T, I>::IdenticalAssets.into())
    }

    // Returns liquidity for a given 2 assets
    #[require_transactional]
    fn do_get_liquidity(
        total_supply: BalanceOf<T, I>,
        pool: &mut Pool<AssetIdOf<T, I>, BalanceOf<T, I>, T::BlockNumber>,
        (ideal_base_amount, ideal_quote_amount): (BalanceOf<T, I>, BalanceOf<T, I>),
    ) -> Result<BalanceOf<T, I>, DispatchError> {
        // Extract to different functionality
        let liquidity = if total_supply.is_zero() {
            T::Assets::mint_into(
                pool.lp_token_id,
                &Self::lock_account_id(),
                T::MinimumLiquidity::get(),
            )?;

            ideal_base_amount
                .get_big_uint()
                .checked_mul(&ideal_quote_amount.get_big_uint())
                // loss of precision due to truncated sqrt
                .map(|r| r.sqrt())
                .and_then(|r| r.checked_sub(&T::MinimumLiquidity::get().get_big_uint()))
                .ok_or(Error::<T, I>::ConversionToU128Failed)?
                .to_u128()
                .ok_or(ArithmeticError::Underflow)?
        } else {
            min(
                ideal_base_amount
                    .get_big_uint()
                    .checked_mul(&total_supply.get_big_uint())
                    .and_then(|r| r.checked_div(&pool.base_amount.get_big_uint()))
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Underflow)?,
                ideal_quote_amount
                    .get_big_uint()
                    .checked_mul(&total_supply.get_big_uint())
                    .and_then(|r| r.checked_div(&pool.quote_amount.get_big_uint()))
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Underflow)?,
            )
        };
        Ok(liquidity)
    }

    #[require_transactional]
    fn do_add_liquidity(
        who: &T::AccountId,
        pool: &mut Pool<AssetIdOf<T, I>, BalanceOf<T, I>, T::BlockNumber>,
        (ideal_base_amount, ideal_quote_amount): (BalanceOf<T, I>, BalanceOf<T, I>),
        (base_asset, quote_asset): (AssetIdOf<T, I>, AssetIdOf<T, I>),
    ) -> Result<(), DispatchError> {
        // Initial invariant
        let mut d0 = 0u128;
        let mut d1 = 0u128;
        if Pools::<T, I>::contains_key(&base_asset, &quote_asset) {
            // d0 = Self::do_get_delta((base_asset, quote_asset)).unwrap();
            let (tot_base_amount, tot_quote_amount) =
                Self::get_reserves(base_asset, quote_asset).unwrap();
            d0 = Self::delta_util(tot_base_amount, tot_quote_amount).unwrap()
        }

        let total_supply = T::Assets::total_issuance(pool.lp_token_id);

        // Extract to different functionality
        let mut liquidity =
            Self::do_get_liquidity(total_supply, pool, (ideal_base_amount, ideal_quote_amount))
                .unwrap();

        // update reserves after liquidity calculation
        pool.base_amount = pool
            .base_amount
            .checked_add(ideal_base_amount)
            .ok_or(ArithmeticError::Overflow)?;
        pool.quote_amount = pool
            .quote_amount
            .checked_add(ideal_quote_amount)
            .ok_or(ArithmeticError::Overflow)?;

        let new_base_amount = pool.base_amount;
        let new_quote_amount = pool.quote_amount;

        if Pools::<T, I>::contains_key(&base_asset, &quote_asset) {
            d1 = Self::do_get_delta_on_the_fly((new_base_amount, new_quote_amount)).unwrap();

            ensure!(d1 >= d0, Error::<T, I>::InvalidInvariant);
        }
        // TODO: the following may not required since fee is -> 0
        // let ideal_base_balance = D1.clone() * pool.base_amount / D0.clone();
        // let ideal_base_new_balance = ideal_base_amount;
        //
        // if ideal_base_balance >
        // let ideal_base_new_balance_difference = (ideal_base_balance - ideal_base_new_balance).abs();
        //
        // let ideal_quote_balance = D1.clone() * pool.base_amount / D0;
        // let ideal_quote_new_balance = ideal_quote_balance;
        // let ideal_quote_new_balance_difference =
        //     (ideal_base_balance - ideal_base_new_balance).abs();

        // let D2 = Self::do_get_delta((base_asset, quote_asset)).unwrap();
        // D2 and D1 in here will be the same
        if total_supply > 0 {
            liquidity = liquidity + liquidity * (d1 - d0) / d0;
        } else {
            liquidity += d1;
        }

        T::Assets::mint_into(pool.lp_token_id, who, liquidity)?;

        T::Assets::transfer(
            base_asset,
            who,
            &Self::account_id(),
            ideal_base_amount,
            true,
        )?;
        T::Assets::transfer(
            quote_asset,
            who,
            &Self::account_id(),
            ideal_quote_amount,
            true,
        )?;

        if Self::protocol_fee_on() {
            // we cannot hold k_last for really large values
            // we can hold two u128s instead
            pool.base_amount_last = pool.base_amount;
            pool.quote_amount_last = pool.quote_amount;
        }

        log::trace!(
            target: "stableswap::do_add_liquidity",
            "who: {:?}, total_supply: {:?}, liquidity: {:?}, base_asset: {:?}, quote_asset: {:?}, ideal_base_amount: {:?},\
             ideal_quote_amount: {:?}",
            &who,
            &total_supply,
            &liquidity,
            &base_asset,
            &quote_asset,
            &ideal_base_amount,
            &ideal_quote_amount
        );

        Ok(())
    }

    fn calculate_reserves_to_remove(
        pool: &mut Pool<AssetIdOf<T, I>, BalanceOf<T, I>, T::BlockNumber>,
        liquidity: BalanceOf<T, I>,
    ) -> Result<(BalanceOf<T, I>, BalanceOf<T, I>), DispatchError> {
        let total_supply = T::Assets::total_issuance(pool.lp_token_id);
        let base_amount = liquidity
            .get_big_uint()
            .checked_mul(&pool.base_amount.get_big_uint())
            .and_then(|r| r.checked_div(&total_supply.get_big_uint()))
            .ok_or(Error::<T, I>::ConversionToU128Failed)?
            .to_u128()
            .ok_or(ArithmeticError::Underflow)?;
        let quote_amount = liquidity
            .get_big_uint()
            .checked_mul(&pool.quote_amount.get_big_uint())
            .and_then(|r| r.checked_div(&total_supply.get_big_uint()))
            .ok_or(Error::<T, I>::ConversionToU128Failed)?
            .to_u128()
            .ok_or(ArithmeticError::Underflow)?;

        Ok((base_amount, quote_amount))
    }

    #[require_transactional]
    fn do_remove_liquidity(
        who: &T::AccountId,
        pool: &mut Pool<AssetIdOf<T, I>, BalanceOf<T, I>, T::BlockNumber>,
        liquidity: BalanceOf<T, I>,
        (base_asset, quote_asset): (AssetIdOf<T, I>, AssetIdOf<T, I>),
    ) -> Result<(BalanceOf<T, I>, BalanceOf<T, I>), DispatchError> {
        let (base_amount, quote_amount) = Self::calculate_reserves_to_remove(pool, liquidity)?;

        pool.base_amount = pool
            .base_amount
            .checked_sub(base_amount)
            .ok_or(Error::<T, I>::InsufficientLiquidity)?;

        pool.quote_amount = pool
            .quote_amount
            .checked_sub(quote_amount)
            .ok_or(Error::<T, I>::InsufficientLiquidity)?;

        T::Assets::burn_from(pool.lp_token_id, who, liquidity)?;

        T::Assets::transfer(base_asset, &Self::account_id(), who, base_amount, false)?;
        T::Assets::transfer(quote_asset, &Self::account_id(), who, quote_amount, false)?;

        if Self::protocol_fee_on() {
            // we cannot hold k_last for really large values
            // we can hold two u128s instead
            pool.base_amount_last = pool.base_amount;
            pool.quote_amount_last = pool.quote_amount;
        }

        log::trace!(
            target: "stableswap::do_remove_liquidity",
            "who: {:?}, liquidity: {:?}, base_asset: {:?}, quote_asset: {:?}, base_amount: {:?}, quote_amount: {:?}",
            &who,
            &liquidity,
            &base_asset,
            &quote_asset,
            &base_amount,
            &quote_amount
        );

        Ok((base_amount, quote_amount))
    }

    #[require_transactional]
    pub fn do_mint_protocol_fee(
        pool: &mut Pool<AssetIdOf<T, I>, BalanceOf<T, I>, T::BlockNumber>,
    ) -> Result<BalanceOf<T, I>, DispatchError> {
        // TODO: If we turn off protocol_fee later in runtime upgrade
        // this will reset root_k_last to zero which may not be good
        let k_last = pool
            .base_amount_last
            .get_big_uint()
            .checked_mul(&pool.quote_amount_last.get_big_uint())
            .ok_or(ArithmeticError::Overflow)?;

        if !Self::protocol_fee_on() {
            // if fees are off and k_last is a value we need to reset it
            if !k_last.is_zero() {
                pool.base_amount_last = Zero::zero();
                pool.quote_amount_last = Zero::zero();
                return Ok(Zero::zero());
            }

            // if fees are off and k_last is zero return
            return Ok(Zero::zero());
        }

        let root_k_last = Self::delta_util(pool.base_amount_last, pool.quote_amount_last)
            .unwrap()
            .get_big_uint();

        // if the early exits do not return we know that k_last is not zero
        // and that protocol fees are on

        let root_k = Self::delta_util(pool.base_amount, pool.quote_amount)
            .unwrap()
            .get_big_uint();

        if root_k <= root_k_last {
            return Ok(Zero::zero());
        }

        let total_supply = T::Assets::total_issuance(pool.lp_token_id).get_big_uint();

        let numerator = root_k
            .checked_sub(&root_k_last)
            .and_then(|r| r.checked_mul(&total_supply))
            .ok_or(Error::<T, I>::ConversionToU128Failed)?;

        let scalar = Self::get_protocol_fee_reciprocal_proportion()?
            .checked_sub(One::one())
            .ok_or(ArithmeticError::Underflow)?
            .get_big_uint();

        let denominator = root_k
            .checked_mul(&scalar)
            .and_then(|r| r.checked_add(&root_k_last))
            .ok_or(Error::<T, I>::ConversionToU128Failed)?;

        let protocol_fees = numerator
            // loss of precision due to truncated division
            .checked_div(&denominator)
            .ok_or(ArithmeticError::Underflow)?
            .to_u128()
            .ok_or(ArithmeticError::Overflow)?;

        T::Assets::mint_into(
            pool.lp_token_id,
            &T::ProtocolFeeReceiver::get(),
            protocol_fees,
        )?;

        log::trace!(
            target: "stableswap::do_mint_protocol_fee",
            "root_k: {:?}, total_supply: {:?}, numerator: {:?}, denominator: {:?}, protocol_fees: {:?}",
            &root_k,
            &total_supply,
            &numerator,
            &denominator,
            &protocol_fees
        );
        Ok(protocol_fees)
    }

    fn do_swap(
        who: &T::AccountId,
        (asset_in, asset_out): (AssetIdOf<T, I>, AssetIdOf<T, I>),
        amount_in: BalanceOf<T, I>,
    ) -> Result<BalanceOf<T, I>, DispatchError> {
        let (is_inverted, base_asset, quote_asset) = Self::sort_assets((asset_in, asset_out))?;

        Pools::<T, I>::try_mutate(
            &base_asset,
            &quote_asset,
            |pool| -> Result<BalanceOf<T, I>, DispatchError> {
                let pool = pool.as_mut().ok_or(Error::<T, I>::PoolDoesNotExist)?;

                let (supply_in, supply_out) = if is_inverted {
                    (pool.quote_amount, pool.base_amount)
                } else {
                    (pool.base_amount, pool.quote_amount)
                };

                ensure!(
                    amount_in >= T::LpFee::get().saturating_reciprocal_mul_floor(One::one()),
                    Error::<T, I>::InsufficientAmountIn
                );
                ensure!(!supply_out.is_zero(), Error::<T, I>::InsufficientAmountOut);

                //let amount_out = Self::get_amount_out(amount_in, supply_in, supply_out)?;
                let amount_out = Self::get_amount_out(amount_in, supply_in, supply_out)?;

                let (new_supply_in, new_supply_out) = (
                    supply_in
                        .checked_add(amount_in)
                        .ok_or(ArithmeticError::Overflow)?,
                    supply_out
                        .checked_sub(amount_out)
                        .ok_or(ArithmeticError::Underflow)?,
                );

                if is_inverted {
                    pool.quote_amount = new_supply_in;
                    pool.base_amount = new_supply_out;
                } else {
                    pool.base_amount = new_supply_in;
                    pool.quote_amount = new_supply_out;
                }

                Self::do_update_oracle(pool)?;

                T::Assets::transfer(asset_in, who, &Self::account_id(), amount_in, true)?;
                T::Assets::transfer(asset_out, &Self::account_id(), who, amount_out, false)?;

                log::trace!(
                    target: "stableswap::do_trade",
                    "who: {:?}, asset_in: {:?}, asset_out: {:?}, amount_in: {:?}, amount_out: {:?}",
                    &who,
                    &asset_in,
                    &asset_out,
                    &amount_in,
                    &amount_out,
                );

                Self::deposit_event(Event::<T, I>::Traded(
                    who.clone(),
                    asset_in,
                    asset_out,
                    amount_in,
                    amount_out,
                    pool.lp_token_id,
                    pool.quote_amount,
                    pool.base_amount,
                ));

                Ok(amount_out)
            },
        )
    }
    // https://miguelmota.com/blog/understanding-stableswap-curve/
    // https://github.com/curvefi/curve-contract/blob/master/contracts/pool-templates/base/SwapTemplateBase.vy
    // https://github.com/parallel-finance/amm-formula/blob/master/src/formula.rs
    // https://curve.fi/files/stableswap-paper.pdf
    // Calculates delta based on already persisted assets
    #[allow(dead_code)]
    pub fn do_get_delta(
        (asset_in, asset_out): (AssetIdOf<T, I>, AssetIdOf<T, I>),
    ) -> Result<Balance, DispatchError> {
        let (tot_base_amount, tot_quote_amount) = Self::get_reserves(asset_in, asset_out).unwrap();

        let d = Self::delta_util(tot_base_amount, tot_quote_amount).unwrap();

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

    // Calculates delta based on amounts
    fn delta_util(
        tot_base_amount: BalanceOf<T, I>,
        tot_quote_amount: BalanceOf<T, I>,
    ) -> Result<Balance, DispatchError> {
        let d = compute_d(
            tot_base_amount,
            tot_quote_amount,
            T::AmplificationCoefficient::get() as u128,
        )?;

        Ok(d)
    }

    pub fn get_base(
        new_quote: BalanceOf<T, I>,
        amp_coeff: BalanceOf<T, I>,
        d: BalanceOf<T, I>,
    ) -> Result<Balance, DispatchError> {
        let base = compute_base(new_quote, amp_coeff, d)?;
        Ok(base)
    }

    #[allow(dead_code)]
    // Responsible to get ratio
    fn get_exchange_value(
        pair: (AssetIdOf<T, I>, AssetIdOf<T, I>),
        _asset_id: AssetIdOf<T, I>,
        amount: BalanceOf<T, I>,
    ) -> Result<Balance, DispatchError> {
        let (_, base_asset, quote_asset) = Self::sort_assets(pair)?;
        let pool = Pools::<T, I>::try_get(base_asset, quote_asset)
            .map_err(|_err| Error::<T, I>::PoolDoesNotExist)?;
        let amp = T::AmplificationCoefficient::get() as u128;
        let pool_base_aum = pool.base_amount;
        let pool_quote_aum = pool.quote_amount;
        let d = Self::delta_util(pool_base_aum, pool_quote_aum)?;
        let new_quote_amount = pool_quote_aum
            .checked_add(amount)
            .ok_or(ArithmeticError::Underflow)?;
        let new_base_amount = Self::get_base(new_quote_amount, amp, d)?;
        let exchange_value = pool_base_aum
            .checked_sub(new_base_amount)
            .ok_or(ArithmeticError::Underflow)?;
        Ok(exchange_value)
    }

    // Calculates delta on the fly
    #[allow(dead_code)]
    pub fn do_get_delta_on_the_fly(
        (tot_base_amount, tot_quote_amount): (Balance, Balance),
    ) -> Result<Balance, DispatchError> {
        let d = Self::delta_util(tot_base_amount, tot_quote_amount).unwrap();

        log::trace!(
            target: "stableSwap::do_get_delta_on_the_fly",
            "tot_base_amount: {:?}, tot_quote_amount: {:?}, delta: {:?}",
            &tot_base_amount,
            &tot_quote_amount,
            &d
        );

        Ok(d)
    }

    #[allow(dead_code)]
    pub fn do_get_alternative_var(
        mut autonomous_var: BalanceOf<T, I>,
        (asset_in, asset_out): (AssetIdOf<T, I>, AssetIdOf<T, I>),
    ) -> Result<Balance, DispatchError> {
        let (resx, _) = Self::get_reserves(asset_in, asset_out).unwrap();
        autonomous_var = autonomous_var
            .get_big_uint()
            .checked_add(&resx.get_big_uint())
            .ok_or(Error::<T, I>::ConversionToU128Failed)?
            .to_u128()
            .ok_or(ArithmeticError::Overflow)?;

        // passes asset in and asset out
        let (tot_base_amount, tot_quote_amount) = Self::get_reserves(asset_in, asset_out).unwrap();
        let d = Self::delta_util(tot_base_amount, tot_quote_amount).unwrap();

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
    // extract the reserves from a pool after sorting assets
    fn get_reserves(
        asset_in: AssetIdOf<T, I>,
        asset_out: AssetIdOf<T, I>,
    ) -> Result<(BalanceOf<T, I>, BalanceOf<T, I>), DispatchError> {
        let (is_inverted, base_asset, quote_asset) = Self::sort_assets((asset_in, asset_out))?;

        let pool = Pools::<T, I>::try_get(base_asset, quote_asset)
            .map_err(|_err| Error::<T, I>::PoolDoesNotExist)?;

        if is_inverted {
            Ok((pool.quote_amount, pool.base_amount))
        } else {
            Ok((pool.base_amount, pool.quote_amount))
        }
    }

    // given an output amount of an asset and pair reserves, returns a required input amount of the other asset
    //
    // amountOut = amountIn * reserveOut / reserveIn + amountIn
    // amountOut * reserveIn + amountOut * amountIn  = amountIn * reserveOut
    // amountOut * reserveIn = amountIn * (reserveOut - amountOut)
    //
    // amountIn = amountOut * reserveIn / (reserveOut - amountOut)
    // amountIn = (amountIn / (1 - fee_percent)) + 1
    fn get_amount_in(
        amount_out: BalanceOf<T, I>,
        reserve_in: BalanceOf<T, I>,
        reserve_out: BalanceOf<T, I>,
    ) -> Result<BalanceOf<T, I>, DispatchError> {
        ensure!(
            amount_out < reserve_out,
            Error::<T, I>::InsufficientSupplyOut
        );
        let numerator = reserve_in
            .checked_mul(amount_out)
            .ok_or(ArithmeticError::Overflow)?;

        let denominator = reserve_out
            .checked_sub(amount_out)
            .ok_or(ArithmeticError::Underflow)?;

        let amount_in = numerator
            .checked_div(denominator)
            .ok_or(ArithmeticError::Underflow)?;

        let fee_percent = T::LpFee::get()
            .checked_add(&T::ProtocolFee::get())
            .and_then(|r| Ratio::from_percent(100).checked_sub(&r))
            .ok_or(ArithmeticError::Underflow)?;

        log::trace!(
            target: "stableswap::get_amount_in",
            "amount_out: {:?}, reserve_in: {:?}, reserve_out: {:?}, numerator: {:?}, denominator: {:?}, amount_in: {:?}",
            &amount_out,
            &reserve_in,
            &reserve_out,
            &numerator,
            &denominator,
            &amount_in
        );

        Ok(fee_percent
            .saturating_reciprocal_mul_floor(amount_in)
            .checked_add(One::one())
            .ok_or(ArithmeticError::Overflow)?)
    }

    fn do_update_oracle(
        pool: &mut Pool<AssetIdOf<T, I>, BalanceOf<T, I>, T::BlockNumber>,
    ) -> Result<(), DispatchError> {
        let block_timestamp = frame_system::Pallet::<T>::block_number();

        if pool.block_timestamp_last != block_timestamp {
            let time_elapsed: BalanceOf<T, I> = block_timestamp
                .saturating_sub(pool.block_timestamp_last)
                .saturated_into();

            // compute by multiplying the numerator with the time elapsed
            let price0_fraction = FixedU128::saturating_from_rational(
                time_elapsed
                    .get_big_uint()
                    .checked_mul(&pool.quote_amount.get_big_uint())
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Overflow)?,
                pool.base_amount,
            );
            let price1_fraction = FixedU128::saturating_from_rational(
                time_elapsed
                    .get_big_uint()
                    .checked_mul(&pool.base_amount.get_big_uint())
                    .ok_or(Error::<T, I>::ConversionToU128Failed)?
                    .to_u128()
                    .ok_or(ArithmeticError::Overflow)?,
                pool.quote_amount,
            );

            // convert stored u128 into FixedU128 before add
            pool.price_0_cumulative_last = FixedU128::from_inner(pool.price_0_cumulative_last)
                .checked_add(&price0_fraction)
                .ok_or(ArithmeticError::Overflow)?
                .into_inner();

            pool.price_1_cumulative_last = FixedU128::from_inner(pool.price_1_cumulative_last)
                .checked_add(&price1_fraction)
                .ok_or(ArithmeticError::Overflow)?
                .into_inner();

            // updates timestamp last so `time_elapsed` is correctly calculated
            pool.block_timestamp_last = block_timestamp;
        }

        Ok(())
    }
    // given a pool, calculate the ideal liquidity amounts as a function of the current
    // pool reserves ratio
    fn get_ideal_amounts(
        pool: &Pool<AssetIdOf<T, I>, BalanceOf<T, I>, T::BlockNumber>,
        (base_amount, quote_amount): (BalanceOf<T, I>, BalanceOf<T, I>),
    ) -> Result<(BalanceOf<T, I>, BalanceOf<T, I>), DispatchError> {
        log::trace!(
            target: "stableswap::get_ideal_amounts",
            "pair: {:?}",
            &(base_amount, quote_amount)
        );

        if pool.is_empty() {
            return Ok((base_amount, quote_amount));
        }

        let ideal_quote_amount = Self::quote(base_amount, pool.base_amount, pool.quote_amount)?;
        if ideal_quote_amount <= quote_amount {
            Ok((base_amount, ideal_quote_amount))
        } else {
            let ideal_base_amount = Self::quote(quote_amount, pool.quote_amount, pool.base_amount)?;
            Ok((ideal_base_amount, quote_amount))
        }
    }
    fn protocol_fee_on() -> bool {
        !T::ProtocolFee::get().is_zero()
    }
    pub fn lock_account_id() -> T::AccountId {
        T::LockAccountId::get()
    }
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account_truncating()
    }
    // given an input amount and a vector of assets, return a vector of output
    // amounts
    fn get_amounts_out(
        amount_in: BalanceOf<T, I>,
        path: Vec<AssetIdOf<T, I>>,
    ) -> Result<Amounts<T, I>, DispatchError> {
        let mut amounts_out: Amounts<T, I> = Vec::new();
        amounts_out.resize(path.len(), 0u128);

        amounts_out[0] = amount_in;
        for i in 0..(path.len() - 1) {
            let (reserve_in, reserve_out) = Self::get_reserves(path[i], path[i + 1])?;
            let amount_out = Self::get_amount_out(amounts_out[i], reserve_in, reserve_out)?;
            amounts_out[i + 1] = amount_out;
        }

        Ok(amounts_out)
    }

    fn get_protocol_fee_reciprocal_proportion() -> Result<BalanceOf<T, I>, DispatchError> {
        Ok(T::ProtocolFee::get()
            .checked_add(&T::LpFee::get())
            .map(|r| T::ProtocolFee::get().div(r))
            .map(|r| r.saturating_reciprocal_mul_floor::<BalanceOf<T, I>>(One::one()))
            .ok_or(ArithmeticError::Underflow)?)
    }
}
// For Parallel Router
impl<T: Config<I>, I: 'static>
    pallet_traits::StableSwap<AccountIdOf<T>, AssetIdOf<T, I>, BalanceOf<T, I>> for Pallet<T, I>
{
    /// Based on the path specified and the available pool balances
    /// this will return the amounts outs when trading the specified
    /// amount in
    fn get_amounts_out(
        amount_in: BalanceOf<T, I>,
        path: Vec<AssetIdOf<T, I>>,
    ) -> Result<Vec<BalanceOf<T, I>>, DispatchError> {
        let balances = Self::get_amounts_out(amount_in, path)?;
        Ok(balances)
    }

    /// Based on the path specified and the available pool balances
    /// this will return the amounts in needed to produce the specified
    /// amount out
    fn get_amounts_in(
        amount_out: BalanceOf<T, I>,
        path: Vec<AssetIdOf<T, I>>,
    ) -> Result<Vec<BalanceOf<T, I>>, DispatchError> {
        let balances = Self::get_amounts_in(amount_out, path)?;
        Ok(balances)
    }

    /// Handles a "swap" on the stableswap side for "who".
    /// This will move the `amount_in` funds to the stableswap PalletId,
    /// trade `pair.0` to `pair.1` and return a result with the amount
    /// of currency that was sent back to the user.
    fn swap(
        who: &AccountIdOf<T>,
        pair: (AssetIdOf<T, I>, AssetIdOf<T, I>),
        amount_in: BalanceOf<T, I>,
    ) -> Result<(), DispatchError> {
        Self::do_swap(who, pair, amount_in)?;
        Ok(())
    }

    /// Returns a vector of all of the pools in storage
    fn get_pools() -> Result<Vec<(AssetIdOf<T, I>, AssetIdOf<T, I>)>, DispatchError> {
        Ok(Pools::<T, I>::iter_keys().collect())
    }

    fn get_reserves(
        asset_in: AssetIdOf<T, I>,
        asset_out: AssetIdOf<T, I>,
    ) -> Result<(BalanceOf<T, I>, BalanceOf<T, I>), DispatchError> {
        let (amount_x, amount_y) = Self::get_reserves(asset_in, asset_out)?;
        Ok((amount_x, amount_y))
    }
}
