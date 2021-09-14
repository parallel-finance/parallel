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

//! Liquidate pallet
//!
//! ## Overview
//!
//! This pallets provides offchain worker to call the liquidate_borrow operation in loans pallet.
//! The collator may opt-in with a pre-funded account. The liquidate strategy is:
//! - find the unhealthy account which has exceeded loans
//! - liquidate the currency with higher loans
//! - liquidator gets any of the affordable collaterals.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{log, pallet_prelude::*, traits::tokens::fungibles::Inspect, transactional};
use frame_system::offchain::{
    AppCrypto, CreateSignedTransaction, ForAny, SendSignedTransaction, Signer,
};
use frame_system::pallet_prelude::*;
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
    offchain::{
        storage_lock::{StorageLock, Time},
        Duration,
    },
    traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedMul, Zero},
    ArithmeticError, FixedPointNumber, FixedPointOperand, FixedU128, Percent, SaturatedConversion,
};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::prelude::*;

pub use pallet::*;
use pallet_loans::WeightInfo;
use primitives::Rate;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"pool");

pub mod crypto {
    use super::KEY_TYPE;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        MultiSignature, MultiSigner,
    };
    app_crypto!(sr25519, KEY_TYPE);

    pub struct AuthId;
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for AuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

/// The miscellaneous information when transforming borrow records.
#[derive(Clone, Debug)]
struct BorrowMisc<AssetId, Balance> {
    currency: AssetId,
    amount: Balance,
    value: FixedU128,
}

/// The miscellaneous information when transforming collateral records.
#[derive(Clone, Debug)]
struct CollateralMisc<AssetId, Balance> {
    currency: AssetId,
    amount: Balance,
    value: FixedU128,
}

type AssetIdOf<T> = <<T as pallet_loans::Config>::Assets as Inspect<
    <T as frame_system::Config>::AccountId,
>>::AssetId;
type BalanceOf<T> = <<T as pallet_loans::Config>::Assets as Inspect<
    <T as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config:
        CreateSignedTransaction<Call<Self>> + frame_system::Config + pallet_loans::Config
    where
        BalanceOf<Self>: FixedPointOperand,
        AssetIdOf<Self>: AtLeast32BitUnsigned,
    {
        /// The account type to perform liquidation
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

        /// The lockdown time when running offchain worker
        #[pallet::constant]
        type LockPeriod: Get<u64>;

        /// The maximum value when liquidate a loan, may different with the loans pallet.
        #[pallet::constant]
        type LiquidateFactor: Get<Percent>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        /// There is no pre-configured currencies
        NoCurrencies,
        /// Failed to get lock to run offchain worker
        GetLockFailed,
        /// No signer available for liquidation, consider adding one via `author_insertKey` RPC.
        NoAvailableAccount,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        BalanceOf<T>: FixedPointOperand,
        AssetIdOf<T>: AtLeast32BitUnsigned,
    {
        fn offchain_worker(block_number: T::BlockNumber) {
            if let Err(e) = Self::liquidate(block_number) {
                log::error!("Failed to run offchain liquidation: {:?}", e);
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        BalanceOf<T>: FixedPointOperand,
        AssetIdOf<T>: AtLeast32BitUnsigned,
    {
        /// The same liquidate_borrow call in loans pallet.
        ///
        /// - `borrower`: the owner of a loan
        /// - `liquidate_currency`: the currency of a loan
        /// - `repay_amount`: the amount will be liquidated
        /// - `collateral_currency`: the currency that liquidator want to get after liquidation.
        #[pallet::weight(T::WeightInfo::liquidate_borrow())]
        #[transactional]
        pub fn liquidate_borrow(
            origin: OriginFor<T>,
            borrower: T::AccountId,
            liquidate_currency: AssetIdOf<T>,
            repay_amount: BalanceOf<T>,
            collateral_currency: AssetIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            pallet_loans::Pallet::<T>::liquidate_borrow_internal(
                who,
                borrower,
                liquidate_currency,
                repay_amount,
                collateral_currency,
            )?;

            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T>
where
    BalanceOf<T>: FixedPointOperand,
    AssetIdOf<T>: AtLeast32BitUnsigned,
{
    fn liquidate(_block_number: T::BlockNumber) -> Result<(), Error<T>> {
        let mut lock = StorageLock::<Time>::with_deadline(
            b"liquidate::lock",
            Duration::from_millis(T::LockPeriod::get()),
        );
        if lock.try_lock().is_err() {
            return Err(Error::<T>::GetLockFailed);
        }

        let signer = Signer::<T, T::AuthorityId>::any_account();
        if !signer.can_sign() {
            return Err(Error::<T>::NoAvailableAccount);
        }

        let aggregated_account_borrows = Self::transform_account_borrows()?;

        let aggregated_account_collatoral = Self::transform_account_collateral()?;

        Self::liquidate_underwater_accounts(
            &signer,
            aggregated_account_borrows,
            aggregated_account_collatoral,
        )?;

        Ok(())
    }

    fn transform_account_borrows() -> Result<
        BTreeMap<T::AccountId, (FixedU128, Vec<BorrowMisc<AssetIdOf<T>, BalanceOf<T>>>)>,
        Error<T>,
    > {
        let result = pallet_loans::AccountBorrows::<T>::iter().fold(
            BTreeMap::<T::AccountId, (FixedU128, Vec<BorrowMisc<AssetIdOf<T>, BalanceOf<T>>>)>::new(
            ),
            |mut acc, (k1, k2, snapshot)| {
                let loans_value = match pallet_loans::Pallet::<T>::get_price(k1).and_then(|price| {
                    let result =
                        pallet_loans::Pallet::<T>::current_balance_from_snapshot(k1, snapshot)
                            .map_err(|_| ArithmeticError::Overflow)?;
                    price
                        .checked_mul(&FixedU128::from_inner(result.saturated_into()))
                        .ok_or_else(|| ArithmeticError::Overflow.into())
                }) {
                    Err(_e) => {
                        acc.remove(&k2);
                        return acc;
                    }
                    Ok(v) => v,
                };
                let default = (FixedU128::zero(), Vec::new());
                let existing = acc.get(&k2).unwrap_or(&default);
                let total_loans_value: FixedU128;
                if let Some(loans_value) = existing.0.checked_add(&loans_value) {
                    total_loans_value = loans_value;
                } else {
                    return acc;
                }
                let mut loans_detail = existing.1.clone();
                loans_detail.push(BorrowMisc {
                    currency: k1,
                    amount: snapshot.principal,
                    value: loans_value,
                });
                acc.insert(k2, (total_loans_value, loans_detail));
                acc
            },
        );

        Ok(result)
    }

    fn transform_account_collateral() -> Result<
        BTreeMap<T::AccountId, (FixedU128, Vec<CollateralMisc<AssetIdOf<T>, BalanceOf<T>>>)>,
        Error<T>,
    > {
        let iter = pallet_loans::AccountDeposits::<T>::iter();
        let result =
            iter.filter(|(.., deposits)| deposits.is_collateral).fold(
                BTreeMap::<
                    T::AccountId,
                    (FixedU128, Vec<CollateralMisc<AssetIdOf<T>, BalanceOf<T>>>),
                >::new(),
                |mut acc, (k1, k2, deposits)| {
                    let balance = match pallet_loans::ExchangeRate::<T>::get(k1)
                        .checked_mul_int(deposits.voucher_balance)
                    {
                        None => {
                            acc.remove(&k2);
                            return acc;
                        }
                        Some(v) => v,
                    };
                    let collateral_value =
                        match pallet_loans::Pallet::<T>::get_price(k1).and_then(|price| {
                            price
                                .checked_mul(&FixedU128::from_inner(balance.saturated_into()))
                                .ok_or_else(|| ArithmeticError::Overflow.into())
                        }) {
                            Err(_e) => {
                                acc.remove(&k2);
                                return acc;
                            }
                            Ok(v) => v,
                        };
                    let collateral_factor: Rate = pallet_loans::Pallet::<T>::market(k1)
                        .map(|elem| elem.collateral_factor.into())
                        .unwrap_or_default();
                    let under_collatoral_value = match collateral_value
                        .checked_mul(&collateral_factor)
                        .ok_or(pallet_loans::Error::<T>::PriceOracleNotReady)
                    {
                        Err(_e) => {
                            acc.remove(&k2);
                            return acc;
                        }
                        Ok(v) => v,
                    };

                    let default = (FixedU128::zero(), Vec::new());
                    let existing = acc.get(&k2).unwrap_or(&default);
                    let totoal_under_collatoral_value = existing.0 + under_collatoral_value;
                    let mut collatoral_detail = existing.1.clone();
                    collatoral_detail.push(CollateralMisc {
                        currency: k1,
                        amount: balance,
                        value: collateral_value,
                    });
                    acc.insert(k2, (totoal_under_collatoral_value, collatoral_detail));
                    acc
                },
            );

        Ok(result)
    }

    fn liquidate_underwater_accounts(
        signer: &Signer<T, <T as Config>::AuthorityId, ForAny>,
        aggregated_account_borrows: BTreeMap<
            T::AccountId,
            (FixedU128, Vec<BorrowMisc<AssetIdOf<T>, BalanceOf<T>>>),
        >,
        aggregated_account_collatoral: BTreeMap<
            T::AccountId,
            (FixedU128, Vec<CollateralMisc<AssetIdOf<T>, BalanceOf<T>>>),
        >,
    ) -> Result<(), Error<T>> {
        aggregated_account_borrows.iter().for_each(
            |(account, (total_loans_value, loans_detail))| {
                let collateral = match aggregated_account_collatoral.get(account) {
                    None => return,
                    Some(v) => v,
                };

                // Borrower should not be liquidated if health factor is higher than 1
                if total_loans_value < &collateral.0 {
                    return;
                }

                let mut new_loans_detail = loans_detail.clone();
                new_loans_detail.sort_by(|a, b| a.value.cmp(&b.value));
                let liquidate_loans = &new_loans_detail[0];

                if let Some(item) = collateral.1.iter().find(|collateral_item| {
                    collateral_item.value.into_inner()
                        >= T::LiquidateFactor::get().mul_floor(liquidate_loans.value.into_inner())
                }) {
                    Self::submit_liquidate_transaction(
                        signer,
                        account.clone(),
                        liquidate_loans.currency,
                        T::LiquidateFactor::get().mul_floor(liquidate_loans.amount),
                        item.currency,
                    );
                }
            },
        );

        Ok(())
    }

    fn submit_liquidate_transaction(
        signer: &Signer<T, <T as Config>::AuthorityId, ForAny>,
        borrower: T::AccountId,
        loan_currency: AssetIdOf<T>,
        liquidation_value: BalanceOf<T>,
        collateral_currency: AssetIdOf<T>,
    ) {
        match signer.send_signed_transaction(|_account| {
            Call::liquidate_borrow(
                borrower.clone(),
                loan_currency,
                liquidation_value,
                collateral_currency,
            )
        }) {
            None => log::info!("No available accounts for liquidation"),
            Some((acc, Ok(()))) => log::info!(
                "[{:?}] Submitted liquidate borrow, borrower: {:?}",
                acc.id,
                borrower
            ),
            Some((acc, Err(e))) => {
                log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e)
            }
        }
    }
}
