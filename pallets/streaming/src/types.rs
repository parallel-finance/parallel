use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
use primitives::Timestamp;
use scale_info::TypeInfo;

use crate::{AccountOf, AssetIdOf, BalanceOf, Config};

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum StreamStatus {
    // stream has not finished yet
    Ongoing,
    // stream is completed, remaining_balance should be zero
    Completed,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum StreamKind {
    // Stream was sent by an account
    Send,
    // Stream would be received by an account
    Receive,
    // Can expand Cancel, Lock and other states if needed
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Stream<T: Config>
// where Stream<T>: Encode
{
    // Remaining Balance
    pub remaining_balance: BalanceOf<T>,
    // Deposit
    pub deposit: BalanceOf<T>,
    // Currency Id
    pub asset_id: AssetIdOf<T>,
    // Rate Per Second
    pub rate_per_sec: BalanceOf<T>,
    // Recipient
    pub recipient: AccountOf<T>,
    // Sender
    pub sender: AccountOf<T>,
    // Start Time
    pub start_time: Timestamp,
    // Stop Time
    pub stop_time: Timestamp,
    // The current status of the stream
    pub status: StreamStatus,
}
