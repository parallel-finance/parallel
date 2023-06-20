# CHANGE LOG

## 2022.1.20

- introduce MinimumLiquidity into AMM pallet (#1179)

## 2022.1.12

- add initial AMM pools into launch scripts (#1171)

## 2022.1.8

- split crowdloans' claim api into claim, withdraw and redeem (#1148)

## 2022.1.6

- switch from cDOT-project to cDOT-lease (#1138)

## 2021.12.31

- introduce ChildStorageKind (#1104)
- add flying childstorage to crowdloans pallet (#1107)

## 2021.12.23

- bump polkadot to v0.9.13 (#1079)

## 2021.12.20

- add Succeeded vault phase (#1060)

## 2021.12.16

- add cap_limit, end_block and update_vault call (#1039)

## 2021.12.8

- refactor crowdloans pallet using xcm-helper (#954)
- support early contribution (#980)

## 2021.12.7

- refactor liquid staking pallet using xcm-helper (#1038)

## 2021.12.1

- add storage item to disable contributions in vrf period (#966)
- support refunding extra xcm fees to another account (#967)

## 2021.11.29

- finalize emergency-shutdown pallet (#913)
- add MinContribution config (#941)
- cleanup crowdloans pallet (#940)

## 2021.11.26

- add reopen call for crowdloans pallet (#933)

## 2021.11.23

- disable state-cache (#910)

## 2021.11.22

- use parachain system as HKO vesting block provider (#908)

## 2021.11.24

- finalize bridge pallet (#871)

## 2021.11.20

- support HKO crosschain transfer to/from karura (#904)

## 2021.11.19

- add unit tests for crowdloans pallet (#893)

## 2021.11.18

- add LAUNCH.md doc (#893)

## 2021.11.12

- add crowdloans to launch script (#880, #882)

## 2021.11.11

- add xcm to crowdloans pallet (#869)

## 2021.11.5

- fix default para id (#861, #860, #866)

## 2021.11.4

- add wasm execution flags to parachain-launch (#859)
- update launch config to use polkadot-v0.9.12 (#855)

## 2021.11.3

- add polkadot support for collator/fullnode script (#854)

## 2021.11.1

- fix the inconsistent liquidity calculation (#839)

## 2021.10.30

- Bump to polkadot-v0.9.12 (#819)

## 2021.10.28

- convert spec to code and setup testing framework (#640)

## 2021.10.27

- fix the error native currendy id (#817)
- restruct token registry (#816)

## 2021.10.26

- Intro liquidation client && Fix launch config (#815)
- Improve the markets governance (#802)
- use static weights for nominee-election & prices (#813)
- cleanup nominee-election & prices pallet (#812)

## 2021.10.25

- adapt create-volume script for polkadot-v0.9.11 (#811)
- make sure enough cash when user borrow (#809)
- enable proxy for parallel, vanilla (#810)
- set safe xcm version to 2 (#808)
- enable governance (#807)
- enable parachain xcm trace (#806)
- add proxy pallet & enable some call filters (#804)

## 2021.10.24

- add launch before sending batch all tx (#800)
- allow known query responses to pass barrier check (#797)
- AMM pool creation via governance only (#700)

## 2021.10.23

- cleanup liquid staking's event & error naming (#793)
- add payout-slashed's weights (#792)
- fix refund's dest location & bump cargo.toml (#789)

## 2021.10.22

- bump parallel-js to v1.4.2 (#786)
- Slash from insurance (#784)
- use kusama_runtime for testing & add host configuration (#785)

## 2021.10.21

- Disable direct XCM execution (#772)
- Impl ptoken in money market (#694)
- export polkadot-xcm storage & config (#767)
- update launch & parachain-launch version (#766)
- allow version subscription between relaychain & parachain (#764)
- use pallet_xcm to wrap version & handle response (#763)

## 2021.10.20

- set max_assets to 1 and disable xcm execute from other chains (#760)

## 2021.10.19

- bump polkadot to v0.9.11 (#693)
- add vanilla-live chainspec (#753)
- add insurance call (#751)

## 2021.10.18

- use UpdateOrigin for non-transact call (#748)
- add update_reserve_factor call (#739)
- burn fees in ump_transact instead (#733)
- Governance xcm weight in Liquidstaking (#735)

## 2021.10.17

- Support building staging, QA docker (#732)

## 2021.10.16

- Add staking market cap (#720)
- Enrich decimal provider by adding USDT (#718)

## 2021.10.15

- Bump to polkadot-v0.9.11 (#693)
- Rename ensure_currency to ensure_market (#704)
- Support HKO/PARA's crosschain transfer (#703)
- Cleanup pallet-liquid-staking's unused code (#702, #701)

## 2021.10.14

- Merge ump transfer & bond (#699)

## 2021.10.13

- Add market cap in launch script (#692)
- Increase the accrue interest interval (#689)

## 2021.10.12

- Integrate pallet-currency-adapter into loans pallet (#685)

## 2021.09.26

- Bump to polkadot-v0.9.10 (#637)
- Migrate to xcm v1 (#637)

## 2021.09.18

- Use ubuntu20.04 as base docker image (#605)
- Remove totally orml-currencies, orml-tokens (#607)

## 2021.09.15

- Use pallet-assets in liquid-staking pallet (#589)
- Add MultiCurrencyAdapter for pallet-assets (#589)
- Fix unit tests of liquid-staking (#595)

## 2021.09.14

- Use pallet-assets in loans pallet (#577)
- Remove pallet-loans-benchmarking (#577)

## 2021.09.07

- Replace pallet-liquid-staking by v2 for future xcm.Transact impls (#539)

## 2021.09.07

- Fix loans.add_market call (#538)

## 2021.09.02

- Implement permisionless and permissioned pool creation (#516)

## 2021.09.01

- DB Access Optimization in AMM pallet (#505)

## 2021.08.31

- Remove parallel-dev (#501)
- Adjust vanilla-runtime to be used in parachain (#501)

## 2021.08.30

- Add Liquidity Ratio support in AMM pallet (#494)

## 2021.08.28

- Integrate AMM pallet into runtime (#482)

## 2021.08.27

- Add benchmarking of AMM pallet (#481)

## 2021.08.24

- Modify AMM pallet pools in storage to allow for a more efficient implementation (#450)
- Support liquidity providing (#450)

## 2021.07.24

- Integrate parachain-launch (#325)
- Split docker images (#325)
- Add subwasm (#325)
- Announce new release to discord channel (#325)

## 2021.07.22

- Add ORML Vesting (#316)

## 2021.07.20

- Add nominee-election pallet (#306)

## 2021.07.19

- Remove type Multiplier
- Fix the wrong return types of JSON-RPC loans_getAccountLiquidity

## 2021.07.15

- Upgrade polkadot to v0.9.8 (#301)

## 2021.07.08

- Upgrade polkadot to v0.9.7
- Add private validators
- Improve parachain workflow

## 2021.06.30

- [`loans`] `Currencies`, `CurrencyInterestModel`, `CollateralFactor`, `ReserveFactor`, `LiquidationIncentive` and `CloseFactor` storages were removed in favor of the `Market` structure.

## 2021.06.22

- Upgrade to polkadot-v0.9.5 (#237)

## 2021.06.19

- Remove APR struct (#226)
- Modify the type of InterestRateModel from struct to enum
- Support the curve rate model

## 2021.06.14

- Upgrade to polkadot-v0.9.4 (#206)

## 2021.06.09

- Add Deposits struct(#206)
- Change the value type of AccountCollateral from Balance to Deposits
- Rename AccountCollateral to AccountDeposits
- Remove the storage AccountCollateralAsset

## 2021.06.07

- Use KSM/DOT as native currency (#197)

## 2021.06.06

- Use timestamp to accrue interest (#186)

## 2021.06.01

- Remove LiquidationThreshold

## 2021.05.28

- Add xKSM & heiko runtime (#152)

## 2021.05.27

- Upgrade to polkadot-v0.9.2 (#147)

## 2021.05.25

- Change hasher to Blake2_128Concat for T::AccountId (#142)
- Use BoundedVec for AccountProcessUnstake (#143)

## 2021.05.23

- Add RpcDataProviderId & AccountData to types.json (#136)

## 2021.05.21

- Add types.json update bot (#130)

## 2021.05.19

- Add governance support (#126)

## 2021.05.18

- Add polkadot-launch (#119)
- Add dockerfile & Docker image build (#125)

## 2021.05.14

- Benchmark the dispatchables in loans pallet (#114)
- Add `APR` struct and refactor rate model (#115)
- upgrade to polkadot-v0.9.1 (#113)

## 2021.05.12

- Fix oracle price benchmarking issue (#100)

## 2021.5.13

- Add multisig pallet (#112)
- Add rpc to get price from orml_oracle (#112)
- Bump deps (#113)

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
  ,
