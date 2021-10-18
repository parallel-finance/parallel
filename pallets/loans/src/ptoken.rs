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

#![cfg_attr(not(feature = "std"), no_std)]

use crate::{AssetIdOf, BalanceOf, *};
use frame_support::traits::tokens::{
    fungibles::{Inspect, Transfer},
    DepositConsequence, WithdrawConsequence,
};

impl<T: Config> Inspect<T::AccountId> for Pallet<T>
where
    BalanceOf<T>: FixedPointOperand,
    AssetIdOf<T>: AtLeast32BitUnsigned,
{
    type AssetId = AssetIdOf<T>;
    type Balance = BalanceOf<T>;

    /// The total amount of issuance in the system.
    fn total_issuance(asset: Self::AssetId) -> Self::Balance {
        Self::total_supply(asset)
    }

    /// The minimum balance any single account may have.
    fn minimum_balance(_asset: Self::AssetId) -> Self::Balance {
        Zero::zero()
    }

    /// Get the balance of `who`.
    fn balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        Self::account_deposits(asset, who).voucher_balance
    }

    /// Get the maximum amount that `who` can withdraw/transfer successfully.
    fn reducible_balance(
        asset_id: Self::AssetId,
        who: &T::AccountId,
        _keep_alive: bool,
    ) -> Self::Balance {
        if let Ok(balance) = Self::can_move(asset_id, who) {
            balance
        } else {
            Zero::zero()
        }
    }

    /// Returns `true` if the balance of `who` may be increased by `amount`.
    fn can_deposit(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DepositConsequence {
        Self::can_increase(asset, who, amount)
    }

    /// Returns `Failed` if the balance of `who` may not be decreased by `amount`, otherwise
    /// the consequence.
    fn can_withdraw(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        Self::can_decrease(asset, who, amount, true)
    }
}

impl<T: Config> Transfer<T::AccountId> for Pallet<T>
where
    BalanceOf<T>: FixedPointOperand,
    AssetIdOf<T>: AtLeast32BitUnsigned,
{
    fn transfer(
        asset: Self::AssetId,
        source: &T::AccountId,
        dest: &T::AccountId,
        amount: Self::Balance,
        _keep_alive: bool,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let ptoken_id = Self::ptoken_id(asset)?;

        Self::transfer_ptokens_allowed(ptoken_id, source, amount)?;

        Self::transfer_ptokens_internal(ptoken_id, source, dest, amount)?;

        Ok(amount)
    }
}

impl<T: Config> Pallet<T>
where
    BalanceOf<T>: FixedPointOperand,
    AssetIdOf<T>: AtLeast32BitUnsigned,
{
    /// Checks if the source should be allowed to transfer ptokens in given conditions
    pub(crate) fn transfer_ptokens_allowed(
        ptoken_id: AssetIdOf<T>,
        source: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        let asset_id = Self::underlying_id(ptoken_id)?;

        let deposit = Self::account_deposits(asset_id, &source);
        if amount > deposit.voucher_balance {
            return Err(Error::<T>::InsufficientCollateral.into());
        }

        if !deposit.is_collateral {
            return Ok(());
        }

        // Formula: effect_value = ptokens_amount * exchange rate * price
        let effects_value = Self::get_price(asset_id)?
            .checked_mul(&FixedU128::from_inner(
                Self::market(asset_id)?.collateral_factor.mul_ceil(
                    Self::calc_underlying_amount(amount, Self::exchange_rate(asset_id))?
                        .saturated_into(),
                ),
            ))
            .ok_or(ArithmeticError::Overflow)?;

        let (liquidity, _) = Self::get_account_liquidity(source)?;
        if liquidity < effects_value {
            return Err(Error::<T>::InsufficientLiquidity.into());
        }

        Ok(())
    }

    pub(crate) fn transfer_ptokens_internal(
        ptoken_id: AssetIdOf<T>,
        source: &T::AccountId,
        dest: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        let asset_id = Self::underlying_id(ptoken_id)?;
        AccountDeposits::<T>::try_mutate_exists(asset_id, source, |deposits| -> DispatchResult {
            let mut d = deposits.unwrap_or_default();
            d.voucher_balance = d
                .voucher_balance
                .checked_sub(&amount)
                .ok_or(ArithmeticError::Underflow)?;
            if d.voucher_balance.is_zero() {
                // remove deposits storage if zero balance
                *deposits = None;
            } else {
                *deposits = Some(d);
            }
            Ok(())
        })?;

        AccountDeposits::<T>::try_mutate(asset_id, &dest, |deposits| -> DispatchResult {
            deposits.voucher_balance = deposits
                .voucher_balance
                .checked_add(&amount)
                .ok_or(ArithmeticError::Overflow)?;
            Ok(())
        })?;

        Ok(())
    }

    pub(super) fn can_move(
        asset_id: AssetIdOf<T>,
        who: &T::AccountId,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let deposit = Self::account_deposits(asset_id, &who);

        if !deposit.is_collateral {
            return Ok(deposit.voucher_balance);
        }

        let exchange_rate = Self::exchange_rate(asset_id);
        let collateral_factor = Self::market(asset_id)?.collateral_factor;
        let price = Self::get_price(asset_id)?;

        // Formula
        // effect_value = ptokens_amount * collateral_factor * exchange rate * price
        let effects_value = price
            .checked_mul(&FixedU128::from_inner(
                collateral_factor.mul_ceil(
                    Self::calc_underlying_amount(deposit.voucher_balance, exchange_rate)?
                        .saturated_into(),
                ),
            ))
            .ok_or(ArithmeticError::Overflow)?;

        let (liquidity, _) = Self::get_account_liquidity(who)?;
        // log::debug!("liquidity: {:?}, effects_value: {:?}", liquidity, effects_value);
        let ptokens = if liquidity < effects_value {
            // Formula
            // ptokens_amount = liquidity / collateral_factor / exchange_rate / price
            // Balance        = Balance   / Permill           / Rate(FixedU128) / FixedU128

            // TODO: Complete the above formula calculation
            let ptokens_amount = Self::calc_collateral_amount(
                liquidity.into_inner().saturated_into(),
                exchange_rate,
            )?;
            // .checked_div(collateral_factor)
            // .ok_or(ArithmeticError::Overflow)?
            // .checked_div(Self::get_price(asset_id))
            // .ok_or(ArithmeticError::Underflow)?;

            Ok(ptokens_amount)
        } else {
            // There is enough liquidity to transfer the entire ptoken amount of deposit
            Ok(deposit.voucher_balance)
        };

        ptokens
    }

    pub(super) fn can_increase(
        asset: AssetIdOf<T>,
        who: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DepositConsequence {
        match Self::ensure_market(asset) {
            Ok(_) => (),
            Err(_) => return DepositConsequence::UnknownAsset,
        }

        if Self::total_supply(asset).checked_add(&amount).is_none() {
            return DepositConsequence::Overflow;
        }

        if Self::balance(asset, who).checked_add(&amount).is_none() {
            return DepositConsequence::Overflow;
        }

        if Self::balance(asset, who).is_zero() && amount < Self::minimum_balance(asset) {
            return DepositConsequence::BelowMinimum;
        }

        DepositConsequence::Success
    }

    /// Return the consequence of a withdraw.
    pub(super) fn can_decrease(
        asset: AssetIdOf<T>,
        who: &T::AccountId,
        amount: BalanceOf<T>,
        _keep_alive: bool,
    ) -> WithdrawConsequence<BalanceOf<T>> {
        match Self::ensure_market(asset) {
            Ok(_) => (),
            Err(_) => return WithdrawConsequence::UnknownAsset,
        }

        if Self::total_supply(asset).checked_sub(&amount).is_none() {
            return WithdrawConsequence::Underflow;
        }
        if let Some(rest) = Self::balance(asset, who).checked_sub(&amount) {
            if rest < Self::minimum_balance(asset) {
                WithdrawConsequence::ReducedToZero(rest)
            } else {
                WithdrawConsequence::Success
            }
        } else {
            WithdrawConsequence::NoFunds
        }
    }
}
