use codec::{Decode, Encode};
use frame_support::traits::tokens::Balance as BalanceT;
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, RuntimeDebug};

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Pool<CurrencyId, Balance> {
    pub base_amount: Balance,
    pub quote_amount: Balance,
    pub root_k_last: Balance,
    pub lp_token_id: CurrencyId,
}

impl<CurrencyId, Balance: BalanceT> Pool<CurrencyId, Balance> {
    pub fn new(lp_token_id: CurrencyId) -> Self {
        Self {
            base_amount: Zero::zero(),
            quote_amount: Zero::zero(),
            root_k_last: Zero::zero(),
            lp_token_id,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.base_amount.is_zero() && self.quote_amount.is_zero()
    }
}
