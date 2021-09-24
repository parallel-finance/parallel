use super::*;
use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn create_new_vault_should_work() {
    new_test_ext().execute_with(|| {
        let vault_id = 0;

        assert_ok!(Crowdloan::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            CurrencyId::Asset(1),                 // token
            vault_id,                             // crowdloan
            0,                                    // project_shares
            1,                                    // currency_shares
            9_999_999,                            // until
        ));

        if let Some(just_created_vault) = Crowdloan::vaults(vault_id) {
            assert_eq!(
                just_created_vault,
                Vault {
                    project_shares: 0,
                    currency_shares: 1,
                    currency: CurrencyId::Asset(1),
                    phase: VaultPhase::CollectingContributionsUntil(9_999_999),
                    claimed: 0
                }
            );
        }
    });
}
