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
use primitives::{switch_relay, ump::*, Balance, CurrencyId, ParaId};
use scale_info::prelude::format;
use sp_runtime::{traits::BlockNumberProvider, ArithmeticError};
use sp_std::{boxed::Box, vec};
use xcm::{latest::prelude::*, DoubleEncoded};

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
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

        /// XCM message sender
        type XcmSender: SendXcm;

        /// Relay network
        #[pallet::constant]
        type RelayNetwork: Get<NetworkId>;

        /// The block number provider
        type BlockNumberProvider: BlockNumberProvider<BlockNumber = Self::BlockNumber>;
    }

    #[pallet::storage]
    #[pallet::getter(fn xcm_fees)]
    pub type XcmFees<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn xcm_weight)]
    pub type XcmWeight<T: Config> = StorageValue<_, XcmWeightMisc<Weight>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_reserves)]
    pub type TotalReserves<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        /// Xcm message send failure
        SendXcmError,
    }
}

pub trait XcmHelper<Balance, AssetId, AccountId> {
    fn update_xcm_fees(fees: Balance);

    fn update_xcm_weight(xcm_weight_misc: XcmWeightMisc<Weight>);

    fn add_reserves(
        relay_currency: AssetId,
        payer: AccountId,
        amount: Balance,
        payee: AccountId,
    ) -> DispatchResult;

    fn ump_transact(
        call: DoubleEncoded<()>,
        weight: Weight,
        beneficiary: MultiLocation,
        relay_currency: AssetId,
        account_id: AccountId,
        xcm_fees_payer: AccountId,
        xcm_fees_payment_strategy: XcmFeesPaymentStrategy,
    ) -> Result<Xcm<()>, DispatchError>;

    fn do_withdraw(
        para_id: ParaId,
        beneficiary: MultiLocation,
        relay_currency: AssetId,
        account_id: AccountId,
        para_account_id: AccountId,
        xcm_fees_payer: AccountId,
        xcm_fees_payment_strategy: XcmFeesPaymentStrategy,
    ) -> Result<(), DispatchError>;

    fn do_contribute(
        para_id: ParaId,
        beneficiary: MultiLocation,
        relay_currency: AssetId,
        account_id: AccountId,
        amount: Balance,
        xcm_fees_payer: AccountId,
        xcm_fees_payment_strategy: XcmFeesPaymentStrategy,
        who: Option<&AccountId>,
    ) -> Result<(), DispatchError>;
}

impl<T: Config> XcmHelper<BalanceOf<T>, AssetIdOf<T>, T::AccountId> for Pallet<T> {
    fn update_xcm_fees(fees: BalanceOf<T>) {
        XcmFees::<T>::mutate(|v| *v = fees);
    }

    fn update_xcm_weight(xcm_weight_misc: XcmWeightMisc<Weight>) {
        XcmWeight::<T>::mutate(|v| *v = xcm_weight_misc);
    }

    fn add_reserves(
        relay_currency: AssetIdOf<T>,
        payer: T::AccountId,
        amount: BalanceOf<T>,
        payee: T::AccountId,
    ) -> DispatchResult {
        T::Assets::transfer(relay_currency, &payer, &payee, amount, false)?;

        TotalReserves::<T>::try_mutate(|b| -> DispatchResult {
            *b = b.checked_add(amount).ok_or(ArithmeticError::Overflow)?;
            Ok(())
        })
    }

    fn ump_transact(
        call: DoubleEncoded<()>,
        weight: Weight,
        beneficiary: MultiLocation,
        relay_currency: AssetIdOf<T>,
        account_id: T::AccountId,
        xcm_fees_payer: T::AccountId,
        xcm_fees_payment_strategy: XcmFeesPaymentStrategy,
    ) -> Result<Xcm<()>, DispatchError> {
        let fees = Self::xcm_fees();
        let asset: MultiAsset = (MultiLocation::here(), fees).into();

        match xcm_fees_payment_strategy {
            XcmFeesPaymentStrategy::Reserves => {
                T::Assets::burn_from(relay_currency, &account_id, fees)?;

                TotalReserves::<T>::try_mutate(|b| -> DispatchResult {
                    *b = b.checked_sub(fees).ok_or(ArithmeticError::Underflow)?;
                    Ok(())
                })?;
            }
            XcmFeesPaymentStrategy::Payer => {
                T::Assets::burn_from(relay_currency, &xcm_fees_payer, fees)?;
            }
        }

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

    fn do_withdraw(
        para_id: ParaId,
        beneficiary: MultiLocation,
        relay_currency: AssetIdOf<T>,
        account_id: T::AccountId,
        para_account_id: T::AccountId,
        xcm_fees_payer: T::AccountId,
        xcm_fees_payment_strategy: XcmFeesPaymentStrategy,
    ) -> Result<(), DispatchError> {
        switch_relay!({
            let call =
                RelaychainCall::<T>::Crowdloans(CrowdloansCall::Withdraw(CrowdloansWithdrawCall {
                    who: para_account_id,
                    index: para_id,
                }));

            let msg = Self::ump_transact(
                call.encode().into(),
                Self::xcm_weight().withdraw_weight,
                beneficiary,
                relay_currency,
                account_id,
                xcm_fees_payer,
                xcm_fees_payment_strategy,
            )?;

            if let Err(_e) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }
        });

        Ok(())
    }

    fn do_contribute(
        para_id: ParaId,
        beneficiary: MultiLocation,
        relay_currency: AssetIdOf<T>,
        account_id: T::AccountId,
        amount: BalanceOf<T>,
        xcm_fees_payer: T::AccountId,
        xcm_fees_payment_strategy: XcmFeesPaymentStrategy,
        who: Option<&T::AccountId>,
    ) -> Result<(), DispatchError> {
        switch_relay!({
            let call =
                RelaychainCall::Utility(Box::new(UtilityCall::BatchAll(UtilityBatchAllCall {
                    calls: vec![
                        RelaychainCall::<T>::System(SystemCall::Remark(SystemRemarkCall {
                            remark: format!(
                                "{:?}#{:?}",
                                T::BlockNumberProvider::current_block_number(),
                                who
                            )
                            .into_bytes(),
                        })),
                        RelaychainCall::<T>::Crowdloans(CrowdloansCall::Contribute(
                            CrowdloansContributeCall {
                                index: para_id,
                                value: amount,
                                signature: None,
                            },
                        )),
                    ],
                })));

            let msg = Self::ump_transact(
                call.encode().into(),
                Self::xcm_weight().contribute_weight,
                beneficiary,
                relay_currency,
                account_id,
                xcm_fees_payer,
                xcm_fees_payment_strategy,
            )?;

            if let Err(_e) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }
        });

        Ok(())
    }
}
