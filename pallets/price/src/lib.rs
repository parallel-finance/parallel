#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{pallet_prelude::*, transactional};
use frame_system::pallet_prelude::*;
use orml_traits::{DataFeeder, DataProvider};
use sp_runtime::traits::{CheckedDiv, CheckedMul};
use primitives::{CurrencyId, Price};

pub use module::*;

#[frame_support::pallet]
pub mod module {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The data source, such as Oracle.
        // type Source: DataProvider<CurrencyId, Price> + DataFeeder<CurrencyId, Price, Self::AccountId>;

        #[pallet::constant]
        /// The stable currency id, it should be AUSD in Acala.
        type GetStableCurrencyId: Get<CurrencyId>;

        #[pallet::constant]
        /// The fixed prices of stable currency, it should be 1 USD in Acala.
        type StableCurrencyFixedPrice: Get<Price>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Lock price. \[currency_id, locked_price\]
        LockPrice(CurrencyId, Price),
        /// Unlock price. \[currency_id\]
        UnlockPrice(CurrencyId),
    }

    /// Mapping from currency id to it's locked price
    #[pallet::storage]
    #[pallet::getter(fn locked_price)]
    pub type LockedPrice<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Price, OptionQuery>;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn feed_price(origin: OriginFor<T>, currency_id: CurrencyId, price: Price) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;
            LockedPrice::<T>::insert(currency_id, price);
            Ok(().into())
        }
    }
}

