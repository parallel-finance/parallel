#![cfg_attr(not(feature = "std"), no_std)]

use primitives::{Balance, CurrencyId};
use sp_runtime::DispatchResult;
use sp_std::prelude::*;
use sp_std::result;

use crate::*;

const DECIMAL: u128 = 1_000_000_000_000_000_000;

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
        let interest_accumulated = borrow_rate_per_block
            .checked_mul(borrows_prior)
            .and_then(|r| r.checked_div(DECIMAL))
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;
        let total_borrows_new = interest_accumulated
            .checked_add(borrows_prior)
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;
        let borrow_index = Self::borrow_index(currency_id);
        let borrow_index_new = borrow_rate_per_block
            .checked_mul(borrow_index)
            .and_then(|r| r.checked_div(DECIMAL))
            .and_then(|r| r.checked_add(borrow_index))
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
        let collateral = mint_amount
            .checked_mul(DECIMAL)
            .and_then(|r| r.checked_div(exchange_rate))
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
        let collateral = redeem_amount
            .checked_mul(DECIMAL)
            .and_then(|r| r.checked_div(exchange_rate))
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

        T::Currency::transfer(currency_id.clone(), &Self::account_id(), who, redeem_amount)?;

        Ok(())
    }

    /// Borrower shouldn't borrow more than what he/she has collateraled in total
    pub(crate) fn borrow_guard(
        borrower: &T::AccountId,
        currency_id: &CurrencyId,
        borrow_amount: Balance,
    ) -> DispatchResult {
        let collateral_assets = AccountCollateralAssets::<T>::try_get(borrower).unwrap_or(vec![]);
        if collateral_assets.is_empty() {
            return Err(Error::<T>::NoCollateralAsset.into());
        }

        let total_collateral_value = 0_u128;

        for currency_id in collateral_assets.iter() {
            let collateral = AccountCollateral::<T>::get(currency_id, borrower);
            let collateral_factor = CollateralRate::<T>::get(currency_id);

            // TODO: use ocw_oracle price
            let currency_price = 1_u128;
            let collateral_value = collateral
                .checked_mul(collateral_factor)
                .and_then(|r| r.checked_div(DECIMAL))
                .and_then(|r| r.checked_mul(currency_price))
                .ok_or(Error::<T>::CollateralOverflow)?;

            total_collateral_value
                .checked_add(collateral_value)
                .ok_or(Error::<T>::CollateralOverflow)?;
        }

        // TODO: use ocw_oracle price
        let borrow_currency = 1_u128;
        let total_borrow_value = borrow_amount
            .checked_mul(borrow_currency)
            .ok_or(Error::<T>::CollateralOverflow)?;

        if total_collateral_value < total_borrow_value {
            return Err(Error::<T>::InsufficientCash.into());
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
    ) -> DispatchResult {
        if let Ok(mut collateral_assets) = AccountCollateralAssets::<T>::try_get(&who) {
            if enable {
                if !collateral_assets.iter().any(|c| c == &currency_id) {
                    collateral_assets.push(currency_id);
                    AccountCollateralAssets::<T>::insert(who.clone(), collateral_assets);
                    Self::deposit_event(Event::<T>::CollateralAssetAdded(who, currency_id));
                }
            } else {
                if let Some(index) = collateral_assets.iter().position(|c| c == &currency_id) {
                    collateral_assets.remove(index);
                    AccountCollateralAssets::<T>::insert(who.clone(), collateral_assets);
                    Self::deposit_event(Event::<T>::CollateralAssetRemoved(who, currency_id));
                }
            }
        } else {
            AccountCollateralAssets::<T>::insert(
                who,
                if enable { vec![currency_id] } else { vec![] },
            );
        }

        Ok(())
    }

    fn borrow_balance_stored(
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
}
