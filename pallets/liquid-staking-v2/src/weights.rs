#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::dispatch::Weight;

pub trait WeightInfo {
    fn stake() -> Weight;
    fn unstake() -> Weight;
    fn record_staking_settlement() -> Weight;
    fn trigger_new_era() -> Weight;
    fn record_withdrawal_unbond_response() -> Weight;
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
    fn trigger_new_era() -> Weight {
        0u64.into()
    }

    fn record_withdrawal_unbond_response() -> Weight {
        0u64.into()
    }
}
