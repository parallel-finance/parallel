// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::all)]

use frame_support::weights::Weight;
use sp_runtime::traits::Get;
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_prices.
pub trait WeightInfo {
    fn withdraw() -> Weight;
}

/// use max weight of Assets and Balance,better to rebenchmark later
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn withdraw() -> Weight {
        Weight::from_ref_time(92_361_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(4 as u64))
    }
}

impl WeightInfo for () {
    fn withdraw() -> Weight {
        Weight::from_ref_time(10_000 as u64)
    }
}
