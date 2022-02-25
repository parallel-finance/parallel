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
pub const AUSD: CurrencyId = 104;
pub const LC_KSM: CurrencyId = 105;
pub const LC_DOT: CurrencyId = 106;
pub const KAR: CurrencyId = 107;
pub const ACA: CurrencyId = 108;

// Ethereum ecosystem
pub const EUSDT: CurrencyId = 201;
pub const EUSDC: CurrencyId = 202;

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
pub const PXKSM: CurrencyId = 2200;
pub const PXDOT: CurrencyId = 2201;
pub const PEUSDT: CurrencyId = 2501;
pub const PEUSDC: CurrencyId = 2502;

// Crowdloan Derivative
pub const CKSM_15_22: CurrencyId = 100150022;
pub const CDOT_6_13: CurrencyId = 200060013;

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
