#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::collapsible_if)]

use frame_support::pallet_prelude::*;
use frame_support::transactional;
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{traits::AccountIdConversion, ModuleId, RuntimeDebug};
use sp_std::convert::TryInto;
use sp_std::vec::Vec;

pub use module::*;

mod staking;

/// Container for pending balance information
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct PendingBalance<Moment> {
    pub balance: Balance,
    pub timestamp: Moment,
}

#[frame_support::pallet]
pub mod module {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Currency type for deposit/withdraw collateral assets to/from loans
        /// module
        type Currency: MultiCurrencyExtended<
            Self::AccountId,
            CurrencyId = CurrencyId,
            Balance = Balance,
            Amount = Amount,
        >;

        /// The loan's module id, keep all collaterals of CDPs.
        #[pallet::constant]
        type ModuleId: Get<ModuleId>;
    }

    #[pallet::error]
    pub enum Error<T> {
        IndexConvertFailed,
        IndexOverflow,
        NoPendingBalance,
    }

    #[pallet::event]
    pub enum Event<T: Config> {}

    #[pallet::storage]
    #[pallet::getter(fn account_pending_balance)]
    pub type AccountPendingBalance<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, Vec<PendingBalance<T::Moment>>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {}

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            GenesisConfig {}
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            T::Currency::update_balance(
                CurrencyId::xDOT,
                &Pallet::<T>::account_id(),
                1_000_000_000_000_000_000_000_000_000,
            )
            .unwrap();
        }
    }

    #[cfg(feature = "std")]
    impl GenesisConfig {
        /// Direct implementation of `GenesisBuild::build_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn build_storage<T: Config>(&self) -> Result<sp_runtime::Storage, String> {
            <Self as frame_support::traits::GenesisBuild<T>>::build_storage(self)
        }

        /// Direct implementation of `GenesisBuild::assimilate_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn assimilate_storage<T: Config>(
            &self,
            storage: &mut sp_runtime::Storage,
        ) -> Result<(), String> {
            <Self as frame_support::traits::GenesisBuild<T>>::assimilate_storage(self, storage)
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_finalize(_now: T::BlockNumber) {}
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn stake(origin: OriginFor<T>, amount: Balance) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::stake_internal(&who, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn unstake(origin: OriginFor<T>, amount: Balance) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::unstake_internal(&who, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn return_pending_balance(
            origin: OriginFor<T>,
            nominator: T::AccountId,
            index: u64,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::return_pending_balance_internal(
                &who,
                &nominator,
                index.try_into().unwrap(),
                // index.try_into().map_err(|_| Error::<T>::IndexConvertFailed),
            )?;

            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::ModuleId::get().into_account()
    }
}
