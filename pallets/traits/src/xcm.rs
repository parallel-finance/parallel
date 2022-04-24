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

#![cfg_attr(not(feature = "std"), no_std)]

use crate::CurrencyId;

use codec::{Decode, Encode};
use frame_support::{
    traits::{
        tokens::{
            fungibles::{Inspect, Mutate, Transfer},
            BalanceConversion,
        },
        Get,
    },
    weights::{constants::WEIGHT_PER_SECOND, Weight},
};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Convert, Hash as THash, SaturatedConversion, Zero};
use sp_std::{borrow::Borrow, marker::PhantomData, result};
use xcm::latest::{
    prelude::*, AssetId as xcmAssetId, Error as XcmError, Fungibility, Junction::AccountId32,
    MultiLocation, NetworkId,
};
use xcm_builder::TakeRevenue;
use xcm_executor::traits::{
    Convert as MoreConvert, MatchesFungible, MatchesFungibles, TransactAsset, WeightTrader,
};

/// Converter struct implementing `AssetIdConversion` converting a numeric asset ID
/// (must be `TryFrom/TryInto<u128>`) into a MultiLocation Value and Viceversa through
/// an intermediate generic type AssetType.
/// The trait bounds enforce is that the AssetTypeGetter trait is also implemented for
/// AssetIdInfoGetter
pub struct AsAssetType<AssetId, AssetType, AssetIdInfoGetter>(
    PhantomData<(AssetId, AssetType, AssetIdInfoGetter)>,
);
impl<AssetId, AssetType, AssetIdInfoGetter> xcm_executor::traits::Convert<MultiLocation, AssetId>
    for AsAssetType<AssetId, AssetType, AssetIdInfoGetter>
where
    AssetId: Clone,
    AssetType: From<MultiLocation> + Into<Option<MultiLocation>> + Clone,
    AssetIdInfoGetter: AssetTypeGetter<AssetId, AssetType>,
{
    fn convert_ref(id: impl Borrow<MultiLocation>) -> Result<AssetId, ()> {
        if let Some(asset_id) = AssetIdInfoGetter::get_asset_id(id.borrow().clone().into()) {
            Ok(asset_id)
        } else {
            Err(())
        }
    }

    fn reverse_ref(what: impl Borrow<AssetId>) -> Result<MultiLocation, ()> {
        if let Some(asset_type) = AssetIdInfoGetter::get_asset_type(what.borrow().clone()) {
            if let Some(location) = asset_type.into() {
                Ok(location)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

/// Instructs how to convert accountId into a MultiLocation
pub struct AccountIdToMultiLocation<AccountId>(sp_std::marker::PhantomData<AccountId>);
impl<AccountId> sp_runtime::traits::Convert<AccountId, MultiLocation>
    for AccountIdToMultiLocation<AccountId>
where
    AccountId: Into<[u8; 32]>,
{
    fn convert(account: AccountId) -> MultiLocation {
        MultiLocation {
            parents: 0,
            interior: X1(AccountId32 {
                network: NetworkId::Any,
                id: account.into(),
            }),
        }
    }
}

// We need to know how to charge for incoming assets
// This takes the first fungible asset, and takes whatever UnitPerSecondGetter establishes
// UnitsToWeightRatio trait, which needs to be implemented by AssetIdInfoGetter
pub struct FirstAssetTrader<
    AssetType: From<MultiLocation> + Clone,
    AssetIdInfoGetter: UnitsToWeightRatio<AssetType>,
    R: TakeRevenue,
>(
    Weight,
    Option<(MultiLocation, u128, u128)>,
    PhantomData<(AssetType, AssetIdInfoGetter, R)>,
);
impl<
        AssetType: From<MultiLocation> + Clone,
        AssetIdInfoGetter: UnitsToWeightRatio<AssetType>,
        R: TakeRevenue,
    > WeightTrader for FirstAssetTrader<AssetType, AssetIdInfoGetter, R>
{
    fn new() -> Self {
        FirstAssetTrader(0, None, PhantomData)
    }

    fn buy_weight(
        &mut self,
        weight: Weight,
        payment: xcm_executor::Assets,
    ) -> Result<xcm_executor::Assets, XcmError> {
        let first_asset = payment
            .fungible_assets_iter()
            .next()
            .ok_or(XcmError::TooExpensive)?;

        // We are only going to check first asset for now. This should be sufficient for simple token
        // transfers. We will see later if we change this.
        match (first_asset.id, first_asset.fun) {
            (xcmAssetId::Concrete(id), Fungibility::Fungible(_)) => {
                let asset_type: AssetType = id.clone().into();
                // Shortcut if we know the asset is not supported
                // This involves the same db read per block, mitigating any attack based on
                // non-supported assets
                if !AssetIdInfoGetter::payment_is_supported(asset_type.clone()) {
                    return Err(XcmError::TooExpensive);
                }

                let units_per_second = AssetIdInfoGetter::get_units_per_second(asset_type)
                    .ok_or(XcmError::TooExpensive)?;
                let amount =
                    units_per_second.saturating_mul(weight as u128) / (WEIGHT_PER_SECOND as u128);

                // We dont need to proceed if the amount is 0
                // For cases (specially tests) where the asset is very cheap with respect
                // to the weight needed
                if amount.is_zero() {
                    return Ok(payment);
                }

                let required = MultiAsset {
                    fun: Fungibility::Fungible(amount),
                    id: xcmAssetId::Concrete(id.clone()),
                };
                let unused = payment
                    .checked_sub(required)
                    .map_err(|_| XcmError::TooExpensive)?;
                self.0 = self.0.saturating_add(weight);

                // In case the asset matches the one the trader already stored before, add
                // to later refund

                // Else we are always going to subtract the weight if we can, but we latter do
                // not refund it

                // In short, we only refund on the asset the trader first successfully was able
                // to pay for an execution
                let new_asset = match self.1.clone() {
                    Some((prev_id, prev_amount, units_per_second)) => {
                        if prev_id == id {
                            Some((id, prev_amount.saturating_add(amount), units_per_second))
                        } else {
                            None
                        }
                    }
                    None => Some((id, amount, units_per_second)),
                };

                // Due to the trait bound, we can only refund one asset.
                if let Some(new_asset) = new_asset {
                    self.0 = self.0.saturating_add(weight);
                    self.1 = Some(new_asset);
                };

                Ok(unused)
            }
            _ => Err(XcmError::TooExpensive),
        }
    }

    fn refund_weight(&mut self, weight: Weight) -> Option<MultiAsset> {
        if let Some((id, prev_amount, units_per_second)) = self.1.clone() {
            let weight = weight.min(self.0);
            self.0 -= weight;
            let amount = units_per_second * (weight as u128) / (WEIGHT_PER_SECOND as u128);
            self.1 = Some((
                id.clone(),
                prev_amount.saturating_sub(amount),
                units_per_second,
            ));
            Some(MultiAsset {
                fun: Fungibility::Fungible(amount),
                id: xcmAssetId::Concrete(id),
            })
        } else {
            None
        }
    }
}

/// Deal with spent fees, deposit them as dictated by R
impl<
        AssetType: From<MultiLocation> + Clone,
        AssetIdInfoGetter: UnitsToWeightRatio<AssetType>,
        R: TakeRevenue,
    > Drop for FirstAssetTrader<AssetType, AssetIdInfoGetter, R>
{
    fn drop(&mut self) {
        if let Some((id, amount, _)) = self.1.clone() {
            R::take_revenue((id, amount).into());
        }
    }
}

// Defines the trait to obtain a generic AssetType from a generic AssetId and viceversa
pub trait AssetTypeGetter<AssetId, AssetType> {
    // Get asset type from assetId
    fn get_asset_type(asset_id: AssetId) -> Option<AssetType>;

    // Get assetId from assetType
    fn get_asset_id(asset_type: AssetType) -> Option<AssetId>;
}

// Defines the trait to obtain the units per second of a give asset_type for local execution
// This parameter will be used to charge for fees upon asset_type deposit
pub trait UnitsToWeightRatio<AssetType> {
    // Whether payment in a particular asset_type is suppotrted
    fn payment_is_supported(asset_type: AssetType) -> bool;
    // Get units per second from asset type
    fn get_units_per_second(asset_type: AssetType) -> Option<u128>;
}

/// XCM fee depositor to which we implement the TakeRevenue trait
/// It receives a fungibles::Mutate implemented argument, a matcher to convert MultiAsset into
/// AssetId and amount, and the fee receiver account
pub struct XcmFeesToAccount<Assets, Matcher, AccountId, ReceiverAccount>(
    PhantomData<(Assets, Matcher, AccountId, ReceiverAccount)>,
);
impl<
        Assets: Mutate<AccountId>,
        Matcher: MatchesFungibles<Assets::AssetId, Assets::Balance>,
        AccountId: Clone,
        ReceiverAccount: Get<AccountId>,
    > TakeRevenue for XcmFeesToAccount<Assets, Matcher, AccountId, ReceiverAccount>
{
    fn take_revenue(revenue: MultiAsset) {
        match Matcher::matches_fungibles(&revenue) {
            Ok((asset_id, amount)) => {
                if !amount.is_zero() {
                    let ok = Assets::mint_into(asset_id, &ReceiverAccount::get(), amount).is_ok();
                    debug_assert!(ok, "`mint_into` cannot generally fail; qed");
                }
            }
            Err(_) => log::debug!(
                target: "xcm",
                "take revenue failed matching fungible"
            ),
        }
    }
}

// Our AssetType. For now we only handle Xcm Assets
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum AssetType {
    Xcm(MultiLocation),
}

impl Default for AssetType {
    fn default() -> Self {
        Self::Xcm(MultiLocation::here())
    }
}

impl From<MultiLocation> for AssetType {
    fn from(location: MultiLocation) -> Self {
        Self::Xcm(location)
    }
}

impl From<AssetType> for Option<MultiLocation> {
    fn from(asset: AssetType) -> Option<MultiLocation> {
        match asset {
            AssetType::Xcm(location) => Some(location),
        }
    }
}

// Implementation on how to retrieve the AssetId from an AssetType
// We simply hash the AssetType and take the lowest 32 bits
impl From<AssetType> for CurrencyId {
    fn from(asset: AssetType) -> CurrencyId {
        match asset {
            AssetType::Xcm(id) => {
                let mut result: [u8; 4] = [0u8; 4];
                let hash: H256 = id.using_encoded(BlakeTwo256::hash);
                result.copy_from_slice(&hash.as_fixed_bytes()[0..4]);
                u32::from_le_bytes(result)
            }
        }
    }
}

// How to convert from CurrencyId to MultiLocation
pub struct CurrencyIdtoMultiLocation<LegacyAssetConverter, ForeignAssetConverter>(
    sp_std::marker::PhantomData<(LegacyAssetConverter, ForeignAssetConverter)>,
);
impl<LegacyAssetConverter, ForeignAssetConverter>
    sp_runtime::traits::Convert<CurrencyId, Option<MultiLocation>>
    for CurrencyIdtoMultiLocation<LegacyAssetConverter, ForeignAssetConverter>
where
    LegacyAssetConverter: Convert<CurrencyId, Option<MultiLocation>>,
    ForeignAssetConverter: xcm_executor::traits::Convert<MultiLocation, CurrencyId>,
{
    fn convert(currency_id: CurrencyId) -> Option<MultiLocation> {
        let mut multi_location = LegacyAssetConverter::convert(currency_id);
        multi_location = match multi_location {
            Some(_) => multi_location,
            None => ForeignAssetConverter::reverse_ref(&currency_id).ok(),
        };
        multi_location
    }
}

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
