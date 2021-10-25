use crate::{
    mock::{new_test_ext, Loans, Origin, Test, ACTIVE_MARKET_MOCK, ALICE, DOT, MARKET_MOCK, XDOT},
    Error, InterestRateModel, MarketState,
};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use primitives::{Rate, Ratio};
use sp_runtime::FixedPointNumber;

macro_rules! rate_model_sanity_check {
    ($call:ident) => {
        new_test_ext().execute_with(|| {
            // Invalid base_rate
            assert_noop!(
                Loans::$call(Origin::root(), XDOT, {
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
                Loans::$call(Origin::root(), XDOT, {
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
                Loans::$call(Origin::root(), XDOT, {
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
                Loans::$call(Origin::root(), XDOT, {
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
                Loans::$call(Origin::root(), XDOT, {
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
        Loans::add_market(Origin::root(), XDOT, MARKET_MOCK).unwrap();
        assert_eq!(Loans::market(XDOT).unwrap().state, MarketState::Pending);
        Loans::activate_market(Origin::root(), XDOT).unwrap();
        assert_eq!(Loans::market(XDOT).unwrap().state, MarketState::Active);
    })
}

#[test]
fn active_market_does_not_modify_unknown_market_currencies() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Loans::activate_market(Origin::root(), XDOT),
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
            Loans::add_market(Origin::root(), XDOT, ACTIVE_MARKET_MOCK),
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
        Loans::add_market(Origin::root(), XDOT, MARKET_MOCK).unwrap();
        assert_eq!(Loans::market(XDOT).unwrap(), MARKET_MOCK);
    })
}

#[test]
fn add_market_ensures_that_market_does_not_exist() {
    new_test_ext().execute_with(|| {
        assert_ok!(Loans::add_market(Origin::root(), XDOT, MARKET_MOCK));
        assert_noop!(
            Loans::add_market(Origin::root(), XDOT, MARKET_MOCK),
            Error::<Test>::MarketAlredyExists
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
        Loans::force_update_market(Origin::root(), DOT, MARKET_MOCK).unwrap();
        assert_eq!(Loans::market(DOT).unwrap().state, MarketState::Pending);
    })
}

#[test]
fn force_update_market_ensures_that_it_is_not_possible_to_modify_unknown_market_currencies() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Loans::force_update_market(Origin::root(), XDOT, MARKET_MOCK),
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
                XDOT,
                market.collateral_factor,
                market.reserve_factor,
                market.close_factor,
                market.liquidate_incentive,
                market.cap,
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
            market.reserve_factor,
            Default::default(),
            market.liquidate_incentive,
            0
        ));

        assert_eq!(Loans::market(DOT).unwrap().close_factor, Default::default());
        assert_eq!(Loans::market(DOT).unwrap().cap, 0);
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
                XDOT,
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
                XDOT,
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
                XDOT,
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
                XDOT,
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
                XDOT,
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
