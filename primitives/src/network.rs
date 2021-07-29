#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;

/// Network type for parallel.
#[derive(Clone, Copy, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum NetworkType {
    Parallel,
    Heiko,
}

impl NetworkType {
    /// Return ss58 address prefix from network type.
    pub fn ss58_addr_format_id(&self) -> u8 {
        match self {
            NetworkType::Heiko => HEIKO_PREFIX,
            NetworkType::Parallel => PARALLEL_PREFIX,
        }
    }
}

pub const HEIKO_PREFIX: u8 = 110;
pub const PARALLEL_PREFIX: u8 = 172;
/// Token symbol of heiko network.
pub const HEIKO_TOKEN: &str = "HKO";
/// Token symbol of parallel network.
pub const PARALLEL_TOKEN: &str = "PARA";
