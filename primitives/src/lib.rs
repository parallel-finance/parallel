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

use codec::{Compact, CompactAs, Decode, Encode, Error};
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    FixedU128, MultiSignature, Permill, RuntimeDebug,
};
use sp_std::{convert::Into, prelude::*};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// An index to a block.
pub type BlockNumber = u32;

/// Header type.
/// TODO: remove this or other alias
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type.
/// TODO: remove this or other alias
pub type Block = generic::Block<Header, sp_runtime::OpaqueExtrinsic>;

/// Block ID.
/// TODO: remove this or other alias
pub type BlockId = generic::BlockId<Block>;

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

/// The fixed point number used in loans pallet.
pub type Multiplier = FixedU128;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Hash))]
pub enum CurrencyId {
    DOT = 0,
    KSM = 1,
    USDT = 2,
    #[allow(non_camel_case_types)]
    xDOT = 3,
    #[allow(non_camel_case_types)]
    xKSM = 4,
    Native = 5,
}

pub const TOKEN_DECIMAL: u128 = 1_000_000_000_000_000_000;

pub const RATE_DECIMAL: u128 = 1_000_000_000_000_000_000;

pub const CURRENCY_DECIMAL: u8 = 18;

pub const BLOCK_PER_YEAR: u128 = 5256000;

pub const MIN_PRICE: FixedU128 = FixedU128::from_inner(u128::MIN);

pub type Price = FixedU128;

pub type Timestamp = u64;

pub type PriceDetail = (Price, Timestamp);

pub type TimeStampedPrice = orml_oracle::TimestampedValue<PriceWithDecimal, Moment>;

pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

pub trait PriceFeeder {
    fn get_price(currency_id: &CurrencyId) -> Option<PriceDetail>;
}

pub trait EmergencyPriceFeeder<CurrencyId, PriceWithDecimal> {
    fn set_emergency_price(currency_id: CurrencyId, price: PriceWithDecimal);
    fn reset_emergency_price(currency_id: CurrencyId);
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DataProviderId {
    Aggregated = 0,
}

#[derive(Encode, Decode, Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PriceWithDecimal {
    pub price: Price,
    pub decimal: u8,
}
impl CompactAs for PriceWithDecimal {
    type As = Price;

    fn encode_as(&self) -> &Self::As {
        &self.price
    }

    fn decode_from(price: Self::As) -> Result<Self, Error> {
        Ok(PriceWithDecimal {
            price,
            decimal: CURRENCY_DECIMAL,
        })
    }
}
impl From<Compact<PriceWithDecimal>> for PriceWithDecimal {
    fn from(x: Compact<PriceWithDecimal>) -> PriceWithDecimal {
        x.0
    }
}
