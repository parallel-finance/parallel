#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::all)]

use frame_support::dispatch::Weight;

pub trait WeightInfo {
    fn stake() -> Weight;
    fn unstake() -> Weight;
    fn record_staking_settlement() -> Weight;
    fn settlement() -> Weight;
    fn pop_queue() -> Weight;
}

impl WeightInfo for () {
    fn stake() -> Weight {
        0u64.into()
    }
    fn unstake() -> Weight {
        0u64.into()
    }
    fn record_staking_settlement() -> Weight {
        0u64.into()
    }
    fn settlement() -> Weight {
        0u64.into()
    }
    fn pop_queue() -> Weight {
        10000u64.into()
    }
}
