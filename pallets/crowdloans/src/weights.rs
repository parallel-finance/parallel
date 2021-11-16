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

use frame_support::weights::Weight;
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_crowdloans.
pub trait WeightInfo {
    fn create_vault() -> Weight;
    fn contribute() -> Weight;
    fn participate() -> Weight;
    fn close() -> Weight;
    fn auction_failed() -> Weight;
    fn claim_refund() -> Weight;
    fn slot_expired() -> Weight;
    fn update_reserve_factor() -> Weight;
    fn update_xcm_fees_compensation() -> Weight;
    fn update_xcm_weight() -> Weight;
}

/// Weights for pallet_crowdloans using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_vault() -> Weight {
        10_000 as Weight
    }
    fn contribute() -> Weight {
        10_000 as Weight
    }
    fn participate() -> Weight {
        10_000 as Weight
    }
    fn close() -> Weight {
        10_000 as Weight
    }
    fn auction_failed() -> Weight {
        10_000 as Weight
    }
    fn claim_refund() -> Weight {
        10_000 as Weight
    }
    fn slot_expired() -> Weight {
        10_000 as Weight
    }
    fn update_reserve_factor() -> Weight {
        10_000 as Weight
    }
    fn update_xcm_fees_compensation() -> Weight {
        10_000 as Weight
    }
    fn update_xcm_weight() -> Weight {
        10_000 as Weight
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn create_vault() -> Weight {
        10_000 as Weight
    }
    fn contribute() -> Weight {
        10_000 as Weight
    }
    fn participate() -> Weight {
        10_000 as Weight
    }
    fn close() -> Weight {
        10_000 as Weight
    }
    fn auction_failed() -> Weight {
        10_000 as Weight
    }
    fn claim_refund() -> Weight {
        10_000 as Weight
    }
    fn slot_expired() -> Weight {
        10_000 as Weight
    }
    fn update_reserve_factor() -> Weight {
        10_000 as Weight
    }
    fn update_xcm_fees_compensation() -> Weight {
        10_000 as Weight
    }
    fn update_xcm_weight() -> Weight {
        10_000 as Weight
    }
}
