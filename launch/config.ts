const config = {
  liquidAsset: 1000,
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
        ptokenId: 2100
      },
      balances: []
    },
    {
      name: 'Parallel Staking Kusama',
      symbol: 'XKSM',
      assetId: 1000,
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
        ptokenId: 3000
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
        ptokenId: 2102
      },
      balances: [['5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf', '100000000000000000000']]
    },
    {
      name: 'Parallel Crowdloans Kusama',
      symbol: 'CKSM',
      assetId: 4000,
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
        ptokenId: 3100
      },
      balances: []
    }
  ],
  crowdloans: [
    {
      paraId: 2000,
      derivativeIndex: 0,
      image: 'parallelfinance/polkadot-collator:v0.9.12',
      chain: 'shell',
      ctokenId: 4000
    }
  ]
}

export default config
