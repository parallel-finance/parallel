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

//! #  (Adapter)

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::traits::fungible::Inspect;
use frame_support::traits::tokens::{DepositConsequence, WithdrawConsequence};
use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{
        tokens::fungibles::{Inspect as Inspects, Mutate, Transfer},
        Get, UnixTime,
    },
    transactional, Blake2_128Concat, PalletId, Twox64Concat,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor, RawOrigin};
use primitives::{AssetId, Balance, Rate};
use sp_runtime::{
    traits::{AccountIdConversion, IntegerSquareRoot, StaticLookup},
    ArithmeticError, DispatchError, Perbill,
};

type AssetIdOf<T> =
    <<T as Config>::Assets as Inspects<<T as frame_system::Config>::AccountId>>::AssetId;
type BalanceOf<T> =
    <<T as Config>::Assets as Inspects<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::tokens::fungible;
    use frame_system::ensure_root;
    use primitives::AssetId;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Assets: Transfer<Self::AccountId> + Inspects<Self::AccountId> + Mutate<Self::AccountId>;

        type Balances: fungible::Inspect<Self::AccountId>
            + fungible::Mutate<Self::AccountId>
            + fungible::Transfer<Self::AccountId>;

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
        todo!()
    }

    fn balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        todo!()
    }

    fn reducible_balance(
        asset: Self::AssetId,
        who: &T::AccountId,
        keep_alive: bool,
    ) -> Self::Balance {
        todo!()
    }

    fn can_deposit(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DepositConsequence {
        todo!()
    }

    fn can_withdraw(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        todo!()
    }
}
