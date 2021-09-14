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
use sp_runtime::RuntimeDebug;
use sp_std::{
    convert::{Into, TryFrom},
    prelude::*,
};

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

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Hash))]
pub enum PoolAsset {
    Asset(u32),
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Hash))]
pub enum CurrencyOrAsset {
    NativeCurrency,
    Asset(u32),
    PoolAsset(PoolAsset, PoolAsset),
}

impl CurrencyOrAsset {
    pub fn common_asset_id(asset_id_0: Self, asset_id_1: Self) -> Option<Self> {
        let asset_0 = match asset_id_0 {
            CurrencyOrAsset::Asset(symbol) => PoolAsset::Asset(symbol),
            _ => return None,
        };
        let asset_1 = match asset_id_1 {
            CurrencyOrAsset::Asset(symbol) => PoolAsset::Asset(symbol),
            _ => return None,
        };
        Some(CurrencyOrAsset::PoolAsset(asset_0, asset_1))
    }
}
