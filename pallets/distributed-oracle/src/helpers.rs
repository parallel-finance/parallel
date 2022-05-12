use crate::{AccountOf, Timestamp};
use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use frame_system::Config;
use primitives::{Balance, Price};
use scale_info::TypeInfo;
use std::collections::BTreeMap;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, Default)]
#[scale_info(skip_type_params(T))]
pub struct OracleDeposit {
    /// The total amount of the stash's balance that we are currently accounting for.
    /// It's just `active` plus all the `unlocking` balances.
    #[codec(compact)]
    pub total: Balance,

    /// Stake Added Unix Time
    pub timestamp: Timestamp,

    /// Participated rounds
    pub blocks_in_round: u128,
}

// Struct for Repeater
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, Default)]
#[scale_info(skip_type_params(T))]
pub struct Repeater {
    pub staked_balance: Balance,
    pub last_submission: Timestamp,
    pub reward: Balance,
}

// type Participated<T> = (AccountOf<T>, Timestamp);

// we want to know who has participated in this round
// and we we want to know who is slashed and rewarded when round is done

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct RoundManager<T: Config> {
    pub participated: BTreeMap<AccountOf<T>, Timestamp>,
    pub people_to_slash: BTreeMap<AccountOf<T>, Timestamp>,
    pub people_to_reward: BTreeMap<AccountOf<T>, Timestamp>,
    // pub participated: Vec<Participated<T>>,
    // pub people_to_slash: Vec<AccountOf<T>>,
    // pub people_to_reward: Vec<AccountOf<T>>,
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

// impl RoundManager {
//     fn reward_at_round_end(&mut self) {

//     };
//     fn slash_at_round_end(&mut self) {

//     };
// }

// #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
// #[scale_info(skip_type_params(T))]
// pub struct RoundInfo {
//     pub price: Price,
//     pub timestamp: Timestamp,
// }

/// Holds Price Per Round
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct RoundHolder<T: Config> {
    pub avg_price: Price,
    pub submitters: BTreeMap<AccountOf<T>, (Price, Timestamp)>,
}

impl<T: Config> Default for RoundHolder<T> {
    fn default() -> Self {
        Self {
            avg_price: Price::default(),
            submitters: BTreeMap::new(),
        }
    }
}

// impl Default for RoundInfo {
//     fn default() -> Self {
//         Self {
//             price: Price::default(),
//             timestamp: 0
//         }
//     }
// }

// impl<T: Config> Default for RoundInfo {
//     fn default() -> Self {
//         Self {
//             price: Price::default(),
//             timestamp: 0
//         }
//     }
// }

// round starts
// people add prices
// round price acceptance ends
// RoundManager
// checks who didnt respond
// who's prices were 50% greater then median price for round
// who did respond
// Round Manager
// updates slashes
// updates rewards (`pendings_rewards`)
// round ends

// laterrrr
// a repeater comes by and claims_rewards
// moves pendings_rewards into account?
