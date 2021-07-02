use crate::InterestRateModel;
use primitives::{Rate, Ratio};

/// The current state of a market. For more information, see [Market].
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, PartialEq, codec::Decode, codec::Encode, sp_runtime::RuntimeDebug)]
pub enum MarketState {
    Active,
    Pending,
    Supervision,
}

/// Market.
///
/// A large pool of liquidity where accounts can lend and borrow.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[derive(codec::Decode, codec::Encode, sp_runtime::RuntimeDebug)]
pub struct Market {
    /// The collateral utilization ratio
    pub collateral_factor: Ratio,
    /// Fraction of interest currently set aside for reserves
    pub reserve_factor: Ratio,
    /// The percent, ranging from 0% to 100%, of a liquidatable account's
    /// borrow that can be repaid in a single liquidate transaction.
    pub close_factor: Ratio,
    /// Liquidation incentive ratio
    pub liquidate_incentive: Rate,
    /// Current model being used
    pub rate_model: InterestRateModel,
    pub state: MarketState,
}
