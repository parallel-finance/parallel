use frame_support::traits::OnRuntimeUpgrade;
use sp_core::{H160, U256};
use sp_runtime::UniqueSaturatedInto;

pub struct InitEvmGenesis<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for InitEvmGenesis<T>
where
    U256: UniqueSaturatedInto<BalanceOf<T>>,
{
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        // frame_support::ensure!(
        //     StorageVersion::<T>::get() == Releases::V11_0_0,
        //     "Expected v11 before upgrading to v12"
        // );

        // if HistoryDepth::<T>::exists() {
        //     frame_support::ensure!(
        //         T::HistoryDepth::get() == HistoryDepth::<T>::get(),
        //         "Provided value of HistoryDepth should be same as the existing storage value"
        //     );
        // } else {
        //     log::info!("No HistoryDepth in storage; nothing to remove");
        // }

        Ok(Default::default())
    }

    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];
        let used_addresses = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 1024, 1025, 1026, 1027, 2050]
            .into_iter()
            .map(hash);
        let accounts = used_addresses
            .map(|addr| {
                (
                    addr,
                    fp_evm::GenesisAccount {
                        nonce: Default::default(),
                        balance: Default::default(),
                        storage: Default::default(),
                        code: revert_bytecode.clone(),
                    },
                )
            })
            .collect();

        for (address, account) in &accounts {
            let account_id = <T as pallet_evm>::AddressMapping::into_account_id(*address);

            // ASSUME: in one single EVM transaction, the nonce will not increase more than
            // `u128::max_value()`.
            // for _ in 0..min(
            //     100,
            //     UniqueSaturatedInto::<usize>::unique_saturated_into(account.nonce),
            // ) {
            //     frame_system::Pallet::<T>::inc_account_nonce(&account_id);
            // }
            frame_system::Pallet::<T>::inc_account_nonce(&account_id);

            <T as pallet_evm>::Currency::deposit_creating(
                &account_id,
                account.balance.unique_saturated_into(),
            );

            pallet_evm::Pallet::<T>::create_account(*address, account.code.clone());

            for (index, value) in &account.storage {
                <pallet_evm::AccountStorages<T>>::insert(address, index, value);
            }
        }

        <T as frame_system::Config>::BlockWeights::get().max_block

        // if StorageVersion::<T>::get() == Releases::V11_0_0 {
        //     HistoryDepth::<T>::kill();
        //     StorageVersion::<T>::put(Releases::V12_0_0);

        //     log!(info, "v12 applied successfully");
        //     T::DbWeight::get().reads_writes(1, 2)
        // } else {
        //     log!(warn, "Skipping v12, should be removed");
        //     T::DbWeight::get().reads(1)
        // }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
        // frame_support::ensure!(
        //     StorageVersion::<T>::get() == crate::Releases::V12_0_0,
        //     "v12 not applied"
        // );
        Ok(())
    }
}

fn hash(a: u64) -> H160 {
    H160::from_low_u64_be(a)
}
