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

use frame_support::{
    log,
    traits::tokens::{
        fungibles::{Inspect, Transfer},
        DepositConsequence, WithdrawConsequence,
    },
};

use crate::{AssetIdOf, BalanceOf, *};

impl<T: Config> Inspect<T::AccountId> for Pallet<T>
where
    BalanceOf<T>: FixedPointOperand + From<u128>,
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
        Self::can_move(asset_id, who).unwrap_or_default()
    }

    /// Returns `true` if the balance of `who` may be increased by `amount`.
    fn can_deposit(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DepositConsequence {
        match (
            Self::ensure_market(asset),
            Self::total_supply(asset).checked_add(&amount),
            // Self::balance(asset, who).is_zero() && amount < Self::minimum_balance(asset),
        ) {
            (Err(_), _) => DepositConsequence::UnknownAsset,
            (_, None) => DepositConsequence::Overflow,
            _ => {
                if Self::balance(asset, who) + amount < Self::minimum_balance(asset) {
                    DepositConsequence::BelowMinimum
                } else {
                    DepositConsequence::Success
                }
            }
        }
    }

    /// Returns `Failed` if the balance of `who` may not be decreased by `amount`, otherwise
    /// the consequence.
    fn can_withdraw(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        match (
            Self::ensure_market(asset),
            Self::balance(asset, who).checked_sub(&amount),
        ) {
            (Err(_), _) => WithdrawConsequence::UnknownAsset,
            (_, None) => WithdrawConsequence::NoFunds,
            (_, Some(rest)) => {
                if rest < Self::minimum_balance(asset) {
                    WithdrawConsequence::ReducedToZero(rest)
                } else {
                    WithdrawConsequence::Success
                }
            }
        }
    }
}

impl<T: Config> Transfer<T::AccountId> for Pallet<T>
where
    BalanceOf<T>: FixedPointOperand + From<u128>,
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
    BalanceOf<T>: FixedPointOperand + From<u128>,
    AssetIdOf<T>: AtLeast32BitUnsigned,
{
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
        let crate::types::Deposits {
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

        if liquidity > collateral_value {
            return Ok(voucher_balance);
        }

        // Formula
        // usable_voucher_amount = liquidity / collateral_factor / price
        // TODO(alannotnerd): Add full test cases.
        let price = Self::get_price(asset_id)?;

        let usable_voucher_amount = liquidity
            .checked_div(&market.collateral_factor.into())
            .and_then(|v| v.checked_div(&price))
            .ok_or(ArithmeticError::Overflow)?
            .into_inner();

        let exchange_rate = Self::exchange_rate(asset_id);
        let amount = Self::calc_collateral_amount(usable_voucher_amount.into(), exchange_rate)?;
        log::trace!(target: "loans::can_move", "{:?}", amount);
        Ok(amount)
    }
}
