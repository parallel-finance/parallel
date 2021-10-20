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

use crate::{types::Deposits, AssetIdOf, BalanceOf, *};
use frame_support::traits::tokens::{
    fungibles::{Inspect, Transfer},
    DepositConsequence, WithdrawConsequence,
};

impl<T: Config> Inspect<T::AccountId> for Pallet<T>
where
    BalanceOf<T>: From<u128>,
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

    /// Get the ptoken balance of `who`.
    fn balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        Self::account_deposits(asset, who).voucher_balance
    }

    /// Get the maximum amount that `who` can withdraw/transfer successfully.
    /// For ptoken, We don't care if keep_alive is enabled
    fn reducible_balance(
        asset_id: Self::AssetId,
        who: &T::AccountId,
        _keep_alive: bool,
    ) -> Self::Balance {
        Self::reducible_ptoken(asset_id, who).unwrap_or_default()
    }

    /// Returns `true` if the balance of `who` may be increased by `amount`.
    fn can_deposit(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DepositConsequence {
        if Self::ensure_market(asset).is_err() {
            return DepositConsequence::UnknownAsset;
        }

        if Self::total_supply(asset).checked_add(amount).is_none() {
            return DepositConsequence::Overflow;
        }

        if Self::balance(asset, who) + amount < Self::minimum_balance(asset) {
            return DepositConsequence::BelowMinimum;
        }

        DepositConsequence::Success
    }

    /// Returns `Failed` if the balance of `who` may not be decreased by `amount`, otherwise
    /// the consequence.
    fn can_withdraw(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        if Self::ensure_market(asset).is_err() {
            return WithdrawConsequence::UnknownAsset;
        }

        let sub_result = Self::balance(asset, who).checked_sub(amount);
        if sub_result.is_none() {
            return WithdrawConsequence::NoFunds;
        }

        let rest = sub_result.expect("Cannot be none; qed");
        if rest < Self::minimum_balance(asset) {
            return WithdrawConsequence::ReducedToZero(rest);
        }

        WithdrawConsequence::Success
    }
}

impl<T: Config> Transfer<T::AccountId> for Pallet<T>
where
    BalanceOf<T>: From<u128>,
{
    /// Returns `Err` if the reducible ptoken of `who` is insufficient
    ///
    /// For ptoken, We don't care if keep_alive is enabled
    fn transfer(
        asset: Self::AssetId,
        source: &T::AccountId,
        dest: &T::AccountId,
        amount: Self::Balance,
        _keep_alive: bool,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let ptoken_id = Self::ptoken_id(asset)?;
        ensure!(
            amount <= Self::reducible_balance(asset, source, false),
            Error::<T>::InsufficientCollateral
        );

        Self::transfer_ptokens_internal(ptoken_id, source, dest, amount)?;
        Ok(amount)
    }
}

impl<T: Config> Pallet<T>
where
    BalanceOf<T>: From<u128>,
{
    fn transfer_ptokens_internal(
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
                .checked_sub(amount)
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
                .checked_add(amount)
                .ok_or(ArithmeticError::Overflow)?;
            Ok(())
        })?;

        Ok(())
    }

    fn reducible_ptoken(
        asset_id: AssetIdOf<T>,
        who: &T::AccountId,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let Deposits {
            is_collateral,
            voucher_balance,
        } = Self::account_deposits(asset_id, &who);

        if !is_collateral {
            return Ok(voucher_balance);
        }

        let market = Self::ensure_market(asset_id)?;
        let collateral_value = Self::collateral_asset_value(who, asset_id, &market)?;

        // liquidity of all assets
        let (liquidity, _) = Self::get_account_liquidity(who)?;

        if liquidity >= collateral_value {
            return Ok(voucher_balance);
        }

        // Formula
        // reducible_underlying_amount = liquidity / collateral_factor / price
        let price = Self::get_price(asset_id)?;

        let reducible_supply_balance = liquidity
            .checked_div(&market.collateral_factor.into())
            .ok_or(ArithmeticError::Overflow)?;

        let reducible_underlying_amount = reducible_supply_balance
            .checked_div(&price)
            .ok_or(ArithmeticError::Underflow)?
            .into_inner();

        let exchange_rate = Self::exchange_rate(asset_id);
        let amount = Self::calc_collateral_amount(reducible_underlying_amount, exchange_rate)?;
        Ok(amount)
    }
}
