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

use crate::AssetId;

pub const HKO: AssetId = 0;
pub const DOT: AssetId = 1;
pub const KSM: AssetId = 100;
pub const XKSM: AssetId = 1000;

// Token Registration Information
// +──────────+────────+────────────────────+
// | Network  | Token  | Register in block  |
// +──────────+────────+────────────────────+
// | Kusama   | HKO    | N/A                |
// | Kusama   | PARA   | N/A                |
// | Kusama   | KSM    | N/A                |
// | Kusama   | XKSM   | N/A                |
// | Kusama   | DOT    | N/A                |
// | Kusama   | XDOT   | N/A                |
// | Kusama   | USDT   | N/A                |
// | Pokadot  | HKO    | N/A                |
// | Pokadot  | PARA   | N/A                |
// | Pokadot  | KSM    | N/A                |
// | Pokadot  | XKSM   | N/A                |
// | Pokadot  | DOT    | N/A                |
// | Pokadot  | XDOT   | N/A                |
// | Pokadot  | USDT   | N/A                |
// +──────────+────────+────────────────────+
