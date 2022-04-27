use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::tokens::Balance as BalanceT;
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, RuntimeDebug};

#[derive(
    Encode,
    Decode,
    Eq,
    PartialEq,
    Copy,
    Clone,
    RuntimeDebug,
    PartialOrd,
    Ord,
    TypeInfo,
    MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Pool<CurrencyId, Balance, BlockNumber> {
    pub base_amount: Balance,
    pub quote_amount: Balance,
    pub base_amount_last: Balance,
    pub quote_amount_last: Balance,
    pub lp_token_id: CurrencyId,
    pub block_timestamp_last: BlockNumber,
    pub price_0_cumulative_last: Balance,
    pub price_1_cumulative_last: Balance,
}

impl<CurrencyId, Balance: BalanceT, BlockNumber: BalanceT> Pool<CurrencyId, Balance, BlockNumber> {
    pub fn new(lp_token_id: CurrencyId) -> Self {
        Self {
            base_amount: Zero::zero(),
            quote_amount: Zero::zero(),
            base_amount_last: Zero::zero(),
            quote_amount_last: Zero::zero(),
            lp_token_id,
            block_timestamp_last: Zero::zero(),
            price_0_cumulative_last: Zero::zero(),
            price_1_cumulative_last: Zero::zero(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.base_amount.is_zero() && self.quote_amount.is_zero()
    }
}
