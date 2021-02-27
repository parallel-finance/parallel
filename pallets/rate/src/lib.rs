#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch, traits::Get};
use frame_system::ensure_signed;

use substrate_fixed::types::{U16F16, U32F32};

pub const BLOCK_PER_YEAR: U16F16 = U16F16::from_num(5256000);

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
    trait Store for Module<T: Config> as Rate {

        // MultiplierPerBlock get(fn multipler_per_block): Option<U16F16>;
        // BaseRatePerBlock get(fn base_rate_per_block): Option<U16F16>;
        // JumpMultiplierPerBlock get(fn jump_multiplier_per_block): Option<U16F16>;
        // Kink get(fn kink): Option<U16F16>;
        MultiplierPerBlock get(fn multipler_per_block): U16F16 = U16F16::from_num(1);
        BaseRatePerBlock get(fn base_rate_per_block): U16F16 = U16F16::from_num(1);
        JumpMultiplierPerBlock get(fn jump_multiplier_per_block): U16F16 = U16F16::from_num(1);
        Kink get(fn kink): U16F16 = U16F16::from_num(1);
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        BtcDotUpdated(u64, AccountId),
        NewInterestParams(U16F16, U16F16, U16F16, U16F16),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        Overflow
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;
    }
}

impl<T: Config> Module<T> {
    fn update_jump_rate_model_internal(
        base_rate_per_year: U16F16,
        multiplier_per_year: U16F16,
        jump_multiplier_per_year: U16F16,
        kink: U16F16,
    ) -> dispatch::DispatchResult {
        BaseRatePerBlock::put(
            base_rate_per_year
                .checked_div(BLOCK_PER_YEAR)
                .ok_or(Error::<T>::Overflow)?,
        );
        MultiplierPerBlock::put(
            multiplier_per_year
                .checked_div(
                    BLOCK_PER_YEAR
                        .checked_mul(kink)
                        .ok_or(Error::<T>::Overflow)?,
                )
                .ok_or(Error::<T>::Overflow)?,
        );
        JumpMultiplierPerBlock::put(
            jump_multiplier_per_year
                .checked_div(BLOCK_PER_YEAR)
                .ok_or(Error::<T>::Overflow)?,
        );
        Kink::put(kink);

        Self::deposit_event(RawEvent::NewInterestParams(
            base_rate_per_year,
            multiplier_per_year,
            jump_multiplier_per_year,
            kink,
        ));
        Ok(())
    }
}
