#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::dispatch::Weight;

pub trait WeightInfo {
    fn stake() -> Weight;
    fn unstake() -> Weight;
    fn record_rewards() -> Weight;
    fn set_era_index() -> Weight;
}

impl WeightInfo for () {
    fn stake() -> Weight {
        0u64.into()
    }
    fn unstake() -> Weight {
        0u64.into()
    }
    fn record_rewards() -> Weight {
        0u64.into()
    }
    fn set_era_index() -> Weight {
        0u64.into()
    }
}
