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
use frame_support::{
    traits::tokens::{
        fungibles::{Inspect, Transfer},
        DepositConsequence, WithdrawConsequence,
    },
};

impl<T: Config> Inspect<T::AccountId> for Pallet<T>
where
    BalanceOf<T>: FixedPointOperand,
    AssetIdOf<T>: AtLeast32BitUnsigned,
{
    type AssetId = AssetIdOf<T>;
    type Balance = BalanceOf<T>;

    fn total_issuance(asset: Self::AssetId) -> Self::Balance {
        Self::total_supply(asset)
    }

    fn minimum_balance(_asset: Self::AssetId) -> Self::Balance {
        0u64.saturated_into()
    }

    fn balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        Self::account_deposits(asset, who).voucher_balance
    }

    fn reducible_balance(
        asset: Self::AssetId,
        who: &T::AccountId,
        _keep_alive: bool,
    ) -> Self::Balance {
        let deposit = Self::account_deposits(asset, &who);

        if !deposit.is_collateral {
            return deposit.voucher_balance;
        }

        let liquidity = match Self::get_account_liquidity(&who) {
            Ok((liquidity, _)) => liquidity,
            Err(e) => {
                log::error!("Inspect account liquidity meet error: {:?}", e);
                FixedU128::from_inner(0)
            }
        };
        
        if liquidity.into_inner() > deposit.voucher_balance.saturated_into() {
            deposit.voucher_balance
        } else {
            liquidity.into_inner().saturated_into()
        }
    }

    fn can_deposit(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DepositConsequence {
        Self::can_increase(asset, who, amount)
    }

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
        Self::transfer_ptokens_allowed(asset, source, amount)?;

        Self::transfer_ptokens_internal(asset, source, dest, amount)?;

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

        // Formula: effect_value = ptokens_amount * exchange rate * price
        let effects_value = Self::get_price(asset)?
            .checked_mul(&FixedU128::from_inner(
                Self::market(asset)?.collateral_factor.mul_ceil(
                    Self::calc_underlying_amount(amount, Self::exchange_rate(asset))?
                        .saturated_into(),
                ),
            ))
            .ok_or(ArithmeticError::Overflow)?;

        let (liquidity, _) = Self::get_account_liquidity(source)?;
        if effects_value > liquidity {
            return Err(Error::<T>::InsufficientLiquidity.into());
        }

        Ok(())
    }
    pub(crate) fn transfer_ptokens_internal(
        asset: AssetIdOf<T>,
        source: &T::AccountId,
        dest: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
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

        Ok(())
    }
    
    pub(super) fn can_increase(
		asset: AssetIdOf<T>,
		who: &T::AccountId,
		amount: BalanceOf<T>,
	) -> DepositConsequence {
        match Self::ensure_currency(asset) {
            Ok(_) => (),
            Err(_) => return DepositConsequence::UnknownAsset,
        }
		
		if Self::total_supply(asset).checked_add(&amount).is_none() {
			return DepositConsequence::Overflow
		}
		
		if Self::balance(asset, who).checked_add(&amount).is_none() {
			return DepositConsequence::Overflow
		}
		if Self::balance(asset, who).is_zero() {
			if amount < Self::minimum_balance(asset) {
				return DepositConsequence::BelowMinimum
			}
		}
        T::Assets::can_deposit(asset, who, amount)
	}
	
	/// Return the consequence of a withdraw.
	pub(super) fn can_decrease(
		asset: AssetIdOf<T>,
		who: &T::AccountId,
		amount: BalanceOf<T>,
		keep_alive: bool,
	) -> WithdrawConsequence<BalanceOf<T>> {
        match Self::ensure_currency(asset) {
            Ok(_) => (),
            Err(_) => return WithdrawConsequence::UnknownAsset,
        }
		
		if Self::total_supply(asset).checked_sub(&amount).is_none() {
			return WithdrawConsequence::Underflow
		}
		if let Some(rest) = Self::balance(asset, who).checked_sub(&amount) {
			let is_provider = false;
			let is_required = is_provider && !frame_system::Pallet::<T>::can_dec_provider(who);
			let must_keep_alive = keep_alive || is_required;

			if rest < Self::minimum_balance(asset) {
				if must_keep_alive {
					WithdrawConsequence::WouldDie
				} else {
					WithdrawConsequence::ReducedToZero(rest)
				}
			} else {
				T::Assets::can_withdraw(asset, who, amount)
			}
		} else {
			WithdrawConsequence::NoFunds
		}
	}
}
