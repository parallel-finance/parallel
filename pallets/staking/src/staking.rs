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
        let mut pending_balances = Self::account_pending_balance(nominator);
        pending_balances.push(PendingBalance {
            balance: amount,
            timestamp: <pallet_timestamp::Pallet<T>>::get(),
        });
        AccountPendingBalance::<T>::insert(nominator, pending_balances);

        Ok(())
    }
    /// Return the pending balance(Dot) to nominator
    ///
    /// Ensured atomic.
    #[transactional]
    pub fn return_pending_balance_internal(
        payer: &T::AccountId,
        nominator: &T::AccountId,
        index: usize,
    ) -> DispatchResult {
        let mut pending_balances = Self::account_pending_balance(nominator);
        if pending_balances.len() <= index {
            return Err(Error::<T>::IndexOverflow.into());
        }
        let pending_balance = pending_balances[index];
        T::Currency::transfer(CurrencyId::DOT, payer, nominator, pending_balance.balance)?;

        // swap and pop
        pending_balances[index] = pending_balances[pending_balances.len() - 1];
        pending_balances.pop();
        AccountPendingBalance::<T>::insert(nominator, pending_balances);

        Ok(())
    }
}
