use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use primitives::TokenSymbol;

#[test]
fn add_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 20),
            (5, 5)
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 20);

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).base_amount,
            20
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::Token(TokenSymbol::DOT),
                &1.into()
            ),
            90
        );
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::Token(TokenSymbol::xDOT),
                &1.into()
            ),
            80
        );
    })
}

#[test]
fn add_more_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 20),
            (5, 5)
        ));

        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (30, 40),
            (5, 5)
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 60);

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).base_amount,
            60
        );

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).quote_amount,
            30
        );

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 60);

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 30);
    })
}

#[test]
fn add_more_liquidity_should_not_work_if_minimum_base_amount_is_higher() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 20),
            (5, 5)
        ));

        assert_noop!(
            AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (30, 40), (55, 5)),
            Error::<Test, Instance1>::NotAIdealPriceRatio
        );
    })
}

#[test]
fn add_liquidity_should_not_work_if_not_allowed_for_normal_user() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            PermissionedAMM::add_liquidity(
                Origin::signed(1.into()),
                (DOT, XDOT),
                (30, 40),
                (55, 5)
            ),
            Error::<Test, Instance2>::PoolCreationDisabled
        );
    })
}

#[test]
fn add_more_liquidity_with_low_balance_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 20),
            (5, 5)
        ));

        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (30, 40),
            (1, 1)
        ));

        assert_noop!(
            AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (50, 60), (5, 5)),
            orml_tokens::Error::<Test>::BalanceTooLow,
        );
    })
}

#[test]
fn add_liquidity_by_another_user_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 20),
            (5, 5)
        ));

        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (30, 40),
            (5, 5)
        ));

        assert_ok!(AMM::add_liquidity(
            Origin::signed(2.into()),
            (DOT, XDOT),
            (5, 10),
            (5, 5)
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 70);
    })
}

#[test]
fn add_liquidity_should_work_if_created_by_root() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::force_create_pool(
            frame_system::RawOrigin::Root.into(),
            (DOT, XDOT),
            (10, 20),
            1.into()
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 20);

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).base_amount,
            20
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::Token(TokenSymbol::DOT),
                &1.into()
            ),
            90
        );
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::Token(TokenSymbol::xDOT),
                &1.into()
            ),
            80
        );
    })
}

#[test]
fn add_liquidity_by_root_should_not_work_if_pool_already_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 20),
            (5, 5)
        ));

        assert_noop!(
            AMM::force_create_pool(
                frame_system::RawOrigin::Root.into(),
                (DOT, XDOT),
                (10, 20),
                1.into()
            ),
            Error::<Test, Instance1>::PoolAlreadyExists,
        );
    })
}

#[test]
fn remove_liquidity_whole_share_should_work() {
    new_test_ext().execute_with(|| {
        // A pool with a single LP provider
        // who deposit tokens and withdraws their whole share
        // (most simple case)

        let _ = AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (10, 90), (5, 5));

        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::total_issuance(
                AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).lp_token
            ),
            30
        );

        assert_ok!(AMM::remove_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            30
        ));

        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::total_issuance(
                AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).lp_token
            ),
            0
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::Token(TokenSymbol::DOT),
                &1.into()
            ),
            100
        );
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::Token(TokenSymbol::xDOT),
                &1.into()
            ),
            100
        );
    })
}

#[test]
fn remove_liquidity_only_portion_should_work() {
    new_test_ext().execute_with(|| {
        // A pool with a single LP provider who
        // deposit tokens and withdraws
        // a portion of their total shares (simple case)

        let _ = AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (10, 90), (5, 5));

        assert_ok!(AMM::remove_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            15
        ));

        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::total_issuance(
                AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).lp_token
            ),
            15
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::Token(TokenSymbol::DOT),
                &1.into()
            ),
            95
        );
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::Token(TokenSymbol::xDOT),
                &1.into()
            ),
            55
        );
    })
}

#[test]
fn remove_liquidity_user_more_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 25),
            (5, 5)
        ));
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (15, 30),
            (5, 5)
        ));

        assert_ok!(AMM::remove_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            15
        ));

        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::total_issuance(
                AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).lp_token
            ),
            18
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::Token(TokenSymbol::DOT),
                &1.into()
            ),
            88
        );
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::Token(TokenSymbol::xDOT),
                &1.into()
            ),
            70
        );
    })
}

#[test]
fn remove_liquidity_when_pool_does_not_exist_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AMM::remove_liquidity(Origin::signed(1.into()), (DOT, XDOT), 15),
            Error::<Test, Instance1>::PoolDoesNotExist
        );
    })
}

#[test]
fn remove_liquidity_with_more_liquidity_should_not_work() {
    new_test_ext().execute_with(|| {
        // A pool with a single LP provider
        // who deposit tokens and withdraws their whole share
        // (most simple case)

        let _ = AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (10, 90), (5, 5));

        assert_noop!(
            AMM::remove_liquidity(Origin::signed(1.into()), (DOT, XDOT), 300),
            Error::<Test, Instance1>::MoreLiquidity
        );
    })
}

#[test]
fn trade_should_work() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = AccountId::from(4_u64);

        // create pool and add liquidity
        assert_ok!(AMM::add_liquidity(
            Origin::signed(3.into()),
            (DOT, XDOT),
            (100_000_000, 100_000_000),
            (99_999, 99_999),
        ));

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100_000_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000_000); // DOT

        // calculate amount out
        let amount_out = AMM::trade(&trader, (DOT, XDOT), 1_000, 980);

        // amount out should be 994
        assert_eq!(amount_out.unwrap(), 994);

        // // pools values should be updated - we should have less XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 99_999_006);

        // // pools values should be updated - we should have more DOT in the pool
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000_998);
    })
}

#[test]
fn trade_should_not_work_if_insufficient_amount_in() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = AccountId::from(4_u64);

        // create pool and add liquidity
        assert_ok!(AMM::add_liquidity(
            Origin::signed(3.into()),
            (DOT, XDOT),
            (100_000, 100_000),
            (99_999, 99_999),
        ));

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000); // DOT

        // amount out is less than minimum_amount_out
        assert_noop!(
            AMM::trade(&trader, (DOT, XDOT), 332, 300),
            Error::<Test, Instance1>::InsufficientAmountIn
        );
    })
}

#[test]
fn trade_should_work_flipped_currencies() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = AccountId::from(4_u64);

        // create pool and add liquidity
        assert_ok!(AMM::add_liquidity(
            Origin::signed(3.into()),
            (DOT, XDOT),
            (100_000, 50_000),
            (99_999, 49_999),
        ));

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000); // DOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 50_000); // XDOT

        // calculate amount out
        let amount_out = AMM::trade(&trader, (XDOT, DOT), 500, 800);
        // fees
        // lp = 1.5 (rounded to 1)
        // protocol = 1
        // total = 2

        // amount out should be 986
        assert_eq!(amount_out.unwrap(), 986);

        // pools values should be updated - we should have less DOT in the pool
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 99_014);

        // pools values should be updated - we should have more XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 50_499);
    })
}

#[test]
fn trade_should_not_work_if_amount_less_than_miniumum() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = AccountId::from(4_u64);

        // create pool and add liquidity
        assert_ok!(AMM::add_liquidity(
            Origin::signed(3.into()),
            (DOT, XDOT),
            (100_000, 100_000),
            (99_999, 99_999),
        ));
        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100_000);
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000);

        // amount out is less than minimum_amount_out
        assert_noop!(
            AMM::trade(&trader, (DOT, XDOT), 1_000, 1_000),
            Error::<Test, Instance1>::InsufficientAmountOut
        );
    })
}

#[test]
fn trade_should_not_work_if_amount_in_is_zero() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = AccountId::from(4_u64);

        // create pool and add liquidity
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (100, 100),
            (90, 90)
        ));

        // fail if amount_in is zero
        assert_noop!(
            AMM::trade(&trader, (DOT, XDOT), 0, 0),
            Error::<Test, Instance1>::InsufficientAmountIn
        );
    })
}

#[test]
fn trade_should_not_work_if_pool_does_not_exist() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = AccountId::from(4_u64);

        // try to trade in pool with no liquidity
        assert_noop!(
            AMM::trade(&trader, (DOT, XDOT), 10, 10),
            Error::<Test, Instance1>::PoolDoesNotExist
        );
    })
}
