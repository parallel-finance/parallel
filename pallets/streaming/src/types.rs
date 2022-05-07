use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::UnixTime, RuntimeDebug};
use primitives::Timestamp;
use scale_info::TypeInfo;

use crate::{AccountOf, AssetIdOf, BalanceOf, Config};
use sp_runtime::{traits::Zero, ArithmeticError, DispatchError, DispatchResult};

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

impl<T: Config> Stream<T> {
    pub fn try_deduct(&mut self, amount: BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError> {
        self.remaining_balance = self
            .remaining_balance
            .checked_sub(amount)
            .ok_or(ArithmeticError::Underflow)?;

        Ok(self.remaining_balance)
    }

    pub fn try_complete(&mut self) -> DispatchResult {
        if self.remaining_balance.is_zero() {
            self.status = StreamStatus::Completed;
        }

        Ok(())
    }

    fn claimed_balance(&self) -> Result<BalanceOf<T>, DispatchError> {
        Ok(self
            .deposit
            .checked_sub(self.remaining_balance)
            .ok_or(ArithmeticError::Underflow)?)
    }

    pub fn delta_of(&self) -> Result<u64, DispatchError> {
        let now = T::UnixTime::now().as_secs();
        if now <= self.start_time {
            Ok(Zero::zero())
        } else if now < self.stop_time {
            now.checked_sub(self.start_time)
                .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))
        } else {
            self.stop_time
                .checked_sub(self.start_time)
                .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))
        }
    }

    // Measure balance of stream with rate per sec
    pub fn balance_of(&self, who: &AccountOf<T>) -> Result<BalanceOf<T>, DispatchError> {
        let delta = self.delta_of()? as BalanceOf<T>;

        /*
         * If the stream `balance` does not equal `deposit`, it means there have been withdrawals.
         * We have to subtract the total amount withdrawn from the amount of money that has been
         * streamed until now.
         */
        let recipient_balance = if self.deposit > self.remaining_balance {
            let claimed_amount = self.claimed_balance()?;
            let recipient_balance = delta
                .checked_mul(self.rate_per_sec)
                .ok_or(ArithmeticError::Overflow)?;
            recipient_balance
                .checked_sub(claimed_amount)
                .ok_or(ArithmeticError::Underflow)?
        } else {
            delta
                .checked_mul(self.rate_per_sec)
                .ok_or(ArithmeticError::Overflow)?
        };

        match *who {
            ref _recipient if *who == self.recipient => {
                if delta == (self.stop_time - self.start_time).into() {
                    Ok(self.remaining_balance)
                } else {
                    Ok(recipient_balance)
                }
            }
            ref _sender if *who == self.sender => {
                let sender_balance = self
                    .remaining_balance
                    .checked_sub(recipient_balance)
                    .ok_or(ArithmeticError::Underflow)?;

                Ok(sender_balance)
            }
            _ => Ok(Zero::zero()),
        }
    }
}
