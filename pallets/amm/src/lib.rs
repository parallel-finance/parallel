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

mod benchmarking;

pub mod weights;

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
use sp_runtime::{
    traits::{
        AccountIdConversion, CheckedDiv, IntegerSquareRoot, One, StaticLookup, UniqueSaturatedInto,
        Zero,
    },
    ArithmeticError, DispatchError, FixedU128, Perbill, SaturatedConversion,
};
pub use weights::WeightInfo;

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

        /// Currency type for deposit/withdraw assets to/from amm
        /// module
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Weight information for extrinsics in this pallet.
        type AMMWeightInfo: WeightInfo;

        /// A configuration flag to enable or disable the creation of new pools by "normal" users.
        #[pallet::constant]
        type AllowPermissionlessPoolCreation: Get<bool>;

        /// Defines the fees taken out of each trade and sent back to the AMM pool,
        /// typically 0.3%.
        #[pallet::constant]
        type LpFee: Get<Perbill>;

        /// How much the protocol is taking out of each trade.
        #[pallet::constant]
        type ProtocolFee: Get<Perbill>;

        /// Who/where to send the protocol fees
        #[pallet::constant]
        type ProtocolFeeReceiver: Get<Self::AccountId>;
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Pool does not exist
        PoolDoesNotExist,
        /// More liquidity than user's liquidity
        MoreLiquidity,
        /// Not a ideal price ratio
        NotAIdealPriceRatio,
        /// Pool creation has been disabled
        PoolCreationDisabled,
        /// Pool does not exist
        PoolAlreadyExists,
        /// Amount out is too small
        InsufficientAmountOut,
        /// Amount in is too small
        InsufficientAmountIn,
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
        Trade(T::AccountId, AssetIdOf<T, I>, AssetIdOf<T, I>, Rate),
    }

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<T::BlockNumber> for Pallet<T, I> {}

    #[pallet::pallet]
    pub struct Pallet<T, I = ()>(_);

    #[derive(
        Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo,
    )]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct PoolLiquidityAmount<CurrencyId, Balance> {
        pub base_amount: Balance,
        pub quote_amount: Balance,
        pub pool_assets: CurrencyId,
    }

    /// The exchange rate from the underlying to the internal collateral
    #[pallet::storage]
    pub type ExchangeRate<T, I = ()> = StorageValue<_, Rate, ValueQuery>;

    /// Accounts that deposits and withdraw assets in one or more pools
    #[pallet::storage]
    #[pallet::getter(fn liquidity_providers)]
    pub type LiquidityProviders<T: Config<I>, I: 'static = ()> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>,
            NMapKey<Blake2_128Concat, AssetIdOf<T, I>>,
            NMapKey<Blake2_128Concat, AssetIdOf<T, I>>,
        ),
        PoolLiquidityAmount<AssetIdOf<T, I>, BalanceOf<T, I>>,
        OptionQuery,
        GetDefault,
    >;

    /// A bag of liquidity composed by two different assets
    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
        _,
        Twox64Concat,
        AssetIdOf<T, I>,
        Twox64Concat,
        AssetIdOf<T, I>,
        PoolLiquidityAmount<AssetIdOf<T, I>, BalanceOf<T, I>>,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Allow users to add liquidity to a given pool
        ///
        /// - `pool`: Currency pool, in which liquidity will be added
        /// - `liquidity_amounts`: Liquidity amounts to be added in pool
        /// - `minimum_amounts`: specifying its "worst case" ratio when pool already exists
        #[pallet::weight(
		T::AMMWeightInfo::add_liquidity_non_existing_pool() // Adds liquidity in already existing account.
		.max(T::AMMWeightInfo::add_liquidity_existing_pool()) // Adds liquidity in new account
		)]
        #[transactional]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            pool: (AssetIdOf<T, I>, AssetIdOf<T, I>),
            liquidity_amounts: (BalanceOf<T, I>, BalanceOf<T, I>),
            minimum_amounts: (BalanceOf<T, I>, BalanceOf<T, I>),
            asset_id: AssetIdOf<T, I>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let (is_inverted, base_asset, quote_asset) = Self::get_upper_currency(pool.0, pool.1);

            let (base_amount, quote_amount) = if is_inverted {
                (liquidity_amounts.1, liquidity_amounts.0)
            } else {
                (liquidity_amounts.0, liquidity_amounts.1)
            };

            Pools::<T, I>::try_mutate(
                base_asset,
                quote_asset,
                |pool_liquidity_amount| -> DispatchResultWithPostInfo {
                    if let Some(liquidity_amount) = pool_liquidity_amount {
                        let optimal_quote_amount = Self::quote(
                            base_amount,
                            liquidity_amount.base_amount,
                            liquidity_amount.quote_amount,
                        );

                        let (ideal_base_amount, ideal_quote_amount): (
                            BalanceOf<T, I>,
                            BalanceOf<T, I>,
                        ) = if optimal_quote_amount <= quote_amount {
                            (base_amount, optimal_quote_amount)
                        } else {
                            let optimal_base_amount = Self::quote(
                                quote_amount,
                                liquidity_amount.quote_amount,
                                liquidity_amount.base_amount,
                            );
                            (optimal_base_amount, quote_amount)
                        };

                        let (minimum_base_amount, minimum_quote_amount) = if is_inverted {
                            (minimum_amounts.1, minimum_amounts.0)
                        } else {
                            (minimum_amounts.0, minimum_amounts.1)
                        };

                        ensure!(
                            ideal_base_amount >= minimum_base_amount
                                && ideal_quote_amount >= minimum_quote_amount
                                && ideal_base_amount <= base_amount
                                && ideal_quote_amount <= quote_amount,
                            Error::<T, I>::NotAIdealPriceRatio
                        );

                        let (base_amount, quote_amount) = (ideal_base_amount, ideal_quote_amount);
                        let total_ownership = T::Assets::total_issuance(asset_id);
                        let ownership = sp_std::cmp::min(
                            (base_amount.saturating_mul(total_ownership))
                                .checked_div(liquidity_amount.base_amount)
                                .ok_or(ArithmeticError::Overflow)?,
                            (quote_amount.saturating_mul(total_ownership))
                                .checked_div(liquidity_amount.quote_amount)
                                .ok_or(ArithmeticError::Overflow)?,
                        );

                        liquidity_amount.base_amount = liquidity_amount
                            .base_amount
                            .checked_add(base_amount)
                            .ok_or(ArithmeticError::Overflow)?;
                        liquidity_amount.quote_amount = liquidity_amount
                            .quote_amount
                            .checked_add(quote_amount)
                            .ok_or(ArithmeticError::Overflow)?;

                        *pool_liquidity_amount = Some(*liquidity_amount);

                        LiquidityProviders::<T, I>::try_mutate(
                            (&who, &base_asset, &quote_asset),
                            |pool_liquidity_amount| -> DispatchResult {
                                if let Some(liquidity_amount) = pool_liquidity_amount {
                                    liquidity_amount.base_amount = liquidity_amount
                                        .base_amount
                                        .checked_add(base_amount)
                                        .ok_or(ArithmeticError::Overflow)?;
                                    liquidity_amount.quote_amount = liquidity_amount
                                        .quote_amount
                                        .checked_add(quote_amount)
                                        .ok_or(ArithmeticError::Overflow)?;
                                    *pool_liquidity_amount = Some(*liquidity_amount);
                                }
                                Ok(())
                            },
                        )?;

                        Self::mint_transfer_liquidity(
                            who,
                            ownership,
                            asset_id,
                            base_asset,
                            quote_asset,
                            base_amount,
                            quote_amount,
                        )?;
                        Ok(Some(T::AMMWeightInfo::add_liquidity_non_existing_pool()).into())
                    } else {
                        Self::add_new_liquidity(
                            asset_id,
                            who,
                            base_asset,
                            quote_asset,
                            base_amount,
                            quote_amount,
                            asset_id,
                            pool_liquidity_amount,
                        )?;

                        Ok(Some(T::AMMWeightInfo::add_liquidity_existing_pool()).into())
                    }
                },
            )
        }

        /// Allow users to remove liquidity from a given pool
        ///
        /// - `pool`: Currency pool, in which liquidity will be removed
        /// - `ownership_to_remove`: Ownership to be removed from user's ownership
        #[pallet::weight(T::AMMWeightInfo::remove_liquidity())]
        #[transactional]
        pub fn remove_liquidity(
            origin: OriginFor<T>,
            pool: (AssetIdOf<T, I>, AssetIdOf<T, I>),
            ownership_to_remove: BalanceOf<T, I>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let (_, base_asset, quote_asset) = Self::get_upper_currency(pool.0, pool.1);

            Pools::<T, I>::try_mutate(
                base_asset,
                quote_asset,
                |pool_liquidity_amount| -> DispatchResult {
                    let mut liquidity_amount = pool_liquidity_amount
                        .take()
                        .ok_or(Error::<T, I>::PoolDoesNotExist)?;
                    let total_ownership = T::Assets::total_issuance(liquidity_amount.pool_assets);
                    ensure!(
                        total_ownership >= ownership_to_remove,
                        Error::<T, I>::MoreLiquidity
                    );

                    let base_amount = (ownership_to_remove
                        .saturating_mul(liquidity_amount.base_amount))
                    .checked_div(total_ownership)
                    .ok_or(ArithmeticError::Underflow)?;

                    let quote_amount = (ownership_to_remove
                        .saturating_mul(liquidity_amount.quote_amount))
                    .checked_div(total_ownership)
                    .ok_or(ArithmeticError::Underflow)?;

                    liquidity_amount.base_amount = liquidity_amount
                        .base_amount
                        .checked_sub(base_amount)
                        .ok_or(ArithmeticError::Underflow)?;
                    liquidity_amount.quote_amount = liquidity_amount
                        .quote_amount
                        .checked_sub(quote_amount)
                        .ok_or(ArithmeticError::Underflow)?;

                    LiquidityProviders::<T, I>::try_mutate(
                        (&who, &base_asset, &quote_asset),
                        |pool_liquidity_amount| -> DispatchResult {
                            if let Some(liquidity_amount) = pool_liquidity_amount {
                                liquidity_amount.base_amount = liquidity_amount
                                    .base_amount
                                    .checked_sub(base_amount)
                                    .ok_or(ArithmeticError::Underflow)?;
                                liquidity_amount.quote_amount = liquidity_amount
                                    .quote_amount
                                    .checked_sub(quote_amount)
                                    .ok_or(ArithmeticError::Underflow)?;
                            }
                            Ok(())
                        },
                    )?;
                    T::Assets::burn_from(liquidity_amount.pool_assets, &who, ownership_to_remove)?;
                    T::Assets::transfer(base_asset, &Self::account_id(), &who, base_amount, false)?;
                    T::Assets::transfer(
                        quote_asset,
                        &Self::account_id(),
                        &who,
                        quote_amount,
                        false,
                    )?;

                    Self::deposit_event(Event::<T, I>::LiquidityRemoved(
                        who,
                        base_asset,
                        quote_asset,
                    ));

                    Ok(())
                },
            )
        }

        /// "force" the creation of a new pool by root
        ///
        /// - `pool`: Currency pool, in which liquidity will be added
        /// - `liquidity_amounts`: Liquidity amounts to be added in pool
        /// - `lptoken_receiver`: Allocate any liquidity tokens to lptoken_receiver
        #[pallet::weight(T::AMMWeightInfo::force_create_pool())]
        #[transactional]
        pub fn force_create_pool(
            origin: OriginFor<T>,
            pool: (AssetIdOf<T, I>, AssetIdOf<T, I>),
            liquidity_amounts: (BalanceOf<T, I>, BalanceOf<T, I>),
            lptoken_receiver: T::AccountId,
            asset_id: AssetIdOf<T, I>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let (is_inverted, base_asset, quote_asset) = Self::get_upper_currency(pool.0, pool.1);
            ensure!(
                !Pools::<T, I>::contains_key(&base_asset, &quote_asset),
                Error::<T, I>::PoolAlreadyExists
            );

            let (base_amount, quote_amount) = if is_inverted {
                (liquidity_amounts.1, liquidity_amounts.0)
            } else {
                (liquidity_amounts.0, liquidity_amounts.1)
            };

            let ownership = base_amount.saturating_mul(quote_amount).integer_sqrt();
            let amm_pool = PoolLiquidityAmount {
                base_amount,
                quote_amount,
                pool_assets: asset_id,
            };
            Pools::<T, I>::insert(&base_asset, &quote_asset, amm_pool);
            Self::insert_into_liquidity_providers(
                &lptoken_receiver,
                asset_id,
                &base_asset,
                &quote_asset,
                amm_pool,
            )?;

            Self::mint_transfer_liquidity(
                lptoken_receiver.clone(),
                ownership,
                asset_id,
                base_asset,
                quote_asset,
                base_amount,
                quote_amount,
            )?;
            Ok(().into())
        }
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    pub fn get_upper_currency(
        curr_a: AssetIdOf<T, I>,
        curr_b: AssetIdOf<T, I>,
    ) -> (bool, AssetIdOf<T, I>, AssetIdOf<T, I>) {
        if curr_a > curr_b {
            (false, curr_a, curr_b)
        } else {
            (true, curr_b, curr_a)
        }
    }

    pub fn quote(
        amount: BalanceOf<T, I>,
        base_pool: BalanceOf<T, I>,
        quote_pool: BalanceOf<T, I>,
    ) -> BalanceOf<T, I> {
        (amount.saturating_mul(quote_pool))
            .checked_div(base_pool)
            .expect("cannot overflow with positive divisor; qed")
    }

    fn mint_transfer_liquidity(
        who: T::AccountId,
        ownership: BalanceOf<T, I>,
        currency_asset: AssetIdOf<T, I>,
        base_asset: AssetIdOf<T, I>,
        quote_asset: AssetIdOf<T, I>,
        base_amount: BalanceOf<T, I>,
        quote_amount: BalanceOf<T, I>,
    ) -> DispatchResult {
        T::Assets::mint_into(currency_asset, &who, ownership)?;
        T::Assets::transfer(base_asset, &who, &Self::account_id(), base_amount, true)?;
        T::Assets::transfer(quote_asset, &who, &Self::account_id(), quote_amount, true)?;

        Self::deposit_event(Event::<T, I>::LiquidityAdded(who, base_asset, quote_asset));

        Ok(())
    }

    fn insert_into_liquidity_providers(
        lptoken_receiver: &T::AccountId,
        asset_id: AssetIdOf<T, I>,
        base_asset: &AssetIdOf<T, I>,
        quote_asset: &AssetIdOf<T, I>,
        amm_pool: PoolLiquidityAmount<AssetIdOf<T, I>, BalanceOf<T, I>>,
    ) -> DispatchResult {
        LiquidityProviders::<T, I>::insert(
            (&lptoken_receiver, &base_asset, &quote_asset),
            amm_pool,
        );

        pallet_assets::Pallet::<T>::force_create(
            RawOrigin::Root.into(),
            asset_id.unique_saturated_into(),
            T::Lookup::unlookup(Self::account_id()),
            true,
            One::one(),
        )?;

        Ok(())
    }

    fn add_new_liquidity(
        asset_id: AssetIdOf<T, I>,
        who: T::AccountId,
        base_asset: AssetIdOf<T, I>,
        quote_asset: AssetIdOf<T, I>,
        base_amount: BalanceOf<T, I>,
        quote_amount: BalanceOf<T, I>,
        currency_asset: AssetIdOf<T, I>,
        pool_liquidity_amount: &mut Option<PoolLiquidityAmount<AssetIdOf<T, I>, BalanceOf<T, I>>>,
    ) -> DispatchResult {
        ensure!(
            T::AllowPermissionlessPoolCreation::get(),
            Error::<T, I>::PoolCreationDisabled
        );

        let ownership = base_amount.saturating_mul(quote_amount).integer_sqrt();
        let amm_pool = PoolLiquidityAmount {
            base_amount,
            quote_amount,
            pool_assets: currency_asset,
        };

        *pool_liquidity_amount = Some(amm_pool);
        Self::insert_into_liquidity_providers(&who, asset_id, &base_asset, &quote_asset, amm_pool)?;
        Self::mint_transfer_liquidity(
            who,
            ownership,
            currency_asset,
            base_asset,
            quote_asset,
            base_amount,
            quote_amount,
        )?;

        Ok(())
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
        let (input_token, output_token) = pair;

        // Sort pair to interact with the correct pool.
        let (is_inverted, base_asset, quote_asset) =
            Self::get_upper_currency(input_token, output_token);

        // If the pool exists, update pool base_amount and quote_amount by trade amounts
        Pools::<T, I>::try_mutate(
            &base_asset,
            &quote_asset,
            |pool_liquidity_amount| -> Result<BalanceOf<T, I>, DispatchError> {
                // 1. If the pool we want to trade does not exist in the current instance, error
                let mut liquidity_amount = pool_liquidity_amount
                    .take()
                    .ok_or(Error::<T, I>::PoolDoesNotExist)?;

                // supply_in == liquidity_amount.base_amount unless inverted
                let (supply_in, supply_out) = if is_inverted {
                    (liquidity_amount.quote_amount, liquidity_amount.base_amount)
                } else {
                    (liquidity_amount.base_amount, liquidity_amount.quote_amount)
                };

                // amount must incur at least 1 in lp fees
                ensure!(
                    amount_in >= T::LpFee::get().saturating_reciprocal_mul(One::one())
                        && amount_in >= T::ProtocolFee::get().saturating_reciprocal_mul(One::one()),
                    Error::<T, I>::InsufficientAmountIn
                );

                // 2. Compute all fees to be taken out, see @Fees
                // we round down for trader convenience
                let lp_fees = T::LpFee::get().mul_floor(amount_in);
                let protocol_fees = T::ProtocolFee::get().mul_floor(amount_in);

                // subtract protocol fees from amount_in
                let amount_without_protocol_fees = amount_in
                    .checked_sub(protocol_fees)
                    .ok_or(ArithmeticError::Underflow)?;

                // subtract lp fees from amount_in minus protocol fees
                let amount_in_after_all_fees = amount_without_protocol_fees
                    .checked_sub(lp_fees)
                    .ok_or(ArithmeticError::Underflow)?;

                // 3. Given the input amount amount_in left after fees, compute amount_out
                // let amount_out = amount_in * supply_out / (supply_in + amount_in)
                let amount_out = amount_in_after_all_fees
                    .saturating_mul(supply_out)
                    .checked_div(
                        supply_in
                            .checked_add(amount_in_after_all_fees)
                            .ok_or(ArithmeticError::Overflow)?,
                    )
                    .ok_or(ArithmeticError::Underflow)?;

                // 4. If `amount_out` is lower than `min_amount_out`, error
                ensure!(
                    amount_out >= minimum_amount_out && amount_in > Zero::zero(),
                    Error::<T, I>::InsufficientAmountOut
                );

                // 5. Update the `Pools` storage to track the `base_amount` and `quote_amount`
                // variables (increase and decrease by `amount_in` and `amount_out`)
                // increase liquidity_amount.base_amount by amount_in, unless inverted
                if is_inverted {
                    liquidity_amount.quote_amount = liquidity_amount
                        .quote_amount
                        .checked_add(amount_without_protocol_fees)
                        .ok_or(ArithmeticError::Overflow)?;

                    liquidity_amount.base_amount = liquidity_amount
                        .base_amount
                        .checked_sub(amount_out)
                        .ok_or(ArithmeticError::Underflow)?;
                } else {
                    liquidity_amount.base_amount = liquidity_amount
                        .base_amount
                        .checked_add(amount_without_protocol_fees)
                        .ok_or(ArithmeticError::Overflow)?;

                    liquidity_amount.quote_amount = liquidity_amount
                        .quote_amount
                        .checked_sub(amount_out)
                        .ok_or(ArithmeticError::Underflow)?;
                }
                *pool_liquidity_amount = Some(liquidity_amount);

                // 6. Wire amount_in of the input token (identified by pair.0) from who to PalletId
                T::Assets::transfer(
                    input_token,
                    who,
                    &Self::account_id(),
                    amount_without_protocol_fees,
                    true,
                )?;

                // 7. Wire amount_out of the output token (identified by pair.1) to who from PalletId
                T::Assets::transfer(output_token, &Self::account_id(), who, amount_out, true)?;

                // 8. Wire protocol fees as needed (input token)
                T::Assets::transfer(
                    input_token,
                    who,
                    &T::ProtocolFeeReceiver::get(),
                    protocol_fees,
                    true,
                )?;

                // Emit event of trade with rate calculated
                Self::deposit_event(Event::<T, I>::Trade(
                    who.clone(),
                    base_asset,
                    quote_asset,
                    FixedU128::from_inner(amount_out.saturated_into())
                        .checked_div(&FixedU128::from_inner(amount_in.saturated_into()))
                        .ok_or(ArithmeticError::Underflow)?,
                ));

                // Return amount out for router pallet
                Ok(amount_out)
            },
        ) // return output of try_mutate as `trade` output
    }
}
