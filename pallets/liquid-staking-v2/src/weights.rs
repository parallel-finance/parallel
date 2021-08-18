#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::dispatch::Weight;

pub trait WeightInfo {
    fn record_rewards() -> Weight;
    fn set_era_index() -> Weight;
    fn record_bond_response() -> Weight;
    fn record_unbond_response() -> Weight;
}

impl WeightInfo for () {
    fn record_rewards() -> Weight {
        0u64.into()
    }

    fn set_era_index() -> Weight {
        0u64.into()
    }

    fn record_bond_response() -> Weight {
        0u64.into()
    }

    fn record_unbond_response() -> Weight {
        0u64.into()
    }
}
