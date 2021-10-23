const config = {
  liquidAsset: 5000,
  stakingAsset: 100,
  assets: [
    {
      name: 'Kusama',
      symbol: 'KSM',
      assetId: 100,
      decimal: 12,
      marketOption: {
        closeFactor: 50e4,
        collateralFactor: 50e4,
        reserveFactor: 15e4,
        cap: '100000000000000000',
        liquidateIncentive: '1100000000000000000',
        rateModel: {
          jumpModel: {
            baseRate: '20000000000000000',
            jumpRate: '100000000000000000',
            fullRate: '320000000000000000',
            jumpUtilization: 8e5
          }
        },
        state: 'Pending',
        ptokenId: 1000
      },
      balances: []
    },
    {
      name: 'Parallel Kusama',
      symbol: 'XKSM',
      assetId: 5000,
      decimal: 12,
      marketOption: {
        closeFactor: 50e4,
        collateralFactor: 50e4,
        reserveFactor: 15e4,
        cap: '100000000000000000',
        liquidateIncentive: '1100000000000000000',
        rateModel: {
          jumpModel: {
            baseRate: '20000000000000000',
            jumpRate: '100000000000000000',
            fullRate: '320000000000000000',
            jumpUtilization: 8e5
          }
        },
        state: 'Pending',
        ptokenId: 2000
      },
      balances: []
    },
    {
      name: 'Tether Dollar',
      symbol: 'USDT',
      assetId: 102,
      decimal: 6,
      marketOption: {
        closeFactor: 50e4,
        collateralFactor: 50e4,
        reserveFactor: 15e4,
        cap: '100000000000000000',
        liquidateIncentive: '1100000000000000000',
        rateModel: {
          jumpModel: {
            baseRate: '20000000000000000',
            jumpRate: '100000000000000000',
            fullRate: '320000000000000000',
            jumpUtilization: 8e5
          }
        },
        state: 'Pending',
        ptokenId: 1002
      },
      balances: [['5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf', '100000000000000000000']]
    }
  ]
}

export default config
