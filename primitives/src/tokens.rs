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

use crate::CurrencyId;

// Native Token
pub const HKO: CurrencyId = 0;
pub const PARA: CurrencyId = 1;

// Polkadot ecosystem
pub const KSM: CurrencyId = 100;
pub const DOT: CurrencyId = 101;
pub const USDT: CurrencyId = 102;
pub const KUSD: CurrencyId = 103;

// Liquid Staking Derivative
pub const XKSM: CurrencyId = 1000;
pub const XDOT: CurrencyId = 1001;

// Money Market Derivative
pub const PHKO: CurrencyId = 2000;
pub const PPARA: CurrencyId = 2001;
pub const PKSM: CurrencyId = 2100;
pub const PDOT: CurrencyId = 2101;
pub const PUSDT: CurrencyId = 2102;
pub const PKUSD: CurrencyId = 2103;

pub const PXKSM: CurrencyId = 3000;
pub const PXDOT: CurrencyId = 3001;
pub const PCKSM_KSX: CurrencyId = 3100;
pub const PCKSM_SKU: CurrencyId = 3101;
pub const PCKSM_SUB: CurrencyId = 3102;

// Crowdloans Derivative
pub const CKSM_KSX: CurrencyId = 4000;
pub const CKSM_SKU: CurrencyId = 4001;
pub const CKSM_SUB: CurrencyId = 4002;

// Token Registration Information
// +───────────+──────────────+────────────────────+
// | Network   | Token        | Register in block  |
// +───────────+──────────────+────────────────────+
// | Heiko     | HKO          | Native             |
// | Heiko     | PARA         | N/A                |
// | Heiko     | KSM          | N/A                |
// | Heiko     | DOT          | N/A                |
// | Heiko     | USDT         | N/A                |
// | Heiko     | KUSD         | N/A                |
// | Heiko     | XKSM         | N/A                |
// | Heiko     | XDOT         | N/A                |
// | Heiko     | PHKO         | N/A                |
// | Heiko     | PPARA        | N/A                |
// | Heiko     | PKSM         | N/A                |
// | Heiko     | PDOT         | N/A                |
// | Heiko     | PUSDT        | N/A                |
// | Heiko     | PXKSM        | N/A                |
// | Heiko     | PXDOT        | N/A                |
// | Heiko     | PCKSM_KSX    | N/A                |
// | Heiko     | PCKSM_SKU    | N/A                |
// | Heiko     | PCKSM_SUB    | N/A                |
// | Heiko     | CKSM_KSX     | N/A                |
// | Heiko     | CKSM_SKU     | N/A                |
// | Heiko     | CKSM_SUB     | N/A                |
// | Parallel  | HKO          | N/A                |
// | Parallel  | PARA         | Native             |
// | Parallel  | KSM          | N/A                |
// | Parallel  | DOT          | N/A                |
// | Parallel  | USDT         | N/A                |
// | Parallel  | KUSD         | N/A                |
// | Parallel  | XKSM         | N/A                |
// | Parallel  | XDOT         | N/A                |
// | Parallel  | PHKO         | N/A                |
// | Parallel  | PPARA        | N/A                |
// | Parallel  | PKSM         | N/A                |
// | Parallel  | PDOT         | N/A                |
// | Parallel  | PUSDT        | N/A                |
// | Parallel  | PXKSM        | N/A                |
// | Parallel  | PXDOT        | N/A                |
// +──────────+───────────────+────────────────────+
