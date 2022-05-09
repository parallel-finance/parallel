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

pub mod currency {
    use primitives::Balance;

    pub const MILLICENTS: Balance = 10_000_000;
    pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
    pub const DOLLARS: Balance = 100 * CENTS;

    pub const EXISTENTIAL_DEPOSIT: u128 = 10 * CENTS; // 0.1 Native Token Balance

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
    }
}

pub mod time {
    use primitives::{BlockNumber, Moment};
    /// This determines the average expected block time that we are targeting.
    /// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
    /// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
    /// up by `pallet_aura` to implement `fn slot_duration()`.
    ///
    /// Change this to adjust the block time.
    pub const MILLISECS_PER_BLOCK: Moment = 12000;

    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

    // Time is measured by number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
}

/// Fee-related.
pub mod fee {
    use frame_support::weights::{
        constants::{ExtrinsicBaseWeight, WEIGHT_PER_SECOND},
        WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    };
    use primitives::Balance;
    use smallvec::smallvec;
    pub use sp_runtime::Perbill;

    /// The block saturation level. Fees will be updates based on this value.
    pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

    /// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
    /// node's balance type.
    ///
    /// This should typically create a mapping between the following ranges:
    ///   - [0, MAXIMUM_BLOCK_WEIGHT]
    ///   - [Balance::min, Balance::max]
    ///
    /// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
    ///   - Setting it to `0` will essentially disable the weight fee.
    ///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
    pub struct WeightToFee;
    impl WeightToFeePolynomial for WeightToFee {
        type Balance = Balance;
        fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
            // in kerria, extrinsic base weight (smallest non-zero weight) is mapped to 2 CENTS
            let p = super::currency::CENTS * 2;
            let q = Balance::from(ExtrinsicBaseWeight::get());
            smallvec![WeightToFeeCoefficient {
                degree: 1,
                negative: false,
                coeff_frac: Perbill::from_rational(p % q, q),
                coeff_integer: p / q,
            }]
        }
    }

    pub fn dot_per_second() -> u128 {
        let base_weight = Balance::from(ExtrinsicBaseWeight::get());
        let base_tx_per_second = (WEIGHT_PER_SECOND as u128) / base_weight;
        let para_per_second = base_tx_per_second * super::currency::CENTS / 10;
        para_per_second / 50
    }
}

/// Parachains-related
pub mod paras {
    pub mod statemint {
        pub const ID: u32 = 1000;
    }

    pub mod acala {
        pub const ID: u32 = 2000;
        pub const ACA_KEY: &[u8] = &[0, 0];
        pub const AUSD_KEY: &[u8] = &[0, 1];
        pub const LDOT_KEY: &[u8] = &[0, 3];
        pub const LCDOT_KEY: &[u8] = &[2, 13];
    }

    pub mod moonbeam {
        pub const ID: u32 = 2004;
        pub const GLMR_KEY: u8 = 10;
    }

    pub mod phala {
        pub const ID: u32 = 2035;
    }

    pub mod interlay {
        pub const ID: u32 = 2032;
        pub const IBTC_KEY: &[u8] = &[0, 1];
        pub const INTR_KEY: &[u8] = &[0, 2];
    }

    pub mod equilibrium {
        pub const ID: u32 = 2011;
    }
}
