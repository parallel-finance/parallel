use crate::{
    mock::{new_test_ext, Loans, Origin, Test, ALICE, DOT, MARKET_MOCK, XDOT},
    Error, InterestRateModel, Market, MarketState, Markets,
};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use primitives::{Balance, Rate, Ratio};
use sp_runtime::FixedPointNumber;

const ACTIVE_MARKET_MOCK: Market<Balance> = {
    let mut market = MARKET_MOCK;
    market.state = MarketState::Active;
    market
};

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
        Loans::active_market(Origin::root(), XDOT).unwrap();
        assert_eq!(Loans::market(XDOT).unwrap().state, MarketState::Active);
    })
}

#[test]
fn active_market_does_not_modify_unknown_market_currencies() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Loans::active_market(Origin::root(), XDOT),
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
fn update_market_can_only_be_used_by_root() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Loans::update_market(Origin::signed(ALICE), DOT, MARKET_MOCK),
            BadOrigin
        );
    })
}

#[test]
fn update_market_does_not_modify_state() {
    new_test_ext().execute_with(|| {
        Loans::update_market(Origin::root(), DOT, MARKET_MOCK).unwrap();
        assert_eq!(Loans::market(DOT).unwrap().state, MarketState::Active);
    })
}

#[test]
fn update_market_ensures_that_it_is_not_possible_to_modify_unknown_market_currencies() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Loans::update_market(Origin::root(), XDOT, MARKET_MOCK),
            Error::<Test>::MarketDoesNotExist
        );
    })
}

#[test]
fn update_market_successfully_modifies_a_stored_market() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            Loans::market(DOT).unwrap().close_factor,
            Ratio::from_percent(50)
        );
        Loans::update_market(Origin::root(), DOT, {
            let mut market = MARKET_MOCK;
            market.close_factor = Default::default();
            market
        })
        .unwrap();
        assert_eq!(Loans::market(DOT).unwrap().close_factor, Default::default());
    })
}

#[test]
fn update_market_capacity_successfully() {
    new_test_ext().execute_with(|| {
        let dot_market = || Markets::<Test>::get(DOT).unwrap();
        assert_eq!(dot_market().cap, MARKET_MOCK.cap);

        const NEW_MARKET_CAP: u128 = 1000000000u128;

        assert_ok!(Loans::update_market(
            Origin::root(),
            DOT,
            Market::<_> {
                cap: NEW_MARKET_CAP,
                ..MARKET_MOCK
            }
        ));
        assert_eq!(dot_market().cap, NEW_MARKET_CAP);
    })
}

#[test]
fn update_market_has_sanity_checks_for_rate_models() {
    rate_model_sanity_check!(update_market);
}
