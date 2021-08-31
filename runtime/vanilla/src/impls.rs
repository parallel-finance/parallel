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

use frame_support::traits::{Currency, Imbalance, OnUnbalanced};

pub type NegativeImbalance<T> = <pallet_balances::Pallet<T> as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

/// Logic for the author to get a portion of fees.
pub struct ToStakingPot<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for ToStakingPot<R>
where
    R: pallet_balances::Config + pallet_collator_selection::Config,
    <R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
    fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
        let numeric_amount = amount.peek();
        let staking_pot = <pallet_collator_selection::Pallet<R>>::account_id();
        <pallet_balances::Pallet<R>>::resolve_creating(&staking_pot, amount);
        <frame_system::Pallet<R>>::deposit_event(pallet_balances::Event::Deposit(
            staking_pot,
            numeric_amount,
        ));
    }
}

pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
where
    R: pallet_balances::Config + pallet_collator_selection::Config,
    <R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
        if let Some(mut fees) = fees_then_tips.next() {
            if let Some(tips) = fees_then_tips.next() {
                tips.merge_into(&mut fees);
            }
            <ToStakingPot<R> as OnUnbalanced<_>>::on_unbalanced(fees);
        }
    }
}
