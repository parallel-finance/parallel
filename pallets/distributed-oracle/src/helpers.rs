use crate::{AccountOf, Timestamp};
use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use frame_system::Config;
// use primitives::{AccountId, Balance, Price, RoundNumber};
use primitives::{AccountId, Balance, Price};
use scale_info::TypeInfo;

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

// // Struct for Relayer
// #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
// #[scale_info(skip_type_params(T))]
// #[codec(mel_bound())]
// pub struct Relayer<T: Config> {
//     // Owner
//     owner: AccountOf<T>,
// }

// Struct for Repeater
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, Default)]
#[scale_info(skip_type_params(T))]
pub struct Repeater {
    pub staked_balance: Balance,
    pub last_submission: Timestamp,
    pub reward: Balance,
}

/// global state that collects and distributes funds to repeaters
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, Default)]
#[scale_info(skip_type_params(T))]
pub struct Coffer {
    pub balance: Balance,
    pub blocks_in_round: u128,
}

type Participated<T> = (AccountOf<T>, Timestamp);

// we want to know who has participated in this round
// and we we want to know who is slashed and rewarded when round is done

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct RoundManager<T: Config> {
    pub participated: Vec<Participated<T>>,
    pub people_to_slash: Vec<AccountOf<T>>,
    pub people_to_reward: Vec<AccountOf<T>>,
}

impl<T: Config> Default for RoundManager<T> {
    fn default() -> Self {
        Self {
            participated: Vec::new(),
            people_to_slash: Vec::new(),
            people_to_reward: Vec::new(),
        }
    }
}

// impl RoundManager {
//     fn reward_at_round_end(&mut self) {

//     };
//     fn slash_at_round_end(&mut self) {

//     };
// }

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Submitter<T: Config> {
    pub submitter: AccountOf<T>,
    pub price: Price,
}

/// Holds Price Per Round
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct PriceHolder<T: Config> {
    pub price: Price,
    pub submitters: Vec<Submitter<T>>,
}

impl<T: Config> Default for PriceHolder<T> {
    fn default() -> Self {
        Self {
            price: Price::default(),
            submitters: Vec::new(),
        }
    }
}

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
