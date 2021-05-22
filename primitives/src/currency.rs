#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::{convert::Into, prelude::*};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub trait CurrencyInfo {
    fn name(&self) -> &str;
    fn symbol(&self) -> &str;
    fn decimals(&self) -> u32;
}

macro_rules! impl_currency_id {
    ($(#[$meta:meta])*
	$vis:vis enum CurrencyId {
        $($(#[$vmeta:meta])* $symbol:ident($name:expr, $deci:literal),)*
    }) => {
		$(#[$meta])*
		$vis enum CurrencyId {
			$($(#[$vmeta])* $symbol,)*
		}

		impl CurrencyInfo for CurrencyId {
			fn name(&self) -> &str {
				match self {
					$(CurrencyId::$symbol => $name),*
				}
			}
			fn symbol(&self) -> &str {
				match self {
					$(CurrencyId::$symbol => stringify!($symbol),)*
				}
			}
			fn decimals(&self) -> u32 {
				match self {
					$(CurrencyId::$symbol => $deci,)*
				}
			}
		}
    }
}

impl_currency_id! {
    #[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize, Hash))]
    pub enum CurrencyId {
        DOT("Polkadot", 10),
        KSM("Kusama", 12),
        USDT("Tether", 6),
        #[allow(non_camel_case_types)]
        xDOT("Liquid DOT", 18),
        #[allow(non_camel_case_types)]
        xKSM("Liquid KSM", 18),
        Native("Native", 18),
    }
}
