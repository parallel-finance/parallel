use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
use primitives::{Balance, CurrencyId};
use scale_info::TypeInfo;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Currency {
    pub id: CurrencyId,
    pub external: bool,
    pub fee: Balance,
}

impl Default for Currency {
    fn default() -> Self {
        Self {
            id: CurrencyId::default(),
            external: false,
            fee: Balance::default(),
        }
    }
}
