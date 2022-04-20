initSidebarItems({"enum":[["Call","Contains one variant per dispatchable that can be called by an extrinsic."],["Error","Custom dispatch errors of this pallet."],["Event","The event emitted by this pallet."]],"struct":[["Pallet","The pallet implementing the on-chain logic."],["_GeneratedPrefixForStorageAccountBorrows",""],["_GeneratedPrefixForStorageAccountDeposits",""],["_GeneratedPrefixForStorageAccountEarned",""],["_GeneratedPrefixForStorageBorrowIndex",""],["_GeneratedPrefixForStorageBorrowRate",""],["_GeneratedPrefixForStorageExchangeRate",""],["_GeneratedPrefixForStorageLastAccruedInterestTime",""],["_GeneratedPrefixForStorageMarketRewardSpeed",""],["_GeneratedPrefixForStorageMarkets",""],["_GeneratedPrefixForStorageRewardAccured",""],["_GeneratedPrefixForStorageRewardBorrowState",""],["_GeneratedPrefixForStorageRewardBorrowerIndex",""],["_GeneratedPrefixForStorageRewardSupplierIndex",""],["_GeneratedPrefixForStorageRewardSupplyState",""],["_GeneratedPrefixForStorageSupplyRate",""],["_GeneratedPrefixForStorageTotalBorrows",""],["_GeneratedPrefixForStorageTotalReserves",""],["_GeneratedPrefixForStorageTotalSupply",""],["_GeneratedPrefixForStorageUnderlyingAssetId",""],["_GeneratedPrefixForStorageUtilizationRatio",""]],"trait":[["Config","Configuration trait of this pallet."]],"type":[["AccountBorrows","Mapping of account addresses to outstanding borrow balances CurrencyId -> Owner -> BorrowSnapshot"],["AccountDeposits","Mapping of account addresses to deposit details CollateralType -> Owner -> Deposits"],["AccountEarned","Mapping of account addresses to total deposit interest accrual CurrencyId -> Owner -> EarnedSnapshot"],["BorrowIndex","Accumulator of the total earned interest rate since the opening of the market CurrencyId -> u128"],["BorrowRate","Mapping of borrow rate to currency type"],["ExchangeRate","The exchange rate from the underlying to the internal collateral"],["LastAccruedInterestTime","The timestamp of the last calculation of accrued interest"],["MarketRewardSpeed","Mapping of token id to reward speed"],["Markets","Mapping of asset id to its market"],["Module","Type alias to `Pallet`, to be used by `construct_runtime`."],["RewardAccured","The reward accrued but not yet transferred to each user."],["RewardBorrowState","The Reward market borrow state for each market"],["RewardBorrowerIndex","The Reward index for each market for each borrower as of the last time they accrued Reward"],["RewardSupplierIndex","The Reward index for each market for each supplier as of the last time they accrued Reward"],["RewardSupplyState","The Reward market supply state for each market"],["SupplyRate","Mapping of supply rate to currency type"],["TotalBorrows","Total amount of outstanding borrows of the underlying in this market CurrencyId -> Balance"],["TotalReserves","Total amount of reserves of the underlying held in this market CurrencyId -> Balance"],["TotalSupply","Total number of collateral tokens in circulation CollateralType -> Balance"],["UnderlyingAssetId","Mapping of ptoken id to asset id `ptoken id`: voucher token id `asset id`: underlying token id"],["UtilizationRatio","Borrow utilization ratio"]]});