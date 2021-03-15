#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::StorageMap;
use primitives::{Balance, CurrencyId, RATE_DECIMAL};
use sp_runtime::{traits::Zero, DispatchResult};
use sp_std::prelude::*;
use sp_std::result;

use crate::util::*;
use crate::*;

use pallet_ocw_oracle;

impl<T: Config> Pallet<T> {
    /// This calculates interest accrued from the last checkpointed block
    /// up to the current block and writes new checkpoint to storage.
    pub fn accrue_interest(currency_id: &CurrencyId) -> DispatchResult {
        // Read the previous values out of storage
        let cash_prior = Self::get_total_cash(currency_id.clone());
        let borrows_prior = Self::total_borrows(currency_id);

        // Calculate the current borrow interest rate
        Self::update_borrow_rate(currency_id.clone(), cash_prior, borrows_prior, 0)?;

        /*
         * Compound protocol:
         * Calculate the interest accumulated into borrows and reserves and the new index:
         *  simpleInterestFactor = borrowRate * blockDelta
         *  interestAccumulated = simpleInterestFactor * totalBorrows
         *  totalBorrowsNew = interestAccumulated + totalBorrows
         *  totalReservesNew = interestAccumulated * reserveFactor + totalReserves
         *  borrowIndexNew = simpleInterestFactor * borrowIndex + borrowIndex
         */

        let borrow_rate_per_block = BorrowRate::<T>::get(currency_id);
        let interest_accumulated = mul_then_div(borrows_prior, borrow_rate_per_block, RATE_DECIMAL)
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;
        let total_borrows_new = interest_accumulated
            .checked_add(borrows_prior)
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;
        let borrow_index = Self::borrow_index(currency_id);
        let borrow_index_new = mul_then_div_then_add(
            borrow_index,
            borrow_rate_per_block,
            RATE_DECIMAL,
            borrow_index,
        )
        .ok_or(Error::<T>::CalcAccrueInterestFailed)?;

        TotalBorrows::<T>::insert(currency_id, total_borrows_new);
        BorrowIndex::<T>::insert(currency_id, borrow_index_new);

        Ok(())
    }

    pub fn get_total_cash(currency_id: CurrencyId) -> Balance {
        T::Currency::free_balance(currency_id, &Self::account_id())
    }

    /// Sender supplies assets into the market and receives cTokens in exchange
    ///
    /// Ensured atomic.
    #[transactional]
    pub fn mint_internal(
        who: &T::AccountId,
        currency_id: &CurrencyId,
        mint_amount: Balance,
    ) -> DispatchResult {
        let exchange_rate = Self::exchange_rate(currency_id);
        let collateral = mul_then_div(mint_amount, RATE_DECIMAL, exchange_rate)
            .ok_or(Error::<T>::CalcCollateralFailed)?;

        AccountCollateral::<T>::try_mutate(
            currency_id,
            who,
            |collateral_balance| -> DispatchResult {
                let new_balance = collateral_balance
                    .checked_add(collateral)
                    .ok_or(Error::<T>::CollateralOverflow)?;
                *collateral_balance = new_balance;
                Ok(())
            },
        )?;

        TotalSupply::<T>::try_mutate(currency_id, |total_balance| -> DispatchResult {
            let new_balance = total_balance
                .checked_add(collateral)
                .ok_or(Error::<T>::CollateralOverflow)?;
            *total_balance = new_balance;
            Ok(())
        })?;

        T::Currency::transfer(currency_id.clone(), who, &Self::account_id(), mint_amount)?;

        Ok(())
    }

    /// Sender redeems cTokens in exchange for the underlying asset
    ///
    /// Ensured atomic.
    #[transactional]
    pub fn redeem_internal(
        who: &T::AccountId,
        currency_id: &CurrencyId,
        redeem_amount: Balance,
    ) -> DispatchResult {
        let exchange_rate = Self::exchange_rate(currency_id);
        let collateral = mul_then_div(redeem_amount, RATE_DECIMAL, exchange_rate)
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
        T::Currency::transfer(currency_id.clone(), &Self::account_id(), who, redeem_amount)?;

        Ok(())
    }

    pub(crate) fn total_borrowed_value(
        borrower: &T::AccountId,
    ) -> result::Result<Balance, Error<T>> {
        let mut total_borrow_value: Balance = 0_u128;

        for currency_id in Currencies::<T>::get().iter() {
            let currency_borrow_amount = Self::borrow_balance_stored(borrower, currency_id)?;
            if currency_borrow_amount.is_zero() {
                continue;
            }

            let (borrow_currency_price, _) = pallet_ocw_oracle::Prices::get(currency_id)
                .ok_or(Error::<T>::OracleCurrencyPriceNotReady)?;
            if borrow_currency_price.is_zero() {
                return Err(Error::<T>::OracleCurrencyPriceNotReady);
            }

            total_borrow_value = currency_borrow_amount
                .checked_mul(borrow_currency_price)
                .and_then(|r| r.checked_add(total_borrow_value))
                .ok_or(Error::<T>::OracleCurrencyPriceNotReady)?;
        }

        return Ok(total_borrow_value);
    }

    pub(crate) fn total_will_borrow_value(
        borrower: &T::AccountId,
        borrow_currency_id: &CurrencyId,
        borrow_amount: Balance,
    ) -> result::Result<Balance, Error<T>> {
        let (borrow_currency_price, _) = pallet_ocw_oracle::Prices::get(borrow_currency_id)
            .ok_or(Error::<T>::OracleCurrencyPriceNotReady)?;
        let mut total_borrow_value = borrow_amount
            .checked_mul(borrow_currency_price)
            .ok_or(Error::<T>::CollateralOverflow)?;

        total_borrow_value = total_borrow_value
            .checked_add(Self::total_borrowed_value(borrower)?)
            .ok_or(Error::<T>::OracleCurrencyPriceNotReady)?;

        return Ok(total_borrow_value);
    }

    pub(crate) fn collateral_asset_value(
        borrower: &T::AccountId,
        currency_id: &CurrencyId,
    ) -> result::Result<Balance, Error<T>> {
        let collateral = AccountCollateral::<T>::get(currency_id, borrower);
        if collateral.is_zero() {
            return Ok(0_u128);
        }

        let collateral_factor = CollateralRate::<T>::get(currency_id);
        let currency_exchange_rate = ExchangeRate::<T>::get(currency_id);

        let (currency_price, _) = pallet_ocw_oracle::Prices::get(currency_id)
            .ok_or(Error::<T>::OracleCurrencyPriceNotReady)?;
        if currency_price.is_zero() {
            return Err(Error::<T>::OracleCurrencyPriceNotReady);
        }

        Ok(collateral
            .checked_mul(collateral_factor)
            .and_then(|r| r.checked_div(RATE_DECIMAL))
            .and_then(|r| r.checked_mul(currency_exchange_rate))
            .and_then(|r| r.checked_div(RATE_DECIMAL))
            .and_then(|r| r.checked_mul(currency_price))
            .ok_or(Error::<T>::CollateralOverflow)?)
    }

    pub(crate) fn total_collateral_asset_value(
        borrower: &T::AccountId,
    ) -> result::Result<Balance, Error<T>> {
        let collateral_assets = AccountCollateralAssets::<T>::get(borrower);
        if collateral_assets.is_empty() {
            return Err(Error::<T>::NoCollateralAsset.into());
        }

        let mut total_asset_value: Balance = 0_u128;
        for currency_id in collateral_assets.iter() {
            total_asset_value = total_asset_value
                .checked_add(Self::collateral_asset_value(borrower, currency_id)?)
                .ok_or(Error::<T>::CollateralOverflow)?;
        }

        return Ok(total_asset_value);
    }

    /// Borrower shouldn't borrow more than what he/she has collateraled in total
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

    /// Sender borrows assets from the protocol to their own address
    ///
    /// Ensured atomic.
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

        T::Currency::transfer(
            currency_id.clone(),
            &Self::account_id(),
            borrower,
            borrow_amount,
        )?;

        AccountBorrows::<T>::insert(
            currency_id,
            borrower,
            BorrowSnapshot {
                principal: account_borrows_new,
                interest_index: Self::borrow_index(currency_id),
            },
        );

        TotalBorrows::<T>::insert(currency_id, total_borrows_new);

        Ok(())
    }

    /// Sender repays their own borrow
    ///
    /// Ensured atomic.
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

        T::Currency::transfer(
            currency_id.clone(),
            borrower,
            &Self::account_id(),
            repay_amount,
        )?;

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
                interest_index: Self::borrow_index(currency_id),
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
                        .checked_add(collateral_asset_value)
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
        if snapshot.principal == 0 || snapshot.interest_index == 0 {
            return Ok(0);
        }
        /* Calculate new borrow balance using the interest index:
         *  recentBorrowBalance = borrower.borrowBalance * market.borrowIndex / borrower.borrowIndex
         */
        let recent_borrow_balance = snapshot
            .principal
            .checked_mul(Self::borrow_index(currency_id))
            .and_then(|r| r.checked_div(snapshot.interest_index))
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
            .checked_sub(account_earned.exchange_rate_prior)
            .and_then(|r| r.checked_mul(collateral))
            .and_then(|r| r.checked_div(RATE_DECIMAL))
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
        liquidate_token: CurrencyId,
        repay_amount: Balance,
        collateral_token: CurrencyId,
    ) -> DispatchResult {
        let account_borrows = Self::borrow_balance_stored(&borrower, &liquidate_token)?;
        if account_borrows < repay_amount {
            return Err(Error::<T>::RepayAmountTooBig.into());
        }

        //calculate collateral_token_sum price
        let collateral_ctoken_amount = AccountCollateral::<T>::get(collateral_token, &borrower);
        let exchange_rate = Self::exchange_rate(collateral_token);
        let (collateral_token_price, _) = pallet_ocw_oracle::Prices::get(collateral_token)
            .ok_or(Error::<T>::OracleCurrencyPriceNotReady)?;

        //the total amount of borrower's collateral token
        let collateral_token_amount = collateral_ctoken_amount
            .checked_mul(exchange_rate)
            .and_then(|r| r.checked_div(RATE_DECIMAL))
            .ok_or(Error::<T>::CollateralOverflow)?;

        //the total price of borrower's collateral token
        let collateral_token_sum_price = collateral_token_amount
            .checked_mul(collateral_token_price)
            .ok_or(Error::<T>::CollateralOverflow)?;

        //calculate liquidate_token_sum
        let (liquidate_token_price, _) = pallet_ocw_oracle::Prices::get(liquidate_token)
            .ok_or(Error::<T>::OracleCurrencyPriceNotReady)?;

        let liquidate_token_sum_price = repay_amount
            .checked_mul(liquidate_token_price)
            .ok_or(Error::<T>::CollateralOverflow)?;

        // the incentive for liquitor and punishment for the borrower
        let liquidation_incentive = LiquidationIncentive::<T>::get(liquidate_token);

        //can not liquidate to Zero

        if collateral_token_sum_price
            .checked_mul(liquidation_incentive)
            .ok_or(Error::<T>::CollateralOverflow)?
            < liquidate_token_sum_price
        {
            return Err(Error::<T>::RepayBigThanCollateral.into());
        }

        // we can only liquidate 50% of the collateral
        let colose_factor = CloseFactor::<T>::get(collateral_token);

        // let real_repay_amount = repay_amount.checked_mul(colose_factor)?;
        // calculate the acutal amount and sum price of collateral currency(like BTC), that can be liquidated
        let real_collateral_token_amount = collateral_token_amount
            .checked_mul(colose_factor)
            .and_then(|r| r.checked_div(RATE_DECIMAL))
            .ok_or(Error::<T>::CollateralOverflow)?;
        let real_collateral_token_sum_price = collateral_token_sum_price
            .checked_mul(colose_factor)
            .and_then(|r| r.checked_div(RATE_DECIMAL))
            .ok_or(Error::<T>::CollateralOverflow)?;

        //calculate real liquidate_token amount that the liquidator should pay
        let real_liquidate_token_repay_amount = real_collateral_token_sum_price
            .checked_mul(liquidation_incentive)
            .and_then(|r| r.checked_div(liquidate_token_price))
            .ok_or(Error::<T>::CollateralOverflow)?;

        //real repay amount should less than repay amount
        if repay_amount < real_liquidate_token_repay_amount {
            return Err(Error::<T>::NotEnoughRepayAmount.into());
        }

        //inside transfer token
        Self::liquidate_repay_borrow_internal(
            &liquidator,
            &borrower,
            &liquidate_token,
            &collateral_token,
            real_liquidate_token_repay_amount,
            real_collateral_token_amount,
        )?;

        Ok(())
    }

    pub fn liquidate_repay_borrow_internal(
        liquidator: &T::AccountId,
        borrower: &T::AccountId,
        liquidate_token: &CurrencyId,
        collateral_token: &CurrencyId,
        liquidate_token_repay_amount: Balance,
        collateral_token_amount: Balance,
    ) -> DispatchResult {
        //1. liquidator repay borrower's debt,
        // transfer from liquidator to module account
        T::Currency::transfer(
            liquidate_token.clone(),
            liquidator,
            &Self::account_id(),
            liquidate_token_repay_amount,
        )?;
        //2. the system will reduce borrower's debt
        let account_borrows = Self::borrow_balance_stored(borrower, liquidate_token)?;
        let account_borrows_new = account_borrows
            .checked_sub(liquidate_token_repay_amount)
            .ok_or(Error::<T>::CalcBorrowBalanceFailed)?;
        let total_borrows = Self::total_borrows(liquidate_token);
        let total_borrows_new = total_borrows
            .checked_sub(liquidate_token_repay_amount)
            .ok_or(Error::<T>::CalcBorrowBalanceFailed)?;
        AccountBorrows::<T>::insert(
            liquidate_token,
            borrower,
            BorrowSnapshot {
                principal: account_borrows_new,
                interest_index: Self::borrow_index(liquidate_token),
            },
        );
        TotalBorrows::<T>::insert(liquidate_token, total_borrows_new);
        //3. the liquidator will receive collateral_token(ctoken) from system
        // (divide borrower's ctoken to liquidator)
        // decrease borrower's ctoken
        let exchange_rate = Self::exchange_rate(collateral_token);
        let collateral_ctoken_amount = collateral_token_amount
            .checked_div(exchange_rate)
            .and_then(|r| r.checked_mul(RATE_DECIMAL))
            .ok_or(Error::<T>::CalcCollateralFailed)?;

        AccountCollateral::<T>::try_mutate(
            collateral_token,
            borrower,
            |collateral_balance| -> DispatchResult {
                let new_balance = collateral_balance
                    .checked_sub(collateral_ctoken_amount)
                    .ok_or(Error::<T>::CollateralTooLow)?;
                *collateral_balance = new_balance;
                Ok(())
            },
        )?;
        // increase liquidator's ctoken
        AccountCollateral::<T>::try_mutate(
            collateral_token,
            liquidator,
            |collateral_balance| -> DispatchResult {
                let new_balance = collateral_balance
                    .checked_add(collateral_ctoken_amount)
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
            liquidate_token.clone(),
            collateral_token.clone(),
            liquidate_token_repay_amount,
            collateral_token_amount,
        ));
        Ok(())
    }
}
