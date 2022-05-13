use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::UnixTime, RuntimeDebug};
use primitives::Timestamp;
use scale_info::TypeInfo;

use crate::{AccountOf, AssetIdOf, BalanceOf, Config};
use sp_runtime::{traits::Zero, ArithmeticError, DispatchError, DispatchResult};

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum StreamStatus {
    // The stream has not completed yet
    // as_collateral:
    // - `false`: the stream is still in progress
    // - `true`: the steam is in progress, but is being used as collateral
    Ongoing { as_collateral: bool },
    // The stream is completed
    // cancelled:
    // - `false`: remaining_balance should be zero
    // - `true`: remaining_balance could be zero (or not be zero)
    Completed { cancelled: bool },
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum StreamKind {
    // Stream was sent by an account
    Send,
    // Stream would be received by an account
    Receive,
    // Stream was `Cancelled` or `Completed`
    Finish,
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
    // The end time of the stream
    pub end_time: Timestamp,
    // The current status of the stream
    pub status: StreamStatus,
    // Whether the stream can be cancelled
    pub cancellable: bool,
}

impl<T: Config> Stream<T> {
    pub fn new(
        deposit: BalanceOf<T>,
        asset_id: AssetIdOf<T>,
        rate_per_sec: BalanceOf<T>,
        sender: AccountOf<T>,
        recipient: AccountOf<T>,
        start_time: Timestamp,
        end_time: Timestamp,
        cancellable: bool,
    ) -> Self {
        Self {
            remaining_balance: deposit,
            deposit,
            asset_id,
            rate_per_sec,
            sender,
            recipient,
            start_time,
            end_time,
            status: StreamStatus::Ongoing {
                as_collateral: false,
            },
            cancellable,
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
        match &self.status {
            StreamStatus::Ongoing { as_collateral: _ } => false,
            StreamStatus::Completed { cancelled: _ } => true,
        }
    }

    pub fn has_started(&self) -> Result<bool, DispatchError> {
        let delta = self.delta_of()? as BalanceOf<T>;

        Ok(!delta.is_zero())
    }

    pub fn has_withdrawn(&self) -> bool {
        self.deposit > self.remaining_balance
    }

    fn claimed_balance(&self) -> Result<BalanceOf<T>, DispatchError> {
        Ok(self
            .deposit
            .checked_sub(self.remaining_balance)
            .ok_or(ArithmeticError::Underflow)?)
    }

    fn duration(&self) -> Result<u64, DispatchError> {
        self.end_time
            .checked_sub(self.start_time)
            .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))
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
            self.status = StreamStatus::Completed { cancelled: false };
        }

        Ok(())
    }

    pub fn try_cancel(&mut self, remaining_balance: BalanceOf<T>) -> DispatchResult {
        self.remaining_balance = remaining_balance;
        self.status = StreamStatus::Completed { cancelled: true };

        Ok(())
    }

    pub fn as_collateral(&mut self) -> DispatchResult {
        self.status = StreamStatus::Ongoing {
            as_collateral: true,
        };
        self.cancellable = false;

        Ok(())
    }

    pub fn delta_of(&self) -> Result<u64, DispatchError> {
        let now = T::UnixTime::now().as_secs();
        if now <= self.start_time {
            Ok(Zero::zero())
        } else if now < self.end_time {
            now.checked_sub(self.start_time)
                .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))
        } else {
            self.end_time
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
        let recipient_balance = if delta == self.duration()?.into() {
            // When stream reaches the end_time, it should return remaining_balance
            // otherwise some amount will be lost
            self.remaining_balance
        } else if self.has_withdrawn() {
            let available_balance = delta
                .checked_mul(self.rate_per_sec)
                .ok_or(ArithmeticError::Overflow)?;

            available_balance
                .checked_sub(self.claimed_balance()?)
                .ok_or(ArithmeticError::Underflow)?
        } else {
            delta
                .checked_mul(self.rate_per_sec)
                .ok_or(ArithmeticError::Overflow)?
        };

        match *who {
            ref _recipient if self.is_recipient(who) => {
                if delta == self.duration()?.into() {
                    Ok(self.remaining_balance)
                } else {
                    Ok(recipient_balance)
                }
            }
            ref _sender if self.is_sender(who) => {
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
