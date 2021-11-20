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
pub const PCKSM: CurrencyId = 3100;
pub const PCDOT: CurrencyId = 3101;

// Crowdloans Derivative
// TODO: should use different tokens for crowdloan projects
pub const CKSM: CurrencyId = 4000;
pub const CDOT: CurrencyId = 4001;

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
// | Heiko     | PCKSM        | N/A                |
// | Heiko     | PCDOT        | N/A                |
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
// | Parallel  | PCKSM        | N/A                |
// | Parallel  | PCDOT        | N/A                |
// | Parallel  | CKSM         | N/A                |
// | Parallel  | CDOT         | N/A                |
// +──────────+───────────────+────────────────────+
