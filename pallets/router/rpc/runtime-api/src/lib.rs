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

use codec::Codec;
use primitives::CurrencyId;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    pub trait RouterApi<Balance> where
        Balance: Codec, {
        fn get_best_route(
            amount_in: Balance,
            token_in: CurrencyId,
            token_out: CurrencyId
        ) -> Result<(Vec<CurrencyId>, Balance), DispatchError>;
    }
}
