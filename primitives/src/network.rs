use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;

// Network type for parallel.
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum NetworkType {
    Parallel,
    Heiko,
}

impl NetworkType {
    // Return ss58 address prefix from network type.
    pub fn ss58_addr_format_id(&self) -> u8 {
        match self {
            NetworkType::Heiko => 110,
            NetworkType::Parallel => 172,
        }
    }
}

pub const HEIKO_PREFIX: u8 = 110;
pub const PARALLEL_PREFIX: u8 = 172;
