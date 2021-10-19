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

//! # Currency adapter pallet
//!
//! ## Overview
//!
//! This pallet works like a bridge between pallet-balances & pallet-assets

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::{
    dispatch::DispatchResult,
    traits::{
        tokens::{
            fungible::{Inspect, Mutate, Transfer},
            fungibles::{Inspect as Inspects, Mutate as Mutates, Transfer as Transfers},
            DepositConsequence, WithdrawConsequence,
        },
        Get,
    },
};
use primitives::{Balance, CurrencyId};
use sp_runtime::DispatchError;

type AssetIdOf<T> =
    <<T as Config>::Assets as Inspects<<T as frame_system::Config>::AccountId>>::AssetId;
type BalanceOf<T> =
    <<T as Config>::Assets as Inspects<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Assets: Transfers<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspects<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutates<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        type Balances: Inspect<Self::AccountId, Balance = Balance>
            + Mutate<Self::AccountId, Balance = Balance>
            + Transfer<Self::AccountId, Balance = Balance>;

        #[pallet::constant]
        type GetNativeCurrencyId: Get<AssetIdOf<Self>>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}

impl<T: Config> Inspects<T::AccountId> for Pallet<T> {
    type AssetId = AssetIdOf<T>;
    type Balance = BalanceOf<T>;

    fn total_issuance(asset: Self::AssetId) -> Self::Balance {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::total_issuance()
        } else {
            T::Assets::total_issuance(asset)
        }
    }

    fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::minimum_balance()
        } else {
            T::Assets::minimum_balance(asset)
        }
    }

    fn balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::balance(who)
        } else {
            T::Assets::balance(asset, who)
        }
    }

    fn reducible_balance(
        asset: Self::AssetId,
        who: &T::AccountId,
        keep_alive: bool,
    ) -> Self::Balance {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::reducible_balance(who, keep_alive)
        } else {
            T::Assets::reducible_balance(asset, who, keep_alive)
        }
    }

    fn can_deposit(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DepositConsequence {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::can_deposit(who, amount)
        } else {
            T::Assets::can_deposit(asset, who, amount)
        }
    }

    fn can_withdraw(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::can_withdraw(who, amount)
        } else {
            T::Assets::can_withdraw(asset, who, amount)
        }
    }
}

impl<T: Config> Mutates<T::AccountId> for Pallet<T> {
    fn mint_into(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::mint_into(who, amount)
        } else {
            T::Assets::mint_into(asset, who, amount)
        }
    }

    fn burn_from(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::burn_from(who, amount)
        } else {
            T::Assets::burn_from(asset, who, amount)
        }
    }
}

impl<T: Config> Transfers<T::AccountId> for Pallet<T> {
    fn transfer(
        asset: Self::AssetId,
        source: &T::AccountId,
        dest: &T::AccountId,
        amount: Self::Balance,
        keep_alive: bool,
    ) -> Result<Self::Balance, DispatchError> {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::transfer(source, dest, amount, keep_alive)
        } else {
            T::Assets::transfer(asset, source, dest, amount, keep_alive)
        }
    }
}
