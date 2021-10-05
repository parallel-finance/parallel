use super::*;
use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn create_new_vault_should_work() {
    new_test_ext().execute_with(|| {
        let crowdloan = 1337;
        let project_shares = 420;
        let currency_shares = 113;
        let token = CurrencyId::Asset(1);

        let contribution_strategy =
            ContributionStrategy::Placeholder(crowdloan, Asset(currency_shares), crowdloan as u128);

        let claim_strategy = ClaimStrategy::Placeholder(crowdloan);

        assert_ok!(Crowdloan::create_vault(
            frame_system::RawOrigin::Root.into(), // origin
            token,                                // token
            crowdloan,                            // crowdloan
            Asset(project_shares),                // project_shares
            Asset(currency_shares),               // currency_shares
            contribution_strategy,                // contribution_strategy
            claim_strategy,                       // claim_strategy
        ));

        if let Some(just_created_vault) = Crowdloan::vaults(crowdloan) {
            assert_eq!(
                just_created_vault,
                Vault {
                    project_shares: CurrencyId::Asset(project_shares),
                    currency_shares: CurrencyId::Asset(currency_shares),
                    currency: CurrencyId::Asset(currency_shares),
                    phase: VaultPhase::CollectingContributions,
                    contribution_strategy: ContributionStrategy::Placeholder(
                        crowdloan,
                        CurrencyId::Asset(currency_shares),
                        0,
                    ),
                    claim_strategy: ClaimStrategy::Placeholder(crowdloan),
                    contributed: 0,
                }
            );
        }
    });
}
