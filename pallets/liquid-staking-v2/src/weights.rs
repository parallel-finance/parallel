#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::dispatch::Weight;

pub trait WeightInfo {
    fn record_rewards() -> Weight;
}

impl WeightInfo for () {
    fn record_rewards() -> Weight {
        0u64.into()
    }
}
