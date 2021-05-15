# CHANGE LOG


## 2021.05.14
- Benchmark the dispatchables in loans pallet (#114)
- Add `APR` struct and refactor rate model (#115)

## 2021.05.12
- Fix oracle price benchmarking issue (#100)

## 2021.5.13
- Add multisig pallet (#112)
- Add rpc to get price from orml_oracle (#112)

## 2021.05.10
- Change price from u128 to FixedU128 (#89)
- Change LiquidationIncentive from u128 to Ratio (#89)
- Change LiquidationThreshold from u128 to Ratio (#89)
- Remove types `OraclePrice`. (#89)
- Modify types `Price` from `u128` to `FixedU128` (#89)

## 2021.05.09
- Remove unnecessary dependencies to fix benchmarking (#94)

## 2021.05.07
- Add TotalReserves storage (#92)
- Add `add_reserves` and `reduce_reserves` dispatchables.

## 2021.05.01
- Add prices pallet(#73)
- Add new types :
```
"OracleKey": "CurrencyId",
"OracleValue": "FixedU128",
"OraclePrice": "FixedU128",
"TimestampedValueOf": {
    "value": "FixedU128",
    "timestamp": "u64"
}
```

## 2021.04.29
- Add parallel-dev bin to speed up compilation (#80)
- Add Benchmarking Infrastructure and Implemented Benchmarking for mint and borrow of pallet-loans (#62)

## 2021.04.28
- Modify storage type `SupplyRate` from `u128` to `Rate`. (#82)
- Refactor rate module. (#82)

## 2021.04.25
- Remove BTC market. (#69) (#71)

## 2021.04.23
- Rename storage `CollateralRate` to `CollateralFactor`. (#64)
- Rename storage `UtilityRate` to `UtilizationRatio`. (#64)
- Add new types :

```
"PalletId": "MultiAddress",
"Rate": "FixedU128",
"Ratio": "Permill",
"Multiplier": "FixedU128",
```

- Modify types of `BorrowIndex`, `ExchangeRate`, `MultiplierPerBlock`, `JumpMultiplierPerBlock`, `BorrowRate`, `BaseRatePerBlock` from `u128` to `FixedU128` Modify their decimals from 1e9 to 1e18. (#64)
- Modify types of `CollatreralFactor`, `UtilizationRatio` from `u128` to `Permill`. (#64)
