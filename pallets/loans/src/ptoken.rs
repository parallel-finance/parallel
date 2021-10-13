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

use crate::*;

use frame_support::traits::tokens::{
    fungibles::{Inspect as Inspects, Transfer as Transfers},
    DepositConsequence, WithdrawConsequence,
};

type AssetIdOf<T> =
    <<T as Config>::Assets as Inspects<<T as frame_system::Config>::AccountId>>::AssetId;
type BalanceOf<T> =
    <<T as Config>::Assets as Inspects<<T as frame_system::Config>::AccountId>>::Balance;

// use sp_runtime::traits::{CheckedAdd, CheckedDiv, CheckedSub, Saturating};

impl<T: Config> Inspects<T::AccountId> for Pallet<T> {
    type AssetId = AssetIdOf<T>;
    type Balance = BalanceOf<T>;

    fn total_issuance(_asset: Self::AssetId) -> Self::Balance {
        todo!()
    }

    fn minimum_balance(_asset: Self::AssetId) -> Self::Balance {
        todo!()
    }

    fn balance(_asset: Self::AssetId, _who: &T::AccountId) -> Self::Balance {
        todo!()
    }

    fn reducible_balance(
        _asset: Self::AssetId,
        _who: &T::AccountId,
        _keep_alive: bool,
    ) -> Self::Balance {
        todo!()
    }

    fn can_deposit(
        _asset: Self::AssetId,
        _who: &T::AccountId,
        _amount: Self::Balance,
    ) -> DepositConsequence {
        todo!()
    }

    fn can_withdraw(
        _asset: Self::AssetId,
        _who: &T::AccountId,
        _amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        todo!()
    }
}

impl<T: Config> Transfers<T::AccountId> for Pallet<T>
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
        Self::transfer_ptokens_allowed(asset, source, amount)?;

        AccountDeposits::<T>::try_mutate_exists(asset, source, |deposits| -> DispatchResult {
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

        AccountDeposits::<T>::try_mutate(asset, &dest, |deposits| -> DispatchResult {
            deposits.voucher_balance = deposits
                .voucher_balance
                .checked_add(&amount)
                .ok_or(ArithmeticError::Overflow)?;
            Ok(())
        })?;

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
        asset: AssetIdOf<T>,
        source: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        let deposit = Self::account_deposits(asset, &source);
        if amount > deposit.voucher_balance {
            return Err(Error::<T>::InsufficientCollateral.into());
        }

        if !deposit.is_collateral {
            return Ok(());
        }

        let (liquidity, _) = Self::get_account_liquidity(&source)?;
        if FixedU128::from_inner(amount.saturated_into()) > liquidity {
            return Err(Error::<T>::InsufficientLiquidity.into());
        }

        Ok(())
    }

    pub(crate) fn transfer_ptokens_internal(
        source: &T::AccountId,
        dest: &T::AccountId,
        asset_id: AssetIdOf<T>,
        amount: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        Self::transfer_ptokens_allowed(asset_id, source, amount)?;

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
}
