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
        T::Currency::transfer(CurrencyId::DOT, who, &Self::staking_account_id(), amount)?;
        T::Currency::transfer(CurrencyId::xDOT, &Self::staking_account_id(), who, amount)?;

        Ok(())
    }
    /// Sender redeems xDOTs in exchange for the DOTs
    ///
    /// Ensured atomic.
    #[transactional]
    pub fn unstake_internal(who: &T::AccountId, amount: Balance) -> DispatchResult {
        T::Currency::transfer(CurrencyId::DOT, &Self::staking_account_id(), who, amount)?;
        T::Currency::transfer(CurrencyId::xDOT, who, &Self::staking_account_id(), amount)?;

        Ok(())
    }
}
