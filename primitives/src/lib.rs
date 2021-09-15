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
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::upper_case_acronyms)]

pub mod currency;
pub mod network;
pub mod tokens;

use codec::{Decode, Encode};
pub use currency::{CurrencyId, TokenSymbol};
use sp_runtime::{
    traits::{CheckedDiv, IdentifyAccount, Verify},
    FixedU128, MultiSignature, Permill, RuntimeDebug,
};
use sp_std::{cmp::Ordering, convert::Into, prelude::*};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on
/// the chain.
pub type Signature = MultiSignature;

/// Alias to the public key used for this chain, actually a `MultiSigner`. Like
/// the signature, this also isn't a fixed size when encoded, as different
/// cryptos have different size public keys.
pub type AccountPublic = <Signature as Verify>::Signer;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of
/// them.
pub type AccountIndex = u32;

/// Index of a transaction in the chain. 32-bit should be plenty.
pub type Index = u32;

/// Index of era in relaychain.
pub type EraIndex = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An instant or duration in time.
pub type Moment = u64;

/// Balance of an account.
pub type Balance = u128;

/// Signed version of Balance
pub type Amount = i128;

/// The fixed point number
pub type Rate = FixedU128;

/// The fixed point number, range from 0 to 1.
pub type Ratio = Permill;

pub type Liquidity = FixedU128;

pub type Shortfall = FixedU128;

pub type Price = FixedU128;

pub type Timestamp = u64;

pub type AssetId = u32;

pub const SECONDS_PER_YEAR: Timestamp = 365 * 24 * 60 * 60;

pub type PriceDetail = (Price, Timestamp);

pub type TimeStampedPrice = orml_oracle::TimestampedValue<PriceWithDecimal, Moment>;

pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DataProviderId {
    Aggregated = 0,
}

#[derive(Encode, Decode, Debug, Default, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PriceWithDecimal {
    pub price: Price,
    pub decimal: u8,
}
impl Ord for PriceWithDecimal {
    fn cmp(&self, other: &Self) -> Ordering {
        if let Some((decimal, other_decimal)) = 10u128
            .checked_pow(self.decimal.into())
            .zip(10u128.checked_pow(other.decimal.into()))
        {
            if let Some((price, other_price)) =
                self.price.checked_div(&FixedU128::from_inner(decimal)).zip(
                    other
                        .price
                        .checked_div(&FixedU128::from_inner(other_decimal)),
                )
            {
                return price.cmp(&other_price);
            }
        }
        self.price.cmp(&other.price)
    }
}
impl PartialOrd for PriceWithDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

////////////////////////////////////////////////////////////////////////////////
pub trait PriceFeeder {
    fn get_price(asset_id: &AssetId) -> Option<PriceDetail>;
}

pub trait EmergencyPriceFeeder<AssetId, PriceWithDecimal> {
    fn set_emergency_price(asset_id: AssetId, price: PriceWithDecimal);
    fn reset_emergency_price(asset_id: AssetId);
}

pub trait ExchangeRateProvider {
    fn get_exchange_rate() -> Rate;
}

pub trait LiquidStakingCurrenciesProvider<AssetId> {
    fn get_staking_currency() -> Option<AssetId>;
    fn get_liquid_currency() -> Option<AssetId>;
}

pub trait AMM<T: frame_system::Config> {
    /// Handles a "trade" on the AMM side for "who".
    /// This will move the `amount_in` funds to the AMM PalletId,
    /// trade `pair.0` to `pair.1` and return a result with the amount
    /// of currency that was sent back to the user.
    fn trade(
        who: &T::AccountId,
        pair: (CurrencyId, CurrencyId),
        amount_in: Balance,
        minimum_amount_out: Balance,
    ) -> Result<Balance, frame_support::pallet_prelude::DispatchError>;
}

pub trait DerivativeProvider<AccountId> {
    fn derivative_account_id(who: AccountId, index: u16) -> AccountId;
}
