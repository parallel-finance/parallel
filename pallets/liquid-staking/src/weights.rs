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
    fn set_liquid_currency() -> Weight;
    fn set_staking_currency() -> Weight;
}

impl WeightInfo for () {
    fn stake() -> Weight {
        10000u64.into()
    }
    fn unstake() -> Weight {
        10000u64.into()
    }
    fn record_staking_settlement() -> Weight {
        10000u64.into()
    }
    fn settlement() -> Weight {
        10000u64.into()
    }
    fn pop_queue() -> Weight {
        10000u64.into()
    }
    fn set_liquid_currency() -> Weight {
        10000u64.into()
    }
    fn set_staking_currency() -> Weight {
        10000u64.into()
    }
}
