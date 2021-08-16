use sp_runtime::DispatchResult;

//todo change the return type
pub trait LiquidStakingProtocol<AccountId, Balance> {
    fn stake(who: &AccountId, amount: Balance) -> DispatchResult;
    fn unstake(who: &AccountId, amount: Balance) -> DispatchResult;
    fn claim(who: &AccountId) -> DispatchResult;
}
