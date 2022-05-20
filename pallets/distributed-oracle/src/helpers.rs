use crate::{AccountOf, Timestamp};
use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use frame_system::Config;
use primitives::{Balance, Price};
use scale_info::TypeInfo;
use sp_std::collections::btree_map::BTreeMap;

// Struct for Repeater
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, Default)]
#[scale_info(skip_type_params(T))]
pub struct Repeater {
    pub staked_balance: Balance,
    pub last_submission: Timestamp,
    pub reward: Balance,
}

// we want to know who has participated in this round
// and we we want to know who is slashed and rewarded when round is done

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct RoundManager<T: Config> {
    pub participated: BTreeMap<AccountOf<T>, u128>,
    pub people_to_slash: BTreeMap<AccountOf<T>, u128>,
    pub people_to_reward: BTreeMap<AccountOf<T>, u128>,
}

impl<T: Config> Default for RoundManager<T> {
    fn default() -> Self {
        Self {
            participated: BTreeMap::new(),
            people_to_slash: BTreeMap::new(),
            people_to_reward: BTreeMap::new(),
        }
    }
}

/// Holds Price Per Round
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct RoundHolder<T: Config> {
    pub agg_price: Price,
    pub mean_price: Price,
    pub round_started_time: Timestamp,
    pub submitters: BTreeMap<AccountOf<T>, (Price, Timestamp)>,
    pub submitter_count: u32,
}

impl<T: Config> Default for RoundHolder<T> {
    fn default() -> Self {
        Self {
            agg_price: Price::default(),
            mean_price: Price::default(),
            round_started_time: Timestamp::default(),
            submitters: BTreeMap::new(),
            submitter_count: u32::default(),
        }
    }
}
