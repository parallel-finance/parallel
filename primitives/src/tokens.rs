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
pub const LKSM: CurrencyId = 109;
pub const LDOT: CurrencyId = 110;

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
pub const PAUSD: CurrencyId = 2104;
pub const PLC_KSM: CurrencyId = 2105;
pub const PLC_DOT: CurrencyId = 2106;
pub const PKAR: CurrencyId = 2107;
pub const PACA: CurrencyId = 2108;
pub const PLKSM: CurrencyId = 2109;
pub const PLDOT: CurrencyId = 2110;

pub const PEUSDT: CurrencyId = 2201;
pub const PEUSDC: CurrencyId = 2202;

pub const PXKSM: CurrencyId = 3000;
pub const PXDOT: CurrencyId = 3001;

// Crowdloan Derivative
pub const CKSM_15_22: CurrencyId = 100150022;
pub const CDOT_6_13: CurrencyId = 200060013;
pub const CDOT_7_14: CurrencyId = 200070014;

// Token Registration Information
// +───────────+──────────────+────────────────────+
// | Network   | Token        | Register in block  |
// +───────────+──────────────+────────────────────+
// | Heiko     | HKO          | Native             |
// | Heiko     | KSM          | N/A                |
// | Heiko     | USDT         | N/A                |
// | Heiko     | KUSD         | N/A                |
// | Heiko     | EUSDC        | N/A                |
// | Heiko     | EUSDT        | N/A                |
// | Heiko     | KAR          | N/A                |
// | Heiko     | XKSM         | N/A                |
// | Heiko     | CKSM         | N/A                |
// | Heiko     | LKSM         | N/A                |
// | Heiko     | PHKO         | N/A                |
// | Heiko     | PKSM         | N/A                |
// | Heiko     | PUSDT        | N/A                |
// | Heiko     | PKUSD        | N/A                |
// | Heiko     | PEUSDT       | N/A                |
// | Heiko     | PEUSDC       | N/A                |
// | Heiko     | PKAR         | N/A                |
// | Heiko     | PXKSM        | N/A                |
// | Heiko     | PLKSM        | N/A                |
// | Heiko     | PLCKSM       | N/A                |
// | Heiko     | PCKSM        | N/A                |
// | Parallel  | PARA         | Native             |
// | Parallel  | KSM          | N/A                |
// | Parallel  | DOT          | N/A                |
// | Parallel  | USDT         | N/A                |
// | Parallel  | AUSD         | N/A                |
// | Parallel  | EUSDC        | N/A                |
// | Parallel  | EUSDT        | N/A                |
// | Parallel  | ACA          | N/A                |
// | Parallel  | XDOT         | N/A                |
// | Parallel  | CDOT         | N/A                |
// | Parallel  | LDOT         | N/A                |
// | Parallel  | LCDOT        | N/A                |
// | Parallel  | PPARA        | Native             |
// | Parallel  | PKSM         | N/A                |
// | Parallel  | PDOT         | N/A                |
// | Parallel  | PUSDT        | N/A                |
// | Parallel  | PAUSD        | N/A                |
// | Parallel  | PEUSDC       | N/A                |
// | Parallel  | PEUSDT       | N/A                |
// | Parallel  | PACA         | N/A                |
// | Parallel  | PXDOT        | N/A                |
// | Parallel  | PLDOT        | N/A                |
// | Parallel  | PLCDOT       | N/A                |
// | Parallel  | PCDOT        | N/A                |
// +──────────+───────────────+────────────────────+
