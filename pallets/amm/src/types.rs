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
pub struct Pool<CurrencyId, Balance, BlockNumber, Reserve> {
    pub base_amount: Reserve,
    pub quote_amount: Reserve,
    pub root_k_last: Balance,
    pub lp_token_id: CurrencyId,
    pub block_timestamp_last: BlockNumber,
    pub price_0_cumulative_last: Balance,
    pub price_1_cumulative_last: Balance,
}

impl<CurrencyId, Balance: BalanceT, BlockNumber: BalanceT, Reserve>
    Pool<CurrencyId, Balance, BlockNumber, Reserve>
{
    pub fn new(lp_token_id: CurrencyId, base_amount: Reserve, quote_amount: Reserve) -> Self {
        Self {
            base_amount,
            quote_amount,
            root_k_last: Zero::zero(),
            lp_token_id,
            block_timestamp_last: Zero::zero(),
            price_0_cumulative_last: Zero::zero(),
            price_1_cumulative_last: Zero::zero(),
        }
    }
}
