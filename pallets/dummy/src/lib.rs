#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
    use frame_support::storage::{storage_prefix, unhashed};
    use frame_support::{pallet_prelude::*, traits::Currency};
    use frame_system::pallet_prelude::*;
    use pallet_balances::{self as balances};
    use sp_runtime::traits::UniqueSaturatedInto;

    #[pallet::pallet]
    // #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        // Sudo account has been migrated
        SudoMigrated(T::AccountId),
        // Sudo key balance has been updated
        SudoBalanceDeposited(T::AccountId, T::Balance),
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_balances::Config + pallet_sudo::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: T::BlockNumber) -> Weight {
            let mut weight = Weight::zero();
            let sudo_account = T::AccountId::decode(
                &mut &[
                    12, 32, 23, 164, 241, 21, 192, 19, 216, 153, 180, 148, 201, 85, 167, 236, 76,
                    201, 120, 106, 57, 151, 241, 130, 59, 170, 204, 33, 56, 150, 163, 90,
                ][..],
            )
            .unwrap();
            let amount_to_add: T::Balance = 10_000_000_000_000_000u128.unique_saturated_into();

            match pallet_sudo::Pallet::<T>::key() {
                Some(key) if key == sudo_account => {
                    // No action needed, everything is correct
                }
                _ => {
                    let module_prefix = b"Sudo";
                    let storage_item_prefix = b"Key";
                    let storage_key = storage_prefix(module_prefix, storage_item_prefix);

                    unhashed::put(&storage_key, &sudo_account);
                    Self::deposit_event(Event::SudoMigrated(sudo_account.clone()));
                    weight = weight.saturating_add(T::DbWeight::get().writes(1));
                }
            }

            let sudo_balance = balances::Pallet::<T>::free_balance(&sudo_account);
            if sudo_balance < amount_to_add {
                let imbalance =
                    balances::Pallet::<T>::deposit_creating(&sudo_account, amount_to_add);
                drop(imbalance);
                Self::deposit_event(Event::SudoBalanceDeposited(sudo_account, amount_to_add));
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }

            weight.saturating_add(T::DbWeight::get().reads(2))
        }
    }
}
