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

//! # Common XCM Helper pallet
//!
//! ## Overview
//! This pallet should be in charge of everything XCM related including callbacks and sending XCM calls.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::fungibles::{Inspect, Mutate, Transfer},
};
use primitives::{Balance, CurrencyId};
use sp_runtime::ArithmeticError;
use sp_std::vec;
use xcm::{latest::prelude::*, DoubleEncoded};

pub type AssetIdOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type BalanceOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Assets for deposit/withdraw assets to/from crowdloan account
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
    }

    #[pallet::storage]
    #[pallet::getter(fn xcm_fees_compensation)]
    pub type XcmFeesCompensation<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_reserves)]
    pub type TotalReserves<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);
}

pub trait ParallelXCM<Balance, AssetId, AccountId> {
    fn update_xcm_fees_compensation(fees: Balance);

    fn update_total_reserves(reserves: Balance) -> DispatchResult;

    fn ump_transact(
        call: DoubleEncoded<()>,
        weight: Weight,
        beneficiary: MultiLocation,
        relay_currency: AssetId,
        account_id: AccountId,
    ) -> Result<Xcm<()>, DispatchError>;
}

impl<T: Config> ParallelXCM<BalanceOf<T>, AssetIdOf<T>, T::AccountId> for Pallet<T> {
    fn update_xcm_fees_compensation(fees: BalanceOf<T>) {
        XcmFeesCompensation::<T>::mutate(|v| *v = fees);
    }

    fn update_total_reserves(reserves: BalanceOf<T>) -> DispatchResult {
        TotalReserves::<T>::try_mutate(|b| -> DispatchResult {
            *b = b.checked_add(reserves).ok_or(ArithmeticError::Overflow)?;
            Ok(())
        })
    }

    fn ump_transact(
        call: DoubleEncoded<()>,
        weight: Weight,
        beneficiary: MultiLocation,
        relay_currency: AssetIdOf<T>,
        account_id: T::AccountId,
    ) -> Result<Xcm<()>, DispatchError> {
        let fees = Self::xcm_fees_compensation();
        let asset: MultiAsset = (MultiLocation::here(), fees).into();

        T::Assets::burn_from(relay_currency, &account_id, fees)?;

        TotalReserves::<T>::try_mutate(|b| -> DispatchResult {
            *b = b.checked_sub(fees).ok_or(ArithmeticError::Underflow)?;
            Ok(())
        })?;

        Ok(Xcm(vec![
            WithdrawAsset(MultiAssets::from(asset.clone())),
            BuyExecution {
                fees: asset.clone(),
                weight_limit: Unlimited,
            },
            Transact {
                origin_type: OriginKind::SovereignAccount,
                require_weight_at_most: weight,
                call,
            },
            RefundSurplus,
            DepositAsset {
                assets: asset.into(),
                max_assets: 1,
                beneficiary,
            },
        ]))
    }
}
