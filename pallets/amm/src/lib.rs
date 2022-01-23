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
    traits::{AccountIdConversion, CheckedDiv, IntegerSquareRoot, One, Zero},
    ArithmeticError, DispatchError, FixedU128, Perbill, SaturatedConversion,
};
use sp_std::{cmp::min, result::Result};

pub use pallet::*;

use primitives::{Balance, CurrencyId, Rate};
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
        /// [sender, currency_id, currency_id]
        LiquidityAdded(T::AccountId, AssetIdOf<T, I>, AssetIdOf<T, I>),
        /// Remove liquidity from pool
        /// [sender, currency_id, currency_id]
        LiquidityRemoved(T::AccountId, AssetIdOf<T, I>, AssetIdOf<T, I>),
        /// Trade using liquidity
        /// [trader, currency_id_in, currency_id_out, rate_out_for_in]
        Traded(T::AccountId, AssetIdOf<T, I>, AssetIdOf<T, I>, Rate),
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
                        Self::get_ideal_amounts(pool, desired_amounts)?;

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

                    let protocol_fees = Self::get_protocol_fee(
                        base_asset,
                        quote_asset,
                        ideal_base_amount,
                        ideal_quote_amount,
                    )?;

                    T::Assets::transfer(
                        pool.lp_token_id,
                        &who,
                        &T::ProtocolFeeReceiver::get(),
                        protocol_fees,
                        true,
                    )?;

                    Self::deposit_event(Event::<T, I>::LiquidityAdded(
                        who,
                        base_asset,
                        quote_asset,
                    ));

                    Ok(().into())
                },
            )
        }

        /// Allow users to remove liquidity from a given pool
        ///
        /// - `pool`: Currency pool, in which liquidity will be removed
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
                let mut pool = pool.as_mut().ok_or(Error::<T, I>::PoolDoesNotExist)?;
                let total_supply = T::Assets::total_issuance(pool.lp_token_id);

                ensure!(
                    T::Assets::reducible_balance(pool.lp_token_id, &who, false) >= liquidity,
                    Error::<T, I>::InsufficientLiquidity
                );

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

                let protocol_fees =
                    Self::get_protocol_fee(base_asset, quote_asset, base_amount, quote_amount)?;
                T::Assets::transfer(
                    pool.lp_token_id,
                    &who,
                    &T::ProtocolFeeReceiver::get(),
                    protocol_fees,
                    true,
                )?;

                T::Assets::burn_from(pool.lp_token_id, &who, liquidity)?;
                T::Assets::transfer(base_asset, &Self::account_id(), &who, base_amount, false)?;
                T::Assets::transfer(quote_asset, &Self::account_id(), &who, quote_amount, false)?;

                Self::deposit_event(Event::<T, I>::LiquidityRemoved(
                    who,
                    base_asset,
                    quote_asset,
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

            let mut pool = Pool {
                base_amount: 0,
                quote_amount: 0,
                lp_token_id,
            };

            Pools::<T, I>::insert(&base_asset, &quote_asset, pool);

            Self::do_add_liquidity(
                &lptoken_receiver,
                &mut pool,
                (base_amount, quote_amount),
                (base_asset, quote_asset),
            )?;

            Self::deposit_event(Event::<T, I>::LiquidityAdded(
                lptoken_receiver,
                base_asset,
                quote_asset,
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
        if pool.base_amount.is_zero() && pool.quote_amount.is_zero() {
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
    fn burn_transfer_liquidity(
        who: T::AccountId,
        liquidity: BalanceOf<T, I>,
        currency_asset: AssetIdOf<T, I>,
        base_asset: AssetIdOf<T, I>,
        quote_asset: AssetIdOf<T, I>,
        base_amount: BalanceOf<T, I>,
        quote_amount: BalanceOf<T, I>,
    ) -> DispatchResult {
        T::Assets::burn_from(currency_asset, &who, liquidity)?;
        T::Assets::transfer(base_asset, &Self::account_id(), &who, base_amount, false)?;
        T::Assets::transfer(quote_asset, &Self::account_id(), &who, quote_amount, false)?;

        Self::deposit_event(Event::<T, I>::LiquidityRemoved(
            who,
            base_asset,
            quote_asset,
        ));

        Ok(())
    }

    // update reserves and, on the first call per block, price accumulators
    fn _update(
        base_amount: Balance,
        quote_amount: Balance,
        pool: &mut Pool<AssetIdOf<T, I>, BalanceOf<T, I>>,
    ) -> DispatchResult {
        // set values
        pool.base_amount = base_amount;
        pool.quote_amount = quote_amount;

        // TODO:
        // update future pool variables

        Ok(())
    }

    // update pool reserves
    fn update(
        pool: &mut Pool<AssetIdOf<T, I>, BalanceOf<T, I>>,
        amount_in: Balance,
        amount_out: Balance,
        is_inverted: bool,
    ) -> DispatchResult {
        // 5. Update the `Pools` storage to track the `base_amount` and `quote_amount`
        // variables (increase and decrease by `amount_in` and `amount_out`)
        // increase pool.base_amount by amount_in, unless inverted
        if is_inverted {
            let base_amount = pool
                .base_amount
                .checked_sub(amount_out)
                .ok_or(ArithmeticError::Underflow)?;

            let quote_amount = pool
                .quote_amount
                .checked_add(amount_in)
                .ok_or(ArithmeticError::Overflow)?;

            Self::_update(base_amount, quote_amount, pool)?;
        } else {
            let base_amount = pool
                .base_amount
                .checked_add(amount_in)
                .ok_or(ArithmeticError::Overflow)?;

            let quote_amount = pool
                .quote_amount
                .checked_sub(amount_out)
                .ok_or(ArithmeticError::Underflow)?;

            Self::_update(base_amount, quote_amount, pool)?;
        }

        Ok(())
    }

    fn transfer_between_user_and_pallet(
        input_token: AssetIdOf<T, I>,
        output_token: AssetIdOf<T, I>,
        base_asset: AssetIdOf<T, I>,
        quote_asset: AssetIdOf<T, I>,
        who: &T::AccountId,
        amount_in: Balance,
        amount_out: Balance,
    ) -> DispatchResult {
        // 6. Wire amount_in of the input token (identified by pair.0) from who to PalletId
        T::Assets::transfer(input_token, who, &Self::account_id(), amount_in, true)?;

        // 7. Wire amount_out of the output token (identified by pair.1) to who from PalletId
        T::Assets::transfer(output_token, &Self::account_id(), who, amount_out, true)?;

        // Emit event of trade with rate calculated
        Self::deposit_event(Event::<T, I>::Traded(
            who.clone(),
            base_asset,
            quote_asset,
            FixedU128::from_inner(amount_out.saturated_into())
                .checked_div(&FixedU128::from_inner(amount_in.saturated_into()))
                .ok_or(ArithmeticError::Underflow)?,
        ));

        Ok(())
    }

    // given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset
    fn get_amount_out(
        amount_in: Balance,
        reserve_in: Balance,
        reserve_out: Balance,
        fee_percent: Perbill,
    ) -> Result<BalanceOf<T, I>, DispatchError> {
        ensure!(
            amount_in > Zero::zero(),
            Error::<T, I>::InsufficientAmountIn
        );
        ensure!(
            reserve_in > Zero::zero() && reserve_out > Zero::zero(),
            Error::<T, I>::InsufficientAmountIn
        );

        let fee_amount = fee_percent.mul_floor(amount_in);

        let scaler = 1_000u128;
        let numerator_scalar = scaler
            .checked_sub(fee_amount)
            .ok_or(ArithmeticError::Underflow)?;

        let amount_in_with_fee = amount_in
            .checked_mul(numerator_scalar)
            .ok_or(ArithmeticError::Overflow)?;
        let numerator = amount_in_with_fee
            .checked_mul(reserve_out)
            .ok_or(ArithmeticError::Overflow)?;

        let denominator = reserve_in
            .checked_mul(scaler)
            .and_then(|r| r.checked_add(amount_in_with_fee))
            .ok_or(ArithmeticError::Overflow)?;

        let amount_out = numerator
            .checked_div(denominator)
            .ok_or(ArithmeticError::Underflow)?;

        Ok(amount_out)
    }

    // given an output amount of an asset and pair reserves, returns a required input amount of the other asset
    fn _get_amount_in(
        amount_out: Balance,
        reserve_in: Balance,
        reserve_out: Balance,
        fee_percent: Perbill,
    ) -> Result<BalanceOf<T, I>, DispatchError> {
        ensure!(
            amount_out > Zero::zero(),
            Error::<T, I>::InsufficientAmountOut
        );
        ensure!(
            reserve_in > Zero::zero() && reserve_out > Zero::zero(),
            Error::<T, I>::InsufficientAmountIn
        );

        let fee_amount = fee_percent.mul_floor(amount_out);

        let scaler = 1_000u128;
        let denominator_scalar = scaler
            .checked_sub(fee_amount)
            .ok_or(ArithmeticError::Underflow)?;

        let numerator = reserve_in
            .checked_mul(amount_out)
            .and_then(|r| r.checked_mul(scaler))
            .ok_or(ArithmeticError::Overflow)?;

        let denominator = reserve_out
            .checked_sub(amount_out)
            .and_then(|r| r.checked_mul(denominator_scalar))
            .ok_or(ArithmeticError::Underflow)?;

        let amount_in = numerator
            .checked_div(denominator)
            .ok_or(ArithmeticError::Underflow)?;

        Ok(amount_in
            .checked_add(One::one())
            .ok_or(ArithmeticError::Overflow)?)
    }

    pub fn get_protocol_fee(
        base_asset: AssetIdOf<T, I>,
        quote_asset: AssetIdOf<T, I>,
        base_amount: BalanceOf<T, I>,
        quote_amount: BalanceOf<T, I>,
    ) -> Result<BalanceOf<T, I>, DispatchError> {
        let pool =
            Pools::<T, I>::get(&base_asset, &quote_asset).ok_or(Error::<T, I>::PoolDoesNotExist)?;
        let root_k = base_amount
            .checked_mul(quote_amount)
            .map(|r| r.integer_sqrt())
            .ok_or(ArithmeticError::Overflow)?;
        let root_k_last = pool
            .base_amount
            .checked_mul(pool.quote_amount)
            .map(|r| r.integer_sqrt())
            .ok_or(ArithmeticError::Overflow)?;

        if root_k > root_k_last {
            let total_supply: BalanceOf<T, I> = T::Assets::total_issuance(pool.lp_token_id);

            let numerator = root_k
                .checked_sub(root_k_last)
                .and_then(|r| r.checked_mul(total_supply))
                .ok_or(ArithmeticError::Underflow)?;

            let denominator = root_k
                .checked_mul(5)
                .and_then(|r| r.checked_add(root_k_last))
                .ok_or(ArithmeticError::Overflow)?;

            let liquidity = numerator
                .checked_div(denominator)
                .ok_or(ArithmeticError::Underflow)?;

            Ok(liquidity)
        } else {
            Ok(Zero::zero())
        }
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
    ) -> Result<BalanceOf<T, I>, sp_runtime::DispatchError> {
        // expand variables

        // Sort pair to interact with the correct pool.
        let (is_inverted, base_asset, quote_asset) = Self::sort_assets(pair)?;
        let (input_token, output_token) = pair;

        // If the pool exists, update pool base_amount and quote_amount by trade amounts
        Pools::<T, I>::try_mutate(
            &base_asset,
            &quote_asset,
            |pool| -> Result<BalanceOf<T, I>, DispatchError> {
                // 1. If the pool we want to trade does not exist in the current instance, error
                let pool = pool.as_mut().ok_or(Error::<T, I>::PoolDoesNotExist)?;

                // supply_in == pool.base_amount unless inverted
                let (supply_in, supply_out) = if is_inverted {
                    (pool.quote_amount, pool.base_amount)
                } else {
                    (pool.base_amount, pool.quote_amount)
                };

                // amount must incur at least 1 in lp fees
                ensure!(
                    amount_in >= T::LpFee::get().saturating_reciprocal_mul(One::one())
                        && amount_in >= T::ProtocolFee::get().saturating_reciprocal_mul(One::one()),
                    Error::<T, I>::InsufficientAmountIn
                );

                let amount_out =
                    Self::get_amount_out(amount_in, supply_in, supply_out, T::LpFee::get())?;

                // TODO: we should only do this check if we are calculating a minimum amount out
                // 4. If `amount_out` is lower than `min_amount_out`, error
                ensure!(
                    amount_out >= minimum_amount_out && amount_in > Zero::zero(),
                    Error::<T, I>::InsufficientAmountOut
                );

                Self::update(pool, amount_in, amount_out, is_inverted)?;

                Self::transfer_between_user_and_pallet(
                    input_token,
                    output_token,
                    base_asset,
                    quote_asset,
                    who,
                    amount_in,
                    amount_out,
                )?;

                // Return amount out for router pallet
                Ok(amount_out)
            },
        ) // return output of try_mutate as `trade` output
    }
}
