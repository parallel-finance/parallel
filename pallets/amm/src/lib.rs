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

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod types;

mod benchmarking;

pub mod weights;

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    require_transactional,
    traits::{
        fungibles::{Inspect, Mutate, Transfer},
        Get, IsType,
    },
    transactional, Blake2_128Concat, PalletId,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};

use sp_runtime::{
    traits::{AccountIdConversion, CheckedAdd, CheckedSub, IntegerSquareRoot, One, Zero},
    ArithmeticError, DispatchError, Perbill,
};
use sp_std::{cmp::min, ops::Div, result::Result};

pub use pallet::*;

use primitives::{Balance, CurrencyId};
use types::Pool;
pub use weights::WeightInfo;

pub type AssetIdOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type BalanceOf<T, I = ()> =
    <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

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

        #[pallet::constant]
        type LockAccountId: Get<Self::AccountId>;

        /// Weight information for extrinsics in this pallet.
        type AMMWeightInfo: WeightInfo;

        /// Specify which origin is allowed to create new pools.
        type CreatePoolOrigin: EnsureOrigin<Self::Origin>;

        /// Defines the fees taken out of each trade and sent back to the AMM pool,
        /// typically 0.3%.
        #[pallet::constant]
        type LpFee: Get<Perbill>;

        /// How much the protocol is taking out of each trade.
        #[pallet::constant]
        type ProtocolFee: Get<Perbill>;

        /// Minimum amount of liquidty needed to init a new pool
        /// this amount is burned when the pool is created.
        ///
        /// It's important that we include this value in order to
        /// prevent attacks where a bad actor will create and
        /// remove pools with malious intentions. By requiring
        /// a `MinimumLiquidity`, a pool cannot be removed since
        /// a small amount of tokens are locked forever when liquidity
        /// is first added.
        #[pallet::constant]
        type MinimumLiquidity: Get<BalanceOf<Self, I>>;

        /// Who/where to send the protocol fees
        #[pallet::constant]
        type ProtocolFeeReceiver: Get<Self::AccountId>;
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Pool does not exist
        PoolDoesNotExist,
        /// Insufficient liquidity
        InsufficientLiquidity,
        /// Not an ideal price ratio
        NotAnIdealPrice,
        /// Pool does not exist
        PoolAlreadyExists,
        /// Insufficient amount out
        InsufficientAmountOut,
        /// Insufficient amount in
        InsufficientAmountIn,
        /// Identical assets
        IdenticalAssets,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// Add liquidity into pool
        /// [sender, base_currency_id, quote_currency_id, base_amount, quote_amount]
        LiquidityAdded(
            T::AccountId,
            AssetIdOf<T, I>,
            AssetIdOf<T, I>,
            BalanceOf<T, I>,
            BalanceOf<T, I>,
        ),
        /// Remove liquidity from pool
        /// [sender, base_currency_id, quote_currency_id, liquidity]
        LiquidityRemoved(
            T::AccountId,
            AssetIdOf<T, I>,
            AssetIdOf<T, I>,
            BalanceOf<T, I>,
        ),
        /// Trade using liquidity
        /// [trader, currency_id_in, currency_id_out, amount_in, amount_out]
        Traded(
            T::AccountId,
            AssetIdOf<T, I>,
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
        Pool<AssetIdOf<T, I>, BalanceOf<T, I>>,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Allow users to add liquidity to a given pool
        ///
        /// - `pool`: Currency pool, in which liquidity will be added
        /// - `liquidity_amounts`: Liquidity amounts to be added in pool
        /// - `minimum_amounts`: specifying its "worst case" ratio when pool already exists
        #[pallet::weight(T::AMMWeightInfo::add_liquidity())]
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

                    Self::do_add_liquidity(
                        &who,
                        pool,
                        (ideal_base_amount, ideal_quote_amount),
                        (base_asset, quote_asset),
                    )?;

                    Self::do_mint_protocol_fee(pool)?;

                    Self::deposit_event(Event::<T, I>::LiquidityAdded(
                        who,
                        base_asset,
                        quote_asset,
                        ideal_base_amount,
                        ideal_quote_amount,
                    ));

                    Ok(().into())
                },
            )
        }

        /// Allow users to remove liquidity from a given pool
        ///
        /// - `pair`: Currency pool, in which liquidity will be removed
        /// - `liquidity`: liquidity to be removed from user's liquidity
        #[pallet::weight(T::AMMWeightInfo::remove_liquidity())]
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
                Self::do_remove_liquidity(&who, pool, liquidity, (base_asset, quote_asset))?;
                Self::do_mint_protocol_fee(pool)?;

                Self::deposit_event(Event::<T, I>::LiquidityRemoved(
                    who,
                    base_asset,
                    quote_asset,
                    liquidity,
                ));

                Ok(())
            })
        }

        /// Create of a new pool, governance only
        ///
        /// - `pool`: Currency pool, in which liquidity will be added
        /// - `liquidity_amounts`: Liquidity amounts to be added in pool
        /// - `lptoken_receiver`: Allocate any liquidity tokens to lptoken_receiver
        /// - `lp_token_id`: Liquidity pool share representive token
        #[pallet::weight(T::AMMWeightInfo::create_pool())]
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

            let mut pool = Pool::new(lp_token_id);

            Self::do_add_liquidity(
                &lptoken_receiver,
                &mut pool,
                (base_amount, quote_amount),
                (base_asset, quote_asset),
            )?;

            Pools::<T, I>::insert(&base_asset, &quote_asset, pool);

            Self::deposit_event(Event::<T, I>::LiquidityAdded(
                lptoken_receiver,
                base_asset,
                quote_asset,
                base_amount,
                quote_amount,
            ));

            Ok(().into())
        }
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    pub fn lock_account_id() -> T::AccountId {
        T::LockAccountId::get()
    }

    fn quote(
        base_amount: BalanceOf<T, I>,
        base_pool: BalanceOf<T, I>,
        quote_pool: BalanceOf<T, I>,
    ) -> Result<BalanceOf<T, I>, DispatchError> {
        Ok(base_amount
            .checked_mul(quote_pool)
            .and_then(|r| r.checked_div(base_pool))
            .ok_or(ArithmeticError::Underflow)?)
    }

    fn sort_assets(
        (curr_a, curr_b): (AssetIdOf<T, I>, AssetIdOf<T, I>),
    ) -> Result<(bool, AssetIdOf<T, I>, AssetIdOf<T, I>), DispatchError> {
        if curr_a > curr_b {
            Ok((false, curr_a, curr_b))
        } else if curr_a < curr_b {
            Ok((true, curr_b, curr_a))
        } else {
            Err(Error::<T, I>::IdenticalAssets.into())
        }
    }

    fn get_ideal_amounts(
        pool: &Pool<AssetIdOf<T, I>, BalanceOf<T, I>>,
        (base_amount, quote_amount): (BalanceOf<T, I>, BalanceOf<T, I>),
    ) -> Result<(BalanceOf<T, I>, BalanceOf<T, I>), DispatchError> {
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

    fn get_protocol_fee_reciprocal_proportion() -> Result<BalanceOf<T, I>, DispatchError> {
        Ok(T::ProtocolFee::get()
            .checked_add(&T::LpFee::get())
            .map(|r| T::ProtocolFee::get().div(r))
            .map(|r| r.saturating_reciprocal_mul::<BalanceOf<T, I>>(One::one()))
            .ok_or(ArithmeticError::Underflow)?)
    }

    // given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset
    //
    // reserveIn * reserveOut = (reserveIn + amountIn) * (reserveOut - amountOut)
    // reserveIn * reserveOut = reserveIn * reserveOut + amountIn * reserveOut - (reserveIn + amountIn) * amountOut
    // amountIn * reserveOut = (reserveIn + amountIn) * amountOut
    //
    // amountOut = amountIn * reserveOut / (reserveIn + amountIn)
    // amountIn  = amountIn * (1 - fee_percent)
    fn get_amount_out(
        amount_in: BalanceOf<T, I>,
        reserve_in: BalanceOf<T, I>,
        reserve_out: BalanceOf<T, I>,
    ) -> Result<BalanceOf<T, I>, DispatchError> {
        let lp_fees = T::LpFee::get().mul_floor(amount_in);
        let protocol_fees = T::ProtocolFee::get().mul_floor(amount_in);
        let fees = lp_fees
            .checked_add(protocol_fees)
            .ok_or(ArithmeticError::Overflow)?;

        let amount_in = amount_in
            .checked_sub(fees)
            .ok_or(ArithmeticError::Underflow)?;
        let numerator = amount_in
            .checked_sub(reserve_out)
            .ok_or(ArithmeticError::Overflow)?;

        let denominator = reserve_in
            .checked_add(amount_in)
            .ok_or(ArithmeticError::Overflow)?;

        let amount_out = numerator
            .checked_div(denominator)
            .ok_or(ArithmeticError::Underflow)?;

        Ok(amount_out)
    }

    // given an output amount of an asset and pair reserves, returns a required input amount of the other asset
    //
    // amountOut = amountIn * reserveOut / reserveIn + amountIn
    // amountOut * reserveIn + amountOut * amountIn  = amountIn * reserveOut
    // amountOut * reserveIn = amountIn * (reserveOut - amountOut)
    //
    // amountIn = amountOut * reserveIn / (reserveOut - amountOut)
    // amountIn = amountIn / (1 - fee_percent)
    #[allow(dead_code)]
    fn get_amount_in(
        amount_out: BalanceOf<T, I>,
        reserve_in: BalanceOf<T, I>,
        reserve_out: BalanceOf<T, I>,
    ) -> Result<BalanceOf<T, I>, DispatchError> {
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
            .and_then(|r| Perbill::from_percent(100).checked_sub(&r))
            .ok_or(ArithmeticError::Underflow)?;

        Ok(fee_percent
            .saturating_reciprocal_mul(amount_in)
            .checked_add(One::one())
            .ok_or(ArithmeticError::Overflow)?)
    }

    #[require_transactional]
    fn do_add_liquidity(
        who: &T::AccountId,
        pool: &mut Pool<AssetIdOf<T, I>, BalanceOf<T, I>>,
        (ideal_base_amount, ideal_quote_amount): (BalanceOf<T, I>, BalanceOf<T, I>),
        (base_asset, quote_asset): (AssetIdOf<T, I>, AssetIdOf<T, I>),
    ) -> Result<(), DispatchError> {
        let total_supply = T::Assets::total_issuance(pool.lp_token_id);
        let liquidity = if total_supply.is_zero() {
            T::Assets::mint_into(
                pool.lp_token_id,
                &Self::lock_account_id(),
                T::MinimumLiquidity::get(),
            )?;

            ideal_base_amount
                .checked_mul(ideal_quote_amount)
                .map(|r| r.integer_sqrt())
                .and_then(|r| r.checked_sub(T::MinimumLiquidity::get()))
                .ok_or(ArithmeticError::Underflow)?
        } else {
            min(
                ideal_base_amount
                    .checked_mul(total_supply)
                    .and_then(|r| r.checked_div(pool.base_amount))
                    .ok_or(ArithmeticError::Overflow)?,
                ideal_quote_amount
                    .checked_mul(total_supply)
                    .and_then(|r| r.checked_div(pool.quote_amount))
                    .ok_or(ArithmeticError::Overflow)?,
            )
        };

        T::Assets::mint_into(pool.lp_token_id, &who, liquidity)?;

        pool.base_amount = pool
            .base_amount
            .checked_add(ideal_base_amount)
            .ok_or(ArithmeticError::Overflow)?;
        pool.quote_amount = pool
            .quote_amount
            .checked_add(ideal_quote_amount)
            .ok_or(ArithmeticError::Overflow)?;
        T::Assets::transfer(
            base_asset,
            &who,
            &Self::account_id(),
            ideal_base_amount,
            true,
        )?;
        T::Assets::transfer(
            quote_asset,
            &who,
            &Self::account_id(),
            ideal_quote_amount,
            true,
        )?;

        Ok(())
    }

    #[require_transactional]
    fn do_remove_liquidity(
        who: &T::AccountId,
        pool: &mut Pool<AssetIdOf<T, I>, BalanceOf<T, I>>,
        liquidity: BalanceOf<T, I>,
        (base_asset, quote_asset): (AssetIdOf<T, I>, AssetIdOf<T, I>),
    ) -> Result<(BalanceOf<T, I>, BalanceOf<T, I>), DispatchError> {
        let total_supply = T::Assets::total_issuance(pool.lp_token_id);

        let base_amount = liquidity
            .checked_mul(pool.base_amount)
            .and_then(|r| r.checked_div(total_supply))
            .ok_or(ArithmeticError::Underflow)?;

        let quote_amount = liquidity
            .checked_mul(pool.quote_amount)
            .and_then(|r| r.checked_div(total_supply))
            .ok_or(ArithmeticError::Underflow)?;

        pool.base_amount = pool
            .base_amount
            .checked_sub(base_amount)
            .ok_or(ArithmeticError::Underflow)?;

        pool.quote_amount = pool
            .quote_amount
            .checked_sub(quote_amount)
            .ok_or(ArithmeticError::Underflow)?;

        T::Assets::burn_from(pool.lp_token_id, &who, liquidity)?;
        T::Assets::transfer(base_asset, &Self::account_id(), &who, base_amount, false)?;
        T::Assets::transfer(quote_asset, &Self::account_id(), &who, quote_amount, false)?;

        Ok((base_amount, quote_amount))
    }

    #[require_transactional]
    pub fn do_mint_protocol_fee(
        pool: &mut Pool<AssetIdOf<T, I>, BalanceOf<T, I>>,
    ) -> Result<BalanceOf<T, I>, DispatchError> {
        // TODO: If we turn off protocol_fee later in runtime upgrade
        // this will reset root_k_last to zero which may not be good
        if !Self::protocol_fee_on() || pool.root_k_last.is_zero() {
            if !pool.root_k_last.is_zero() {
                pool.root_k_last = Zero::zero();
            }
            return Ok(Zero::zero());
        }

        let root_k = pool
            .base_amount
            .checked_mul(pool.quote_amount)
            .map(|r| r.integer_sqrt())
            .ok_or(ArithmeticError::Overflow)?;

        if root_k <= pool.root_k_last {
            return Ok(Zero::zero());
        }

        let total_supply = T::Assets::total_issuance(pool.lp_token_id);

        let numerator = root_k
            .checked_sub(pool.root_k_last)
            .and_then(|r| r.checked_mul(total_supply))
            .ok_or(ArithmeticError::Overflow)?;

        let denominator = root_k
            .checked_mul(Self::get_protocol_fee_reciprocal_proportion()?)
            .and_then(|r| r.checked_add(pool.root_k_last))
            .ok_or(ArithmeticError::Overflow)?;

        let protocol_fees = numerator
            .checked_div(denominator)
            .ok_or(ArithmeticError::Underflow)?;

        T::Assets::mint_into(
            pool.lp_token_id,
            &T::ProtocolFeeReceiver::get(),
            protocol_fees,
        )?;

        pool.root_k_last = root_k;

        Ok(protocol_fees)
    }

    #[require_transactional]
    fn do_trade(
        who: &T::AccountId,
        (asset_in, asset_out): (AssetIdOf<T, I>, AssetIdOf<T, I>),
        amount_in: BalanceOf<T, I>,
        minimum_amount_out: BalanceOf<T, I>,
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
                    amount_in >= T::LpFee::get().saturating_reciprocal_mul(One::one()),
                    Error::<T, I>::InsufficientAmountIn
                );
                ensure!(!supply_out.is_zero(), Error::<T, I>::InsufficientAmountOut);

                let amount_out = Self::get_amount_out(amount_in, supply_in, supply_out)?;

                ensure!(
                    amount_out >= minimum_amount_out,
                    Error::<T, I>::NotAnIdealPrice
                );

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

                T::Assets::transfer(asset_in, who, &Self::account_id(), amount_in, true)?;
                T::Assets::transfer(asset_out, &Self::account_id(), who, amount_out, false)?;

                Self::deposit_event(Event::<T, I>::Traded(
                    who.clone(),
                    asset_in,
                    asset_out,
                    amount_in,
                    amount_out,
                ));

                Ok(amount_out)
            },
        )
    }
}

impl<T: Config<I>, I: 'static> primitives::AMM<T, AssetIdOf<T, I>, BalanceOf<T, I>>
    for Pallet<T, I>
{
    fn trade(
        who: &T::AccountId,
        pair: (AssetIdOf<T, I>, AssetIdOf<T, I>),
        amount_in: BalanceOf<T, I>,
        minimum_amount_out: BalanceOf<T, I>,
    ) -> Result<BalanceOf<T, I>, DispatchError> {
        Self::do_trade(who, pair, amount_in, minimum_amount_out)
    }
}
