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

use crate::*;
use primitives::{Balance, CurrencyId};
use sp_runtime::{
    traits::{CheckedSub, Zero},
    DispatchResult, FixedPointNumber, FixedU128,
};
use sp_std::prelude::*;
use sp_std::result;

impl<T: Config> Pallet<T> {
    #[transactional]
    pub fn mint_internal(
        who: &T::AccountId,
        currency_id: &CurrencyId,
        mint_amount: Balance,
    ) -> DispatchResult {
        let exchange_rate = Self::exchange_rate(currency_id);
        let collateral_amount = calc_collateral_amount(mint_amount, exchange_rate)
            .ok_or(Error::<T>::CalcCollateralFailed)?;

        AccountCollateral::<T>::try_mutate(
            currency_id,
            who,
            |collateral_balance| -> DispatchResult {
                let new_balance = collateral_balance
                    .checked_add(collateral_amount)
                    .ok_or(Error::<T>::CollateralOverflow)?;
                *collateral_balance = new_balance;
                Ok(())
            },
        )?;

        TotalSupply::<T>::try_mutate(currency_id, |total_balance| -> DispatchResult {
            let new_balance = total_balance
                .checked_add(collateral_amount)
                .ok_or(Error::<T>::CollateralOverflow)?;
            *total_balance = new_balance;
            Ok(())
        })?;

        T::Currency::transfer(*currency_id, who, &Self::account_id(), mint_amount)?;

        Ok(())
    }

    #[transactional]
    pub fn redeem_internal(
        who: &T::AccountId,
        currency_id: &CurrencyId,
        redeem_amount: Balance,
    ) -> DispatchResult {
        let exchange_rate = Self::exchange_rate(currency_id);
        let collateral = calc_collateral_amount(redeem_amount, exchange_rate)
            .ok_or(Error::<T>::CalcCollateralFailed)?;

        AccountCollateral::<T>::try_mutate(
            currency_id,
            who,
            |collateral_balance| -> DispatchResult {
                let new_balance = collateral_balance
                    .checked_sub(collateral)
                    .ok_or(Error::<T>::CollateralTooLow)?;
                *collateral_balance = new_balance;
                Ok(())
            },
        )?;

        TotalSupply::<T>::try_mutate(currency_id, |total_balance| -> DispatchResult {
            let new_balance = total_balance
                .checked_sub(collateral)
                .ok_or(Error::<T>::CollateralTooLow)?;
            *total_balance = new_balance;
            Ok(())
        })?;

        // debug::info!("moduleAccountBalance: {:?}", T::Currency::free_balance(currency_id.clone(), &who));
        T::Currency::transfer(*currency_id, &Self::account_id(), who, redeem_amount)?;

        Ok(())
    }

    pub(crate) fn total_borrowed_value(
        borrower: &T::AccountId,
    ) -> result::Result<FixedU128, Error<T>> {
        let mut total_borrow_value: FixedU128 = FixedU128::zero();

        for currency_id in Currencies::<T>::get().iter() {
            let currency_borrow_amount = Self::borrow_balance_stored(borrower, currency_id)?;
            if currency_borrow_amount.is_zero() {
                continue;
            }

            let (borrow_currency_price, _) = T::PriceFeeder::get_price(currency_id)
                .ok_or(Error::<T>::OracleCurrencyPriceNotReady)?;
            if borrow_currency_price.is_zero() {
                return Err(Error::<T>::OracleCurrencyPriceNotReady);
            }

            total_borrow_value = borrow_currency_price
                .checked_mul(&FixedU128::from_inner(currency_borrow_amount))
                .and_then(|r| r.checked_add(&total_borrow_value))
                .ok_or(Error::<T>::OracleCurrencyPriceNotReady)?;
        }

        Ok(total_borrow_value)
    }

    pub(crate) fn total_will_borrow_value(
        borrower: &T::AccountId,
        borrow_currency_id: &CurrencyId,
        borrow_amount: Balance,
    ) -> result::Result<FixedU128, Error<T>> {
        let (borrow_currency_price, _) = T::PriceFeeder::get_price(borrow_currency_id)
            .ok_or(Error::<T>::OracleCurrencyPriceNotReady)?;
        let mut total_borrow_value = borrow_currency_price
            .checked_mul(&FixedU128::from_inner(borrow_amount))
            .ok_or(Error::<T>::CollateralOverflow)?;

        total_borrow_value = total_borrow_value
            .checked_add(&Self::total_borrowed_value(borrower)?)
            .ok_or(Error::<T>::OracleCurrencyPriceNotReady)?;

        Ok(total_borrow_value)
    }

    pub(crate) fn collateral_asset_value(
        borrower: &T::AccountId,
        currency_id: &CurrencyId,
    ) -> result::Result<FixedU128, Error<T>> {
        let collateral = AccountCollateral::<T>::get(currency_id, borrower);
        if collateral.is_zero() {
            return Ok(FixedU128::zero());
        }

        let collateral_factor = CollateralFactor::<T>::get(currency_id);
        let exchange_rate = ExchangeRate::<T>::get(currency_id);

        let (currency_price, _) = T::PriceFeeder::get_price(currency_id)
            .ok_or(Error::<T>::OracleCurrencyPriceNotReady)?;
        if currency_price.is_zero() {
            return Err(Error::<T>::OracleCurrencyPriceNotReady);
        }

        let collateral_amount = exchange_rate
            .checked_mul_int(collateral_factor.mul_floor(collateral))
            .ok_or(Error::<T>::CollateralOverflow)?;

        currency_price
            .checked_mul(&FixedU128::from_inner(collateral_amount))
            .ok_or(Error::<T>::CollateralOverflow)
    }

    pub(crate) fn total_collateral_asset_value(
        borrower: &T::AccountId,
    ) -> result::Result<FixedU128, Error<T>> {
        let collateral_assets = AccountCollateralAssets::<T>::get(borrower);
        if collateral_assets.is_empty() {
            return Err(Error::<T>::NoCollateralAsset);
        }

        let mut total_asset_value: FixedU128 = FixedU128::zero();
        for currency_id in collateral_assets.iter() {
            total_asset_value = total_asset_value
                .checked_add(&Self::collateral_asset_value(borrower, currency_id)?)
                .ok_or(Error::<T>::CollateralOverflow)?;
        }

        Ok(total_asset_value)
    }

    /// Borrower shouldn't borrow more than what he/she has pledged in total
    pub(crate) fn borrow_guard(
        borrower: &T::AccountId,
        borrow_currency_id: &CurrencyId,
        borrow_amount: Balance,
    ) -> DispatchResult {
        let total_will_borrow_value =
            Self::total_will_borrow_value(borrower, borrow_currency_id, borrow_amount)?;
        let total_collateral_asset_value = Self::total_collateral_asset_value(borrower)?;

        if total_collateral_asset_value < total_will_borrow_value {
            return Err(Error::<T>::InsufficientCollateral.into());
        }

        Ok(())
    }

    #[transactional]
    pub fn borrow_internal(
        borrower: &T::AccountId,
        currency_id: &CurrencyId,
        borrow_amount: Balance,
    ) -> DispatchResult {
        Self::borrow_guard(borrower, currency_id, borrow_amount)?;

        let account_borrows = Self::borrow_balance_stored(borrower, currency_id)?;

        let account_borrows_new = account_borrows
            .checked_add(borrow_amount)
            .ok_or(Error::<T>::CalcBorrowBalanceFailed)?;
        let total_borrows = Self::total_borrows(currency_id);
        let total_borrows_new = total_borrows
            .checked_add(borrow_amount)
            .ok_or(Error::<T>::CalcBorrowBalanceFailed)?;

        T::Currency::transfer(*currency_id, &Self::account_id(), borrower, borrow_amount)?;

        AccountBorrows::<T>::insert(
            currency_id,
            borrower,
            BorrowSnapshot {
                principal: account_borrows_new,
                borrow_index: Self::borrow_index(currency_id),
            },
        );

        TotalBorrows::<T>::insert(currency_id, total_borrows_new);

        Ok(())
    }

    #[transactional]
    pub fn repay_borrow_internal(
        borrower: &T::AccountId,
        currency_id: &CurrencyId,
        repay_amount: Balance,
    ) -> DispatchResult {
        let account_borrows = Self::borrow_balance_stored(borrower, currency_id)?;
        if account_borrows < repay_amount {
            return Err(Error::<T>::RepayAmountTooBig.into());
        }

        T::Currency::transfer(*currency_id, borrower, &Self::account_id(), repay_amount)?;

        let account_borrows_new = account_borrows
            .checked_sub(repay_amount)
            .ok_or(Error::<T>::CalcBorrowBalanceFailed)?;
        let total_borrows = Self::total_borrows(currency_id);
        let total_borrows_new = total_borrows
            .checked_sub(repay_amount)
            .ok_or(Error::<T>::CalcBorrowBalanceFailed)?;

        AccountBorrows::<T>::insert(
            currency_id,
            borrower,
            BorrowSnapshot {
                principal: account_borrows_new,
                borrow_index: Self::borrow_index(currency_id),
            },
        );

        TotalBorrows::<T>::insert(currency_id, total_borrows_new);

        Ok(())
    }

    pub fn collateral_asset_internal(
        who: T::AccountId,
        currency_id: CurrencyId,
        enable: bool,
    ) -> result::Result<(), Error<T>> {
        let mut collateral_assets = AccountCollateralAssets::<T>::get(&who);
        if enable {
            if !collateral_assets.iter().any(|c| c == &currency_id) {
                let collateral = AccountCollateral::<T>::get(currency_id, &who);
                if !collateral.is_zero() {
                    collateral_assets.push(currency_id);
                    AccountCollateralAssets::<T>::insert(who.clone(), collateral_assets);
                    Self::deposit_event(Event::<T>::CollateralAssetAdded(who, currency_id));
                } else {
                    return Err(Error::<T>::DepositRequiredBeforeCollateral);
                }
            } else {
                return Err(Error::<T>::AlreadyEnabledCollateral);
            }
        } else {
            if let Some(index) = collateral_assets.iter().position(|c| c == &currency_id) {
                let total_collateral_asset_value = Self::total_collateral_asset_value(&who)?;
                let collateral_asset_value = Self::collateral_asset_value(&who, &currency_id)?;
                let total_borrowed_value = Self::total_borrowed_value(&who)?;

                if total_collateral_asset_value
                    > total_borrowed_value
                        .checked_add(&collateral_asset_value)
                        .ok_or(Error::<T>::CollateralOverflow)?
                {
                    collateral_assets.remove(index);
                    AccountCollateralAssets::<T>::insert(who.clone(), collateral_assets);
                    Self::deposit_event(Event::<T>::CollateralAssetRemoved(who, currency_id));
                } else {
                    return Err(Error::<T>::CollateralDisableActionDenied);
                }
            } else {
                return Err(Error::<T>::AlreadyDisabledCollateral);
            }
        }

        Ok(())
    }

    pub fn borrow_balance_stored(
        who: &T::AccountId,
        currency_id: &CurrencyId,
    ) -> result::Result<Balance, Error<T>> {
        let snapshot: BorrowSnapshot = Self::account_borrows(currency_id, who);
        if snapshot.principal == 0 || snapshot.borrow_index.is_zero() {
            return Ok(0);
        }
        // Calculate new borrow balance using the interest index:
        // recent_borrow_balance = snapshot.principal * borrow_index / snapshot.borrow_index
        let recent_borrow_balance = Self::borrow_index(currency_id)
            .checked_div(&snapshot.borrow_index)
            .and_then(|r| r.checked_mul_int(snapshot.principal))
            .ok_or(Error::<T>::CalcBorrowBalanceFailed)?;

        Ok(recent_borrow_balance)
    }

    pub(crate) fn update_earned_stored(
        who: &T::AccountId,
        currency_id: &CurrencyId,
    ) -> DispatchResult {
        let collateral = AccountCollateral::<T>::get(currency_id, who);
        let exchange_rate = ExchangeRate::<T>::get(currency_id);
        let account_earned = AccountEarned::<T>::get(currency_id, who);
        let total_earned_prior_new = exchange_rate
            .checked_sub(&account_earned.exchange_rate_prior)
            .and_then(|r| r.checked_mul_int(collateral))
            .and_then(|r| r.checked_add(account_earned.total_earned_prior))
            .ok_or(Error::<T>::CalcEarnedFailed)?;

        AccountEarned::<T>::insert(
            currency_id,
            who,
            EarnedSnapshot {
                exchange_rate_prior: exchange_rate,
                total_earned_prior: total_earned_prior_new,
            },
        );

        Ok(())
    }

    /// please note, as bellow:
    /// - liquidate_token is borrower's debt, like DAI/USDT
    /// - collateral_token is borrower's collateral, like BTC/KSM/DOT
    /// - repay_amount is amount of liquidate_token (such as DAI/USDT)
    ///
    /// in this function,
    /// the liquidator will pay liquidate_token from own account to module account,
    /// the system will reduce borrower's debt
    /// the liquidator will receive collateral_token(ctoken) from system (divide borrower's ctoken to liquidator)
    /// then liquidator can decide if withdraw (from ctoken to token)
    pub fn liquidate_borrow_internal(
        liquidator: T::AccountId,
        borrower: T::AccountId,
        liquidate_currency_id: CurrencyId,
        repay_amount: Balance,
        collateral_currency_id: CurrencyId,
    ) -> DispatchResult {
        if borrower == liquidator {
            return Err(Error::<T>::LiquidatorIsBorrower.into());
        }

        let account_borrows = Self::borrow_balance_stored(&borrower, &liquidate_currency_id)?;
        if account_borrows == 0 {
            return Err(Error::<T>::NoBorrowBalance.into());
        }

        // we can only liquidate 50% of the borrows
        let close_factor = CloseFactor::<T>::get(liquidate_currency_id);
        if close_factor.mul_floor(account_borrows) < repay_amount {
            return Err(Error::<T>::RepayAmountTooBig.into());
        }

        //calculate collateral_token_sum price
        let collateral_ctoken_amount =
            AccountCollateral::<T>::get(collateral_currency_id, &borrower);
        let exchange_rate = Self::exchange_rate(collateral_currency_id);

        //the total amount of borrower's collateral token
        let collateral_underlying_amount = exchange_rate
            .checked_mul_int(collateral_ctoken_amount)
            .ok_or(Error::<T>::CollateralOverflow)?;

        let (collateral_token_price, _) = T::PriceFeeder::get_price(&collateral_currency_id)
            .ok_or(Error::<T>::OracleCurrencyPriceNotReady)?;

        //the total value of borrower's collateral token
        let collateral_value = collateral_token_price
            .checked_mul(&FixedU128::from_inner(collateral_underlying_amount))
            .ok_or(Error::<T>::CollateralOverflow)?;

        //calculate liquidate_token_sum
        let (liquidate_token_price, _) = T::PriceFeeder::get_price(&liquidate_currency_id)
            .ok_or(Error::<T>::OracleCurrencyPriceNotReady)?;

        let liquidate_value = liquidate_token_price
            .checked_mul(&FixedU128::from_inner(repay_amount))
            .ok_or(Error::<T>::LiquidateValueOverflow)?;

        // the incentive for liquidator and punishment for the borrower
        let liquidation_incentive = LiquidationIncentive::<T>::get(liquidate_currency_id);
        let discd_collateral_value = collateral_value
            .checked_mul(&liquidation_incentive.into())
            .ok_or(Error::<T>::CalcDiscdCollateralValueFailed)?;

        if discd_collateral_value < liquidate_value {
            return Err(Error::<T>::RepayValueGreaterThanCollateral.into());
        }

        // calculate the collateral will get
        //
        // amount: 1 Unit = 10^12 pico
        // price is for 1 pico: 1$ = FixedU128::saturating_from_rational(1, 10^12)
        // if price is N($) and amount is M(Unit):
        // liquidate_value = price * amount = (N / 10^12) * (M * 10^12) = N * M
        // if liquidate_value >= 340282366920938463463.374607431768211455,
        // FixedU128::saturating_from_integer(liquidate_value) will overflow, so we use form_inner
        // instead of saturating_from_integer, and after calculation use into_inner to get final value.
        let real_collateral_underlying_amount = liquidate_value
            .checked_div(&collateral_token_price)
            .and_then(|a| a.checked_div(&liquidation_incentive.into()))
            .ok_or(Error::<T>::EquivalentCollateralAmountOverflow)?;

        //inside transfer token
        Self::liquidate_repay_borrow_internal(
            &liquidator,
            &borrower,
            &liquidate_currency_id,
            &collateral_currency_id,
            repay_amount,
            real_collateral_underlying_amount.into_inner(),
        )?;

        Ok(())
    }

    pub fn liquidate_repay_borrow_internal(
        liquidator: &T::AccountId,
        borrower: &T::AccountId,
        liquidate_currency_id: &CurrencyId,
        collateral_currency_id: &CurrencyId,
        repay_amount: Balance,
        collateral_underlying_amount: Balance,
    ) -> DispatchResult {
        // 1.liquidator repay borrower's debt,
        // transfer from liquidator to module account
        T::Currency::transfer(
            *liquidate_currency_id,
            liquidator,
            &Self::account_id(),
            repay_amount,
        )?;
        //2. the system will reduce borrower's debt
        let account_borrows = Self::borrow_balance_stored(borrower, liquidate_currency_id)?;
        let account_borrows_new = account_borrows
            .checked_sub(repay_amount)
            .ok_or(Error::<T>::CalcBorrowBalanceFailed)?;
        let total_borrows = Self::total_borrows(liquidate_currency_id);
        let total_borrows_new = total_borrows
            .checked_sub(repay_amount)
            .ok_or(Error::<T>::CalcBorrowBalanceFailed)?;
        AccountBorrows::<T>::insert(
            liquidate_currency_id,
            borrower,
            BorrowSnapshot {
                principal: account_borrows_new,
                borrow_index: Self::borrow_index(liquidate_currency_id),
            },
        );
        TotalBorrows::<T>::insert(liquidate_currency_id, total_borrows_new);

        // 3.the liquidator will receive collateral_token(ctoken) from system
        // (divide borrower's ctoken to liquidator)
        // decrease borrower's ctoken
        let exchange_rate = Self::exchange_rate(collateral_currency_id);
        let collateral_amount = calc_collateral_amount(collateral_underlying_amount, exchange_rate)
            .ok_or(Error::<T>::CalcCollateralFailed)?;

        AccountCollateral::<T>::try_mutate(
            collateral_currency_id,
            borrower,
            |collateral_balance| -> DispatchResult {
                let new_balance = collateral_balance
                    .checked_sub(collateral_amount)
                    .ok_or(Error::<T>::CollateralTooLow)?;
                *collateral_balance = new_balance;
                Ok(())
            },
        )?;
        // increase liquidator's ctoken
        AccountCollateral::<T>::try_mutate(
            collateral_currency_id,
            liquidator,
            |collateral_balance| -> DispatchResult {
                let new_balance = collateral_balance
                    .checked_add(collateral_amount)
                    .ok_or(Error::<T>::CollateralOverflow)?;
                *collateral_balance = new_balance;
                Ok(())
            },
        )?;

        //4. we can decide if withdraw to liquidator (from ctoken to token)
        // Self::redeem_internal(&liquidator, &collateral_token, collateral_token_amount)?;
        // liquidator, borrower,liquidate_token,collateral_token,liquidate_token_repay_amount,collateral_token_amount
        Self::deposit_event(Event::<T>::LiquidationOccur(
            liquidator.clone(),
            borrower.clone(),
            *liquidate_currency_id,
            *collateral_currency_id,
            repay_amount,
            collateral_underlying_amount,
        ));

        Ok(())
    }
}

pub fn calc_collateral_amount(underlying_amount: u128, exchange_rate: Rate) -> Option<u128> {
    exchange_rate
        .reciprocal()
        .and_then(|r| r.checked_mul_int(underlying_amount))
}
