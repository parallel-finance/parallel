#![cfg_attr(not(feature = "std"), no_std)]

use primitives::{Balance, CurrencyId};
use sp_runtime::DispatchResult;

use crate::*;

impl<T: Config> Pallet<T> {
    /// Sender stakes DOTs to the validator and receives xDOTs in exchange
    ///
    /// Ensured atomic.
    #[transactional]
    pub fn stake_internal(who: &T::AccountId, amount: Balance) -> DispatchResult {
        T::Currency::transfer(CurrencyId::DOT, who, &Self::account_id(), amount)?;
        T::Currency::transfer(CurrencyId::xDOT, &Self::account_id(), who, amount)?;

        Ok(())
    }
    /// Sender redeems xDOTs in exchange for pending balance(Dot)
    ///
    /// Ensured atomic.
    #[transactional]
    pub fn unstake_internal(nominator: &T::AccountId, amount: Balance) -> DispatchResult {
        T::Currency::transfer(CurrencyId::xDOT, nominator, &Self::account_id(), amount)?;
        AccountPendingBalance::<T>::try_mutate(nominator, |pending_balance| -> DispatchResult {
            let new_balance = pending_balance
                .checked_add(amount)
                .ok_or(Error::<T>::PendingBalanceOverflow)?;
            *pending_balance = new_balance;
            Ok(())
        })?;

        Ok(())
    }
    /// Return the pending balance(Dot) to nominator
    ///
    /// Ensured atomic.
    #[transactional]
    pub fn return_unstake_balance(payer: &T::AccountId, nominator: &T::AccountId) -> DispatchResult {
        let pending_balance = Self::account_pending_balance(nominator);
        T::Currency::transfer(CurrencyId::DOT, payer, nominator, pending_balance)?;
        AccountPendingBalance::<T>::try_mutate_exists(nominator, |pending_balance| -> DispatchResult {
            *pending_balance = None;
            Ok(())
        })?;

        Ok(())
    }
}
