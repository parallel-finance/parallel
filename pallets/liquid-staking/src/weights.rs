#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::all)]

use frame_support::dispatch::Weight;

pub trait WeightInfo {
    fn stake() -> Weight;
    fn unstake() -> Weight;
    fn record_staking_settlement() -> Weight;
    fn settlement() -> Weight;
    fn set_liquid_currency() -> Weight;
    fn set_staking_currency() -> Weight;
    fn withdraw_unbonded() -> Weight;
    fn payout_stakers() -> Weight;
    fn nominate() -> Weight;
    fn rebond() -> Weight;
    fn unbond() -> Weight;
    fn bond_extra() -> Weight;
    fn bond() -> Weight;
    fn pop_queue() -> Weight;
    fn force_update_transaction_compensation() -> Weight;
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
    fn set_liquid_currency() -> Weight {
        10000u64.into()
    }
    fn set_staking_currency() -> Weight {
        10000u64.into()
    }
    fn withdraw_unbonded() -> Weight {
        10000u64.into()
    }
    fn payout_stakers() -> Weight {
        10000u64.into()
    }
    fn nominate() -> Weight {
        10000u64.into()
    }
    fn rebond() -> Weight {
        10000u64.into()
    }
    fn unbond() -> Weight {
        10000u64.into()
    }
    fn bond_extra() -> Weight {
        10000u64.into()
    }
    fn bond() -> Weight {
        10000u64.into()
    }
    fn pop_queue() -> Weight {
        10000u64.into()
    }
    fn force_update_transaction_compensation() -> Weight {
        10000u64.into()
    }
}
