const config = {
  liquidAsset: 1000,
  stakingAsset: 100,
  leaseIndex: 0,
  paraId: 2085,
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
      name: 'Karura Dollar',
      symbol: 'KUSD',
      assetId: 103,
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
        ptokenId: 2103
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
      name: 'Parallel SherpaX Crowdloans Kusama',
      symbol: 'CKSM-KSX',
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
    },
    {
      name: 'Parallel Sakura Crowdloans Kusama',
      symbol: 'CKSM-SKU',
      assetId: 4001,
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
        ptokenId: 3101
      },
      balances: []
    },
    {
      name: 'Parallel Subsocial Crowdloans Kusama',
      symbol: 'CKSM-SUB',
      assetId: 4002,
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
        ptokenId: 3102
      },
      balances: []
    }
  ],
  crowdloans: [
    {
      paraId: 2013,
      derivativeIndex: 0,
      image: 'parallelfinance/polkadot-collator:v0.9.12',
      chain: 'shell',
      ctokenId: 4000,
      cap: '100000000000000000',
      leaseStart: 0,
      leaseEnd: 7
    },
    {
      paraId: 2016,
      derivativeIndex: 1,
      image: 'parallelfinance/polkadot-collator:v0.9.12',
      chain: 'shell',
      ctokenId: 4001,
      cap: '100000000000000000',
      leaseStart: 0,
      leaseEnd: 7
    },
    {
      paraId: 2100,
      derivativeIndex: 2,
      image: 'parallelfinance/polkadot-collator:v0.9.12',
      chain: 'shell',
      ctokenId: 4002,
      cap: '100000000000000000',
      leaseStart: 0,
      leaseEnd: 7
    }
  ]
}

export default config
