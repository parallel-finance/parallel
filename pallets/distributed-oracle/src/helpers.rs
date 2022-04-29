use crate::{AccountOf, Timestamp};
use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use frame_system::Config;
use primitives::Balance;
use scale_info::TypeInfo;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct OracleDeposit {
    /// The total amount of the stash's balance that we are currently accounting for.
    /// It's just `active` plus all the `unlocking` balances.
    #[codec(compact)]
    pub total: Balance,

    /// Stake Added Unix Time
    pub timestamp: Timestamp,
}

impl Default for OracleDeposit {
    fn default() -> Self {
        Self {
            total: 0_u128,
            timestamp: 0_128,
        }
    }
}

// Struct for Relayer
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Relayer<T: Config> {
    // Owner
    owner: AccountOf<T>,
}

// Struct for Repeater
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Repeater {
    pub staked_balance: Balance,
    pub last_submission: Timestamp,
    pub pending: Balance,
}
