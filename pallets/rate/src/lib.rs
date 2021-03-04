#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch};
use frame_system::ensure_signed;
use primitives::Balance;
use primitives::CurrencyId;
use sp_runtime::traits::Zero;
use sp_std::prelude::*;

const BLOCK_PER_YEAR: u128 = 5256000;
// const BLOCK_PER_YEAR: u128 = 2102400;
const DECIMAL: u128 = 1_000_000_000_000_000_000;

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
    trait Store for Module<T: Config> as Rate {

        MultiplierPerBlock get(fn multipler_per_block): Option<u128>;
        BaseRatePerBlock get(fn base_rate_per_block): Option<u128>;
        JumpMultiplierPerBlock get(fn jump_multiplier_per_block): Option<u128>;
        Kink get(fn kink): Option<u128>;

        BorrowRate get(fn borrow_rate): map hasher(blake2_128_concat) CurrencyId => u128;
        SupplyRate get(fn supply_rate): map hasher(blake2_128_concat) CurrencyId => u128;
        UtilityRate get(fn utility_rate): map hasher(blake2_128_concat) CurrencyId => u128;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        NewInterestParams2(AccountId, u128, u128, u128, u128),
        NewInterestParams(u128, u128, u128, u128),
        BorrowRateUpdated(CurrencyId, u128),
        SupplyRateUpdated(CurrencyId, u128),
        UtilityRateUpdated(CurrencyId, u128),
        Test(u128),
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

        #[weight = 10_000]
        pub fn update_jump_rate_model_tx(
            origin,
            base_rate_per_year: Balance,
            multiplier_per_year: Balance,
            jump_multiplier_per_year: Balance,
            kink: Balance,
        ) -> dispatch::DispatchResult {

            let who = ensure_signed(origin)?;

            let base = base_rate_per_year.checked_div(BLOCK_PER_YEAR).unwrap();

            let multiplier =  multiplier_per_year
            .checked_mul(DECIMAL)
            .and_then(|r| r.checked_div(BLOCK_PER_YEAR.checked_mul(kink).unwrap()))
            .unwrap();

            let jump =  jump_multiplier_per_year.checked_div(BLOCK_PER_YEAR).unwrap();
            BaseRatePerBlock::put(
                base
            );
            MultiplierPerBlock::put(
                multiplier
            );
            JumpMultiplierPerBlock::put(
                jump
            );
            Kink::put(kink);

            Self::deposit_event(RawEvent::NewInterestParams(
                base,
                multiplier,
                jump,
                kink,
            ));
            Ok(())
        }


        #[weight = 10_000]
        pub fn update_borrow_rate_tx(origin, currency_id: CurrencyId, cash: Balance, borrows: Balance, reserves: Balance) -> dispatch::DispatchResult {
            let _who = ensure_signed(origin)?;

            let util = Self::utilization_rate
            (cash, borrows, reserves);
            UtilityRate::insert(currency_id, util);
            Self::deposit_event(RawEvent::UtilityRateUpdated(currency_id, util));

            let multiplier_per_block =MultiplierPerBlock::get().unwrap();
            let base_rate_per_block = BaseRatePerBlock::get().unwrap();
            let kink = Kink::get().unwrap();
            let jump_multiplier_per_block = Self::to_decimal(JumpMultiplierPerBlock::get());

            if util <= kink {
                let rate = util.checked_mul(multiplier_per_block)
                    .and_then(|r| r.checked_div(DECIMAL))
                    .and_then(|r| r.checked_add(base_rate_per_block))
                    .unwrap();

                Self::insert_borrow_rate(currency_id, rate);

            } else {
                let normal_rate = kink.checked_mul(multiplier_per_block)
                .and_then(|r| r.checked_div(DECIMAL))
                .and_then(|r| r.checked_add(base_rate_per_block)).unwrap();

                let excess_util = util.saturating_sub(kink);
                let rate = excess_util
                    .checked_mul(jump_multiplier_per_block)
                    .and_then(|r| r.checked_add(normal_rate))
                    .unwrap();

                Self::insert_borrow_rate(currency_id, rate);
            }
            Ok(())
        }

        #[weight = 10_000]
        pub fn update_supply_rate_tx(
            origin,
            currency_id: CurrencyId,
            cash: Balance,
            borrows: Balance,
            reserves: Balance,
            reserve_factor_mantissa: Balance,
        ) -> dispatch::DispatchResult {
            let _who = ensure_signed(origin)?;
            let one_minus_reserve_factor = u128::from(DECIMAL).saturating_sub(reserve_factor_mantissa);

            let borrow_rate =  BorrowRate::get(currency_id);
            let rate_to_pool = Self::to_decimal(borrow_rate.checked_mul(one_minus_reserve_factor));

            let rate = Self::to_decimal(Self::utilization_rate
                (cash, borrows, reserves)
                .checked_mul(rate_to_pool));
            Self::insert_supply_rate(currency_id, rate);
            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    fn insert_borrow_rate(currency_id: CurrencyId, rate: u128) {
        BorrowRate::insert(currency_id, rate.clone());
        Self::deposit_event(RawEvent::BorrowRateUpdated(currency_id, rate));
    }

    fn insert_supply_rate(currency_id: CurrencyId, rate: u128) {
        SupplyRate::insert(currency_id, rate.clone());
        Self::deposit_event(RawEvent::SupplyRateUpdated(currency_id, rate));
    }

    fn to_decimal(n: Option<u128>) -> u128 {
        n.and_then(|r| r.checked_div(DECIMAL)).unwrap()
    }

    pub fn utilization_rate(cash: Balance, borrows: Balance, reserves: Balance) -> u128 {
        // Utilization rate is 0 when there are no borrows
        if borrows.is_zero() {
            return Zero::zero();
        }

        let total = cash
            .checked_add(borrows)
            .and_then(|r| r.checked_sub(reserves))
            .unwrap();

        borrows
            .checked_mul(DECIMAL)
            .and_then(|r| r.checked_div(total))
            .unwrap()
    }

    pub fn update_jump_rate_model(
        base_rate_per_year: Balance,
        multiplier_per_year: Balance,
        jump_multiplier_per_year: Balance,
        kink: Balance,
    ) -> dispatch::DispatchResult {
        let base = base_rate_per_year.checked_div(BLOCK_PER_YEAR).unwrap();

        let multiplier = multiplier_per_year
            .checked_mul(DECIMAL)
            .and_then(|r| r.checked_div(BLOCK_PER_YEAR.checked_mul(kink).unwrap()))
            .unwrap();

        let jump = jump_multiplier_per_year
            .checked_div(BLOCK_PER_YEAR)
            .unwrap();

        BaseRatePerBlock::put(base);
        MultiplierPerBlock::put(multiplier);
        JumpMultiplierPerBlock::put(jump);
        Kink::put(kink);

        Self::deposit_event(RawEvent::NewInterestParams(base, multiplier, jump, kink));
        Ok(())
    }

    pub fn update_borrow_rate(
        currency_id: CurrencyId,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
    ) -> dispatch::DispatchResult {
        let util = Self::utilization_rate(cash, borrows, reserves);
        UtilityRate::insert(currency_id, util);
        Self::deposit_event(RawEvent::UtilityRateUpdated(currency_id, util));

        let multiplier_per_block = MultiplierPerBlock::get().unwrap();
        let base_rate_per_block = BaseRatePerBlock::get().unwrap();
        let kink = Kink::get().unwrap();
        let jump_multiplier_per_block = Self::to_decimal(JumpMultiplierPerBlock::get());

        if util <= kink {
            let rate = util
                .checked_mul(multiplier_per_block)
                .and_then(|r| r.checked_div(DECIMAL))
                .and_then(|r| r.checked_add(base_rate_per_block))
                .unwrap();

            Self::insert_borrow_rate(currency_id, rate);
        } else {
            let normal_rate = kink
                .checked_mul(multiplier_per_block)
                .and_then(|r| r.checked_div(DECIMAL))
                .and_then(|r| r.checked_add(base_rate_per_block))
                .unwrap();

            let excess_util = util.saturating_sub(kink);
            let rate = excess_util
                .checked_mul(jump_multiplier_per_block)
                .and_then(|r| r.checked_add(normal_rate))
                .unwrap();

            Self::insert_borrow_rate(currency_id, rate);
        }
        Ok(())
    }

    pub fn update_supply_rate(
        currency_id: CurrencyId,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
        reserve_factor_mantissa: Balance,
    ) -> dispatch::DispatchResult {
        let one_minus_reserve_factor = u128::from(DECIMAL).saturating_sub(reserve_factor_mantissa);

        let borrow_rate = BorrowRate::get(currency_id);
        let rate_to_pool = Self::to_decimal(borrow_rate.checked_mul(one_minus_reserve_factor));

        let rate = Self::to_decimal(
            Self::utilization_rate(cash, borrows, reserves).checked_mul(rate_to_pool),
        );
        Self::insert_supply_rate(currency_id, rate);

        Ok(())
    }
}
