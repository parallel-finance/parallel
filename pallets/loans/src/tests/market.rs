use crate::{
    mock::{
        market_mock, new_test_ext, Loans, Origin, Test, ACTIVE_MARKET_MOCK, ALICE, DOT,
        MARKET_MOCK, PDOT, PUSDT, SDOT,
    },
    Error, InterestRateModel, MarketState,
};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use primitives::{Rate, Ratio};
use sp_runtime::{traits::Zero, FixedPointNumber};

macro_rules! rate_model_sanity_check {
    ($call:ident) => {
        new_test_ext().execute_with(|| {
            // Invalid base_rate
            assert_noop!(
                Loans::$call(Origin::root(), SDOT, {
                    let mut market = MARKET_MOCK;
                    market.rate_model = InterestRateModel::new_jump_model(
                        Rate::saturating_from_rational(36, 100),
                        Rate::saturating_from_rational(15, 100),
                        Rate::saturating_from_rational(35, 100),
                        Ratio::from_percent(80),
                    );
                    market
                }),
                Error::<Test>::InvalidRateModelParam
            );
            // Invalid jump_rate
            assert_noop!(
                Loans::$call(Origin::root(), SDOT, {
                    let mut market = MARKET_MOCK;
                    market.rate_model = InterestRateModel::new_jump_model(
                        Rate::saturating_from_rational(5, 100),
                        Rate::saturating_from_rational(36, 100),
                        Rate::saturating_from_rational(37, 100),
                        Ratio::from_percent(80),
                    );
                    market
                }),
                Error::<Test>::InvalidRateModelParam
            );
            // Invalid full_rate
            assert_noop!(
                Loans::$call(Origin::root(), SDOT, {
                    let mut market = MARKET_MOCK;
                    market.rate_model = InterestRateModel::new_jump_model(
                        Rate::saturating_from_rational(5, 100),
                        Rate::saturating_from_rational(15, 100),
                        Rate::saturating_from_rational(57, 100),
                        Ratio::from_percent(80),
                    );
                    market
                }),
                Error::<Test>::InvalidRateModelParam
            );
            // base_rate greater than jump_rate
            assert_noop!(
                Loans::$call(Origin::root(), SDOT, {
                    let mut market = MARKET_MOCK;
                    market.rate_model = InterestRateModel::new_jump_model(
                        Rate::saturating_from_rational(10, 100),
                        Rate::saturating_from_rational(9, 100),
                        Rate::saturating_from_rational(14, 100),
                        Ratio::from_percent(80),
                    );
                    market
                }),
                Error::<Test>::InvalidRateModelParam
            );
            // jump_rate greater than full_rate
            assert_noop!(
                Loans::$call(Origin::root(), SDOT, {
                    let mut market = MARKET_MOCK;
                    market.rate_model = InterestRateModel::new_jump_model(
                        Rate::saturating_from_rational(5, 100),
                        Rate::saturating_from_rational(15, 100),
                        Rate::saturating_from_rational(14, 100),
                        Ratio::from_percent(80),
                    );
                    market
                }),
                Error::<Test>::InvalidRateModelParam
            );
        })
    };
}

#[test]
fn active_market_sets_state_to_active() {
    new_test_ext().execute_with(|| {
        Loans::add_market(Origin::root(), SDOT, MARKET_MOCK).unwrap();
        assert_eq!(Loans::market(SDOT).unwrap().state, MarketState::Pending);
        Loans::activate_market(Origin::root(), SDOT).unwrap();
        assert_eq!(Loans::market(SDOT).unwrap().state, MarketState::Active);
    })
}

#[test]
fn active_market_does_not_modify_unknown_market_currencies() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Loans::activate_market(Origin::root(), SDOT),
            Error::<Test>::MarketDoesNotExist
        );
    })
}

#[test]
fn add_market_can_only_be_used_by_root() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Loans::add_market(Origin::signed(ALICE), DOT, MARKET_MOCK),
            BadOrigin
        );
    })
}

#[test]
fn add_market_ensures_that_market_state_must_be_pending() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Loans::add_market(Origin::root(), SDOT, ACTIVE_MARKET_MOCK),
            Error::<Test>::NewMarketMustHavePendingState
        );
    })
}

#[test]
fn add_market_has_sanity_checks_for_rate_models() {
    rate_model_sanity_check!(add_market);
}

#[test]
fn add_market_successfully_stores_a_new_market() {
    new_test_ext().execute_with(|| {
        Loans::add_market(Origin::root(), SDOT, MARKET_MOCK).unwrap();
        assert_eq!(Loans::market(SDOT).unwrap(), MARKET_MOCK);
    })
}

#[test]
fn add_market_ensures_that_market_does_not_exist() {
    new_test_ext().execute_with(|| {
        assert_ok!(Loans::add_market(Origin::root(), SDOT, MARKET_MOCK));
        assert_noop!(
            Loans::add_market(Origin::root(), SDOT, MARKET_MOCK),
            Error::<Test>::MarketAlreadyExists
        );
    })
}

#[test]
fn force_update_market_can_only_be_used_by_root() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Loans::force_update_market(Origin::signed(ALICE), DOT, MARKET_MOCK),
            BadOrigin
        );
    })
}

#[test]
fn force_update_market_works() {
    new_test_ext().execute_with(|| {
        let mut new_market = market_mock(PDOT);
        new_market.state = MarketState::Active;
        Loans::force_update_market(Origin::root(), DOT, new_market).unwrap();
        assert_eq!(Loans::market(DOT).unwrap().state, MarketState::Active);
        assert_eq!(Loans::market(DOT).unwrap().ptoken_id, PDOT);

        // New ptoken_id must not be in use
        assert_noop!(
            Loans::force_update_market(Origin::root(), DOT, market_mock(PUSDT)),
            Error::<Test>::InvalidPtokenId
        );
        assert_ok!(Loans::force_update_market(
            Origin::root(),
            DOT,
            market_mock(1234)
        ));
        assert_eq!(Loans::market(DOT).unwrap().ptoken_id, 1234);
    })
}

#[test]
fn force_update_market_ensures_that_it_is_not_possible_to_modify_unknown_market_currencies() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Loans::force_update_market(Origin::root(), SDOT, MARKET_MOCK),
            Error::<Test>::MarketDoesNotExist
        );
    })
}

#[test]
fn update_market_has_sanity_checks_for_rate_models() {
    rate_model_sanity_check!(force_update_market);
}

#[test]
fn update_market_ensures_that_it_is_not_possible_to_modify_unknown_market_currencies() {
    new_test_ext().execute_with(|| {
        let market = MARKET_MOCK;
        assert_noop!(
            Loans::update_market(
                Origin::root(),
                SDOT,
                market.collateral_factor,
                market.liquidation_threshold,
                market.reserve_factor,
                market.close_factor,
                market.liquidate_incentive_reserved_factor,
                market.liquidate_incentive,
                market.supply_cap,
                market.borrow_cap,
            ),
            Error::<Test>::MarketDoesNotExist
        );
    })
}

#[test]
fn update_market_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            Loans::market(DOT).unwrap().close_factor,
            Ratio::from_percent(50)
        );

        let market = MARKET_MOCK;
        assert_ok!(Loans::update_market(
            Origin::root(),
            DOT,
            market.collateral_factor,
            market.liquidation_threshold,
            market.reserve_factor,
            Default::default(),
            market.liquidate_incentive_reserved_factor,
            market.liquidate_incentive,
            market.supply_cap,
            market.borrow_cap,
        ));

        assert_eq!(Loans::market(DOT).unwrap().close_factor, Default::default());
        assert_eq!(Loans::market(DOT).unwrap().supply_cap, market.supply_cap);
    })
}

#[test]
fn update_market_should_not_work_if_with_invalid_params() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            Loans::market(DOT).unwrap().close_factor,
            Ratio::from_percent(50)
        );

        let market = MARKET_MOCK;
        // check error code while collateral_factor is [0%, 100%)
        assert_ok!(Loans::update_market(
            Origin::root(),
            DOT,
            Ratio::zero(),
            market.liquidation_threshold,
            market.reserve_factor,
            Default::default(),
            market.liquidate_incentive_reserved_factor,
            market.liquidate_incentive,
            market.supply_cap,
            market.borrow_cap,
        ));
        assert_noop!(
            Loans::update_market(
                Origin::root(),
                DOT,
                Ratio::one(),
                market.liquidation_threshold,
                market.reserve_factor,
                Default::default(),
                market.liquidate_incentive_reserved_factor,
                market.liquidate_incentive,
                market.supply_cap,
                market.borrow_cap,
            ),
            Error::<Test>::InvalidFactor
        );
        // check error code while reserve_factor is 0% or bigger than 100%
        assert_noop!(
            Loans::update_market(
                Origin::root(),
                DOT,
                market.collateral_factor,
                market.liquidation_threshold,
                Ratio::zero(),
                Default::default(),
                market.liquidate_incentive_reserved_factor,
                market.liquidate_incentive,
                market.supply_cap,
                market.borrow_cap,
            ),
            Error::<Test>::InvalidFactor
        );
        assert_noop!(
            Loans::update_market(
                Origin::root(),
                DOT,
                market.collateral_factor,
                market.liquidation_threshold,
                Ratio::one(),
                Default::default(),
                market.liquidate_incentive_reserved_factor,
                market.liquidate_incentive,
                market.supply_cap,
                market.borrow_cap,
            ),
            Error::<Test>::InvalidFactor
        );
        // check error code while cap is zero
        assert_noop!(
            Loans::update_market(
                Origin::root(),
                DOT,
                market.collateral_factor,
                market.liquidation_threshold,
                market.reserve_factor,
                Default::default(),
                market.liquidate_incentive_reserved_factor,
                Rate::from_inner(Rate::DIV / 100 * 90),
                Zero::zero(),
                market.borrow_cap,
            ),
            Error::<Test>::InvalidSupplyCap
        );
    })
}

#[test]
fn update_rate_model_works() {
    new_test_ext().execute_with(|| {
        let new_rate_model = InterestRateModel::new_jump_model(
            Rate::saturating_from_rational(6, 100),
            Rate::saturating_from_rational(15, 100),
            Rate::saturating_from_rational(35, 100),
            Ratio::from_percent(80),
        );
        assert_ok!(Loans::update_rate_model(
            Origin::root(),
            DOT,
            new_rate_model,
        ));
        assert_eq!(Loans::market(DOT).unwrap().rate_model, new_rate_model);

        // Invalid base_rate
        assert_noop!(
            Loans::update_rate_model(
                Origin::root(),
                SDOT,
                InterestRateModel::new_jump_model(
                    Rate::saturating_from_rational(36, 100),
                    Rate::saturating_from_rational(15, 100),
                    Rate::saturating_from_rational(35, 100),
                    Ratio::from_percent(80),
                )
            ),
            Error::<Test>::InvalidRateModelParam
        );
        // Invalid jump_rate
        assert_noop!(
            Loans::update_rate_model(
                Origin::root(),
                SDOT,
                InterestRateModel::new_jump_model(
                    Rate::saturating_from_rational(5, 100),
                    Rate::saturating_from_rational(36, 100),
                    Rate::saturating_from_rational(37, 100),
                    Ratio::from_percent(80),
                )
            ),
            Error::<Test>::InvalidRateModelParam
        );
        // Invalid full_rate
        assert_noop!(
            Loans::update_rate_model(
                Origin::root(),
                SDOT,
                InterestRateModel::new_jump_model(
                    Rate::saturating_from_rational(5, 100),
                    Rate::saturating_from_rational(15, 100),
                    Rate::saturating_from_rational(57, 100),
                    Ratio::from_percent(80),
                )
            ),
            Error::<Test>::InvalidRateModelParam
        );
        // base_rate greater than jump_rate
        assert_noop!(
            Loans::update_rate_model(
                Origin::root(),
                SDOT,
                InterestRateModel::new_jump_model(
                    Rate::saturating_from_rational(10, 100),
                    Rate::saturating_from_rational(9, 100),
                    Rate::saturating_from_rational(14, 100),
                    Ratio::from_percent(80),
                )
            ),
            Error::<Test>::InvalidRateModelParam
        );
        // jump_rate greater than full_rate
        assert_noop!(
            Loans::update_rate_model(
                Origin::root(),
                SDOT,
                InterestRateModel::new_jump_model(
                    Rate::saturating_from_rational(5, 100),
                    Rate::saturating_from_rational(15, 100),
                    Rate::saturating_from_rational(14, 100),
                    Ratio::from_percent(80),
                )
            ),
            Error::<Test>::InvalidRateModelParam
        );
    })
}
