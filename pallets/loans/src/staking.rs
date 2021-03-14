#![cfg_attr(not(feature = "std"), no_std)]

use primitives::{Balance, CurrencyId};
use sp_runtime::DispatchResult;

use crate::*;

impl<T: Config> Pallet<T> {
    /// Sender stakes DOTs to the validator and receives LDOTs in exchange
    ///
    /// Ensured atomic.
    #[transactional]
    pub fn staking_internal(who: &T::AccountId, amount: Balance) -> DispatchResult {
        T::Currency::transfer(CurrencyId::DOT, who, &Self::account_id(), amount)?;
        T::Currency::transfer(CurrencyId::LDOT, &Self::account_id(), who, amount)?;

        Ok(())
    }
    /// Sender redeems LDOTs in exchange for the DOTs
    ///
    /// Ensured atomic.
    #[transactional]
    pub fn stop_staking_internal(who: &T::AccountId, amount: Balance) -> DispatchResult {
        T::Currency::transfer(CurrencyId::DOT, &Self::account_id(), who, amount)?;
        T::Currency::transfer(CurrencyId::LDOT, who, &Self::account_id(), amount)?;

        Ok(())
    }
}
