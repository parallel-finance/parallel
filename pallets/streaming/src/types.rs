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
    // stream is cancelled, remaining_balance may be zero
    Cancelled,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum StreamKind {
    // Stream was sent by an account
    Send,
    // Stream would be received by an account
    Receive,
    // Stream was `Cancelled` or `Completed`
    Finish,
    // Can expand Lock and other status if needed
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Stream<T: Config> {
    // The remaining balance can be claimed of the stream
    pub remaining_balance: BalanceOf<T>,
    // The deposit amount of the stream
    pub deposit: BalanceOf<T>,
    // The asset id of the stream
    pub asset_id: AssetIdOf<T>,
    // The rate per-second of the stream
    pub rate_per_sec: BalanceOf<T>,
    // The sender of the stream
    pub sender: AccountOf<T>,
    // The recipient of the stream
    pub recipient: AccountOf<T>,
    // The start time of the stream
    pub start_time: Timestamp,
    // The stop time of the stream
    pub stop_time: Timestamp,
    // The current status of the stream
    pub status: StreamStatus,
}

impl<T: Config> Stream<T> {
    pub fn new(
        deposit: BalanceOf<T>,
        asset_id: AssetIdOf<T>,
        rate_per_sec: BalanceOf<T>,
        sender: AccountOf<T>,
        recipient: AccountOf<T>,
        start_time: Timestamp,
        stop_time: Timestamp,
    ) -> Self {
        Self {
            remaining_balance: deposit,
            deposit,
            asset_id,
            rate_per_sec,
            sender,
            recipient,
            start_time,
            stop_time,
            status: StreamStatus::Ongoing,
        }
    }
    pub fn is_sender(&self, account: &AccountOf<T>) -> bool {
        *account == self.sender
    }

    pub fn is_recipient(&self, account: &AccountOf<T>) -> bool {
        *account == self.recipient
    }

    pub fn sender_balance(&self) -> Result<BalanceOf<T>, DispatchError> {
        self.balance_of(&self.sender)
    }

    pub fn recipient_balance(&self) -> Result<BalanceOf<T>, DispatchError> {
        self.balance_of(&self.recipient)
    }

    pub fn has_finished(&self) -> bool {
        self.status == StreamStatus::Completed || self.status == StreamStatus::Cancelled
    }

    fn claimed_balance(&self) -> Result<BalanceOf<T>, DispatchError> {
        Ok(self
            .deposit
            .checked_sub(self.remaining_balance)
            .ok_or(ArithmeticError::Underflow)?)
    }

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
