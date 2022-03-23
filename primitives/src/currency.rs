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

use frame_support::{
    log,
    traits::{
        fungibles::{Inspect, Mutate, Transfer},
        tokens::BalanceConversion,
        Get,
    },
};
use sp_runtime::{
    traits::{Convert, Zero},
    SaturatedConversion,
};
use sp_std::{convert::Into, marker::PhantomData, prelude::*, result};
use xcm::latest::prelude::*;
use xcm_executor::traits::{Convert as MoreConvert, MatchesFungible, TransactAsset};

pub struct MultiCurrencyAdapter<
    MultiCurrency,
    Match,
    AccountId,
    Balance,
    AccountIdConvert,
    CurrencyIdConvert,
    NativeCurrencyId,
    ExistentialDeposit,
    GiftAccount,
    GiftConvert,
>(
    PhantomData<(
        MultiCurrency,
        Match,
        AccountId,
        Balance,
        AccountIdConvert,
        CurrencyIdConvert,
        NativeCurrencyId,
        ExistentialDeposit,
        GiftAccount,
        GiftConvert,
    )>,
);

enum Error {
    /// Failed to match fungible.
    #[allow(dead_code)]
    FailedToMatchFungible,
    /// `MultiLocation` to `AccountId` Conversion failed.
    AccountIdConversionFailed,
    /// `CurrencyId` conversion failed.
    CurrencyIdConversionFailed,
}

impl From<Error> for XcmError {
    fn from(e: Error) -> Self {
        match e {
            Error::FailedToMatchFungible => {
                XcmError::FailedToTransactAsset("FailedToMatchFungible")
            }
            Error::AccountIdConversionFailed => {
                XcmError::FailedToTransactAsset("AccountIdConversionFailed")
            }
            Error::CurrencyIdConversionFailed => {
                XcmError::FailedToTransactAsset("CurrencyIdConversionFailed")
            }
        }
    }
}

impl<
        MultiCurrency: Inspect<AccountId, Balance = Balance>
            + Mutate<AccountId, Balance = Balance>
            + Transfer<AccountId, Balance = Balance>,
        Match: MatchesFungible<MultiCurrency::Balance>,
        AccountId: sp_std::fmt::Debug + Clone,
        Balance: frame_support::traits::tokens::Balance,
        AccountIdConvert: MoreConvert<MultiLocation, AccountId>,
        CurrencyIdConvert: Convert<MultiAsset, Option<MultiCurrency::AssetId>>,
        NativeCurrencyId: Get<MultiCurrency::AssetId>,
        ExistentialDeposit: Get<Balance>,
        GiftAccount: Get<AccountId>,
        GiftConvert: BalanceConversion<Balance, MultiCurrency::AssetId, Balance>,
    > TransactAsset
    for MultiCurrencyAdapter<
        MultiCurrency,
        Match,
        AccountId,
        Balance,
        AccountIdConvert,
        CurrencyIdConvert,
        NativeCurrencyId,
        ExistentialDeposit,
        GiftAccount,
        GiftConvert,
    >
{
    fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> XcmResult {
        match (
            AccountIdConvert::convert_ref(location),
            CurrencyIdConvert::convert(asset.clone()),
            Match::matches_fungible(asset),
        ) {
            // known asset
            (Ok(who), Some(currency_id), Some(amount)) => {
                if let MultiAsset {
                    id:
                        AssetId::Concrete(MultiLocation {
                            parents: 1,
                            interior: Here,
                        }),
                    ..
                } = asset
                {
                    let gift_account = GiftAccount::get();
                    let native_currency_id = NativeCurrencyId::get();
                    let gift_amount =
                        GiftConvert::to_asset_balance(amount.saturated_into(), currency_id)
                            .unwrap_or_else(|_| Zero::zero());
                    let beneficiary_native_balance =
                        MultiCurrency::reducible_balance(native_currency_id, &who, true);
                    let reducible_balance =
                        MultiCurrency::reducible_balance(native_currency_id, &gift_account, false);

                    if !gift_amount.is_zero()
                        && reducible_balance >= gift_amount
                        && beneficiary_native_balance < gift_amount
                    {
                        let diff =
                            ExistentialDeposit::get() + gift_amount - beneficiary_native_balance;
                        if let Err(e) = MultiCurrency::transfer(
                            native_currency_id,
                            &gift_account,
                            &who,
                            diff,
                            false,
                        ) {
                            log::error!(
                                target: "xcm::deposit_asset",
                                "who: {:?}, currency_id: {:?}, amount: {:?}, native_currency_id: {:?}, gift_amount: {:?}, err: {:?}",
                                who,
                                currency_id,
                                amount,
                                native_currency_id,
                                diff,
                                e
                            );
                        }
                    }
                }

                MultiCurrency::mint_into(currency_id, &who, amount)
                    .map_err(|e| XcmError::FailedToTransactAsset(e.into()))
            }
            _ => Err(XcmError::AssetNotFound),
        }
    }

    fn withdraw_asset(
        asset: &MultiAsset,
        location: &MultiLocation,
    ) -> result::Result<xcm_executor::Assets, XcmError> {
        // throw AssetNotFound error here if not match in order to reach the next foreign transact in tuple
        let amount: MultiCurrency::Balance = Match::matches_fungible(asset)
            .ok_or(XcmError::AssetNotFound)?
            .saturated_into();
        let who = AccountIdConvert::convert_ref(location)
            .map_err(|_| XcmError::from(Error::AccountIdConversionFailed))?;
        let currency_id = CurrencyIdConvert::convert(asset.clone())
            .ok_or_else(|| XcmError::from(Error::CurrencyIdConversionFailed))?;
        MultiCurrency::burn_from(currency_id, &who, amount)
            .map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

        Ok(asset.clone().into())
    }
}
