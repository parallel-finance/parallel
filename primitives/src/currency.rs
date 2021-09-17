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

use codec::{Decode, Encode};
use frame_support::traits::fungibles::{Inspect, Mutate};
use sp_runtime::{traits::Convert, RuntimeDebug, SaturatedConversion};
use sp_std::{
    convert::{Into, TryFrom},
    marker::PhantomData,
    prelude::*,
    result,
};
use xcm::v0::{Error as XcmError, MultiAsset, MultiLocation, Result as XcmResult};
use xcm_executor::traits::{Convert as MoreConvert, MatchesFungible, TransactAsset};

use crate::AssetId;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Hash))]
pub enum TokenSymbol {
    DOT = 0,
    KSM = 1,
    USDT = 2,
    #[allow(non_camel_case_types)]
    xDOT = 3,
    #[allow(non_camel_case_types)]
    xKSM = 4,
    HKO = 5,
    PARA = 6,
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Hash))]
pub enum CurrencyId {
    Token(TokenSymbol),
    LPToken(
        [u8; 32],
        TokenSymbol, // Base asset
        TokenSymbol, // Quote asset
    ),
}

impl CurrencyId {
    pub fn is_token_currency_id(&self) -> bool {
        matches!(self, CurrencyId::Token(_))
    }

    pub fn is_lp_token(&self) -> bool {
        matches!(self, CurrencyId::LPToken(..))
    }
}

impl From<TokenSymbol> for CurrencyId {
    fn from(token_symbol: TokenSymbol) -> CurrencyId {
        CurrencyId::Token(token_symbol)
    }
}

impl TryFrom<CurrencyId> for TokenSymbol {
    type Error = ();
    fn try_from(val: CurrencyId) -> Result<Self, Self::Error> {
        match val {
            CurrencyId::Token(token_symbol) => Ok(token_symbol),
            _ => Err(()),
        }
    }
}

pub struct MultiCurrencyAdapter<
    MultiCurrency,
    Match,
    AccountId,
    AccountIdConvert,
    CurrencyIdConvert,
>(
    PhantomData<(
        MultiCurrency,
        Match,
        AccountId,
        AccountIdConvert,
        CurrencyIdConvert,
    )>,
);

enum Error {
    /// Failed to match fungible.
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
        MultiCurrency: Inspect<AccountId> + Mutate<AccountId>,
        Match: MatchesFungible<MultiCurrency::Balance>,
        AccountId: sp_std::fmt::Debug + Clone,
        AccountIdConvert: MoreConvert<MultiLocation, AccountId>,
        CurrencyIdConvert: Convert<MultiAsset, Option<MultiCurrency::AssetId>>,
    > TransactAsset
    for MultiCurrencyAdapter<MultiCurrency, Match, AccountId, AccountIdConvert, CurrencyIdConvert>
{
    fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> XcmResult {
        match (
            AccountIdConvert::convert_ref(location),
            CurrencyIdConvert::convert(asset.clone()),
            Match::matches_fungible(asset),
        ) {
            // known asset
            (Ok(who), Some(currency_id), Some(amount)) => {
                MultiCurrency::mint_into(currency_id, &who, amount)
                    .map_err(|e| XcmError::FailedToTransactAsset(e.into()))
            }
            // ignore unknown asset
            _ => Ok(()),
        }
    }

    fn withdraw_asset(
        asset: &MultiAsset,
        location: &MultiLocation,
    ) -> result::Result<xcm_executor::Assets, XcmError> {
        let who = AccountIdConvert::convert_ref(location)
            .map_err(|_| XcmError::from(Error::AccountIdConversionFailed))?;
        let currency_id = CurrencyIdConvert::convert(asset.clone())
            .ok_or_else(|| XcmError::from(Error::CurrencyIdConversionFailed))?;
        let amount: MultiCurrency::Balance = Match::matches_fungible(asset)
            .ok_or_else(|| XcmError::from(Error::FailedToMatchFungible))?
            .saturated_into();
        MultiCurrency::burn_from(currency_id, &who, amount)
            .map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

        Ok(asset.clone().into())
    }
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Hash))]
pub enum CurrencyOrAsset {
    NativeCurrency(TokenSymbol),
    Asset(AssetId),
}
