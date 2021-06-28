use crate::InterestRateModel;
use primitives::{Rate, Ratio};

/// The current state of a market. For more information, see [Market].
#[derive(codec::Decode, codec::Encode, sp_runtime::RuntimeDebug)]
pub enum MarketState {
    Active,
    Pending,
    Supervision,
}

/// Market.
///
/// A large pool of liquidity where accounts can lend and borrow.
#[derive(codec::Decode, codec::Encode, sp_runtime::RuntimeDebug)]
pub struct Market {
    collateral_factor: Ratio,
    reserve_factor: Ratio,
    close_factor: Ratio,
    liquidate_incentive: Rate,
    rate_model: InterestRateModel,
    market_state: MarketState,
}
