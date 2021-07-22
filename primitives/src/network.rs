use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;

/// Network type for parallel.
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum NetworkType {
    Parallel,
    Heiko,
}

impl NetworkType {
    /// Return ss58 address prefix from network type.
    pub fn ss58_addr_format_id(&self) -> u8 {
        match self {
            NetworkType::Heiko => 110,
            NetworkType::Parallel => 172,
        }
    }

    /// Return (token_symbol, token_decimal) of this network.
    pub fn token_info(&self) -> (&str, u8) {
        match self {
            NetworkType::Heiko => (HEIKO_TOKEN, 12),
            NetworkType::Parallel => (PARALLEL_TOKEN, 12),
        }
    }
}

pub const HEIKO_PREFIX: u8 = 110;
pub const PARALLEL_PREFIX: u8 = 172;
/// Token symbol of heiko network.
pub const HEIKO_TOKEN: &str = "HKO";
/// Token symbol of parallel network.
pub const PARALLEL_TOKEN: &str = "PARA";
