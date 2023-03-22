use frame_support::{log, traits::OnRuntimeUpgrade};
use pallet_evm::Config;
use sp_core::{Get, H160};
use sp_std::vec::Vec;

fn revert_bytecode() -> Vec<u8> {
    sp_std::vec![0x60, 0x00, 0x60, 0x00, 0xFD]
}

fn used_addresses() -> impl Iterator<Item = H160> {
    sp_std::vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 1024, 1025, 1026, 1027, 2050]
        .into_iter()
        .map(hash)
}
fn hash(a: u64) -> H160 {
    H160::from_low_u64_be(a)
}

pub struct InitEvmGenesis<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for InitEvmGenesis<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        for addr in used_addresses() {
            frame_support::ensure!(
                !<pallet_evm::AccountCodes<T>>::contains_key(addr),
                "Expected empty account code"
            );
        }
        log::info!(target: "runtime::pallet_evm", "pre_upgrade: ready for migrate");

        Ok(Default::default())
    }

    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        if <pallet_evm::AccountCodes<T>>::contains_key(hash(1)) {
            log::warn!(target: "runtime::pallet_evm", "already init evm genesis code");
            return T::DbWeight::get().reads(1);
        }
        for addr in used_addresses() {
            pallet_evm::Pallet::<T>::create_account(addr, revert_bytecode());
        }
        log::info!(target: "runtime::pallet_evm", "init evm genesis code successfully");
        T::DbWeight::get().reads_writes(15, 15 * 2)
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
        for addr in used_addresses() {
            frame_support::ensure!(
                <pallet_evm::AccountCodes<T>>::get(addr) == revert_bytecode(),
                "Expected equal account code"
            );
        }
        log::info!(target: "runtime::pallet_evm", "post_upgrade: migrate successfully");
        Ok(())
    }
}
