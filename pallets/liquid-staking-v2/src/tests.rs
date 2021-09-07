use crate::{
    mock::*,
    types::{MatchingLedger, StakingSettlementKind},
    *,
};
use frame_support::{assert_err, assert_ok, traits::Hooks};
use orml_traits::MultiCurrency;
use primitives::{Balance, CurrencyId, Rate, TokenSymbol};
use sp_runtime::traits::One;

#[test]
fn stake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), 10));
        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::one());
        assert_eq!(StakingPool::<Test>::get(), 10);
        assert_eq!(
            MatchingPool::<Test>::get(),
            MatchingLedger {
                total_stake_amount: 10,
                total_unstake_amount: 0,
            }
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::Token(TokenSymbol::DOT), &ALICE),
            90
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::Token(TokenSymbol::xDOT), &ALICE),
            110
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(
                CurrencyId::Token(TokenSymbol::DOT),
                &LiquidStaking::account_id()
            ),
            10
        );
    })
}

#[test]
fn unstake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), 10));
        assert_ok!(LiquidStaking::unstake(Origin::signed(ALICE), 6));

        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::one());
        assert_eq!(StakingPool::<Test>::get(), 4);
        assert_eq!(
            MatchingPool::<Test>::get(),
            MatchingLedger {
                total_stake_amount: 10,
                total_unstake_amount: 6,
            }
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::Token(TokenSymbol::DOT), &ALICE),
            96
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::Token(TokenSymbol::xDOT), &ALICE),
            104
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(
                CurrencyId::Token(TokenSymbol::DOT),
                &LiquidStaking::account_id()
            ),
            4
        );
    })
}

#[test]
fn test_record_staking_settlement_ok() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::record_staking_settlement(
            Origin::signed(ALICE),
            1,
            100,
            StakingSettlementKind::Reward
        ));

        assert_eq!(LiquidStaking::exchange_rate(), Rate::from(1));
    })
}

#[test]
fn test_duplicated_record_staking_settlement() {
    new_test_ext().execute_with(|| {
        LiquidStaking::record_staking_settlement(
            Origin::signed(ALICE),
            1,
            100,
            StakingSettlementKind::Reward,
        )
        .unwrap();

        assert_err!(
            LiquidStaking::record_staking_settlement(
                Origin::signed(ALICE),
                1,
                100,
                StakingSettlementKind::Reward
            ),
            Error::<Test>::StakingSettlementAlreadyRecorded
        )
    })
}

enum StakeOp {
    Stake(Balance),
    Unstake(Balance),
}

impl StakeOp {
    fn execute(self) {
        match self {
            Self::Stake(amount) => LiquidStaking::stake(Origin::signed(ALICE), amount).unwrap(),
            Self::Unstake(amount) => LiquidStaking::unstake(Origin::signed(ALICE), amount).unwrap(),
        };
    }
}

#[test]
fn test_settlement_should_work() {
    use StakeOp::*;
    new_test_ext().execute_with(|| {
        let test_case: Vec<(Vec<StakeOp>, Balance, (Balance, Balance, Balance), Balance)> = vec![
            (vec![Stake(30), Unstake(5)], 0, (25, 0, 0), 0),
            // Calculate right here.
            (vec![Unstake(10), Unstake(5), Stake(10)], 0, (0, 0, 5), 10),
            (vec![], 0, (0, 0, 0), 0),
        ];

        for (stake_ops, unbonding_amount, matching_result, _pallet_balance) in test_case.into_iter()
        {
            stake_ops.into_iter().for_each(StakeOp::execute);
            assert_eq!(
                LiquidStaking::matching_pool().matching(unbonding_amount),
                matching_result
            );
            assert_ok!(LiquidStaking::settlement(
                Origin::signed(ALICE),
                unbonding_amount
            ));
            Pallet::<Test>::on_idle(0, 10000);
        }
    })
}
