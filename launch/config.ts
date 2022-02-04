const config = {
  liquidAsset: 1000,
  stakingAsset: 100,
  auctionDuration: 201600,
  leaseIndex: 0,
  paraId: 2085,
  relayAsset: 100,
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
      balances: [
        ['5EYCAe5iie3JmgLB4rm1NHQtyYGiaYYBEB1jt7p35dXjQWJ8', '1000000000000000'],
        ['5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf', '100000000000000000']
      ]
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
      name: 'Parallel Crowdloans Kusama - (0 ~ 7)',
      symbol: 'CKSM-0-7',
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
        ptokenId: 2300
      },
      balances: []
    },
    {
      name: 'Parallel LP-USDT/HKO',
      symbol: 'LP-USDT/HKO',
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
        ptokenId: 2301
      },
      balances: []
    },
    {
      name: 'Parallel LP-KSM/USDT',
      symbol: 'LP-KSM/USDT',
      assetId: 5001,
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
        ptokenId: 2302
      },
      balances: []
    },
    {
      name: 'Parallel LP-KSM/HKO',
      symbol: 'LP-KSM/HKO',
      assetId: 5002,
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
        ptokenId: 2303
      },
      balances: []
    }
  ],
  pools: [
    {
      pool: [102, 0],
      liquidityAmounts: ['100000000000', '10000000000000000'],
      lptokenReceiver: '5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf',
      liquidityProviderToken: 5000
    },
    {
      pool: [102, 100],
      liquidityAmounts: ['15000000000000', '50000000000000000'],
      lptokenReceiver: '5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf',
      liquidityProviderToken: 5001
    },
    {
      pool: [100, 0],
      liquidityAmounts: ['10000000000000000', '30000000000000000000'],
      lptokenReceiver: '5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf',
      liquidityProviderToken: 5002
    }
  ],
  crowdloans: [
    {
      paraId: 2013,
      derivativeIndex: 0,
      image: 'parallelfinance/polkadot-collator:v0.9.16',
      chain: 'shell',
      ctokenId: 4000,
      cap: '100000000000000',
      endBlock: 28800,
      leaseStart: 0,
      leaseEnd: 7,
      pending: false
    },
    {
      paraId: 2016,
      derivativeIndex: 1,
      image: 'parallelfinance/polkadot-collator:v0.9.16',
      chain: 'shell',
      ctokenId: 4000,
      cap: '1000000000000000',
      endBlock: 43200,
      leaseStart: 0,
      leaseEnd: 7,
      pending: true
    },
    {
      paraId: 2100,
      derivativeIndex: 2,
      image: 'parallelfinance/polkadot-collator:v0.9.16',
      chain: 'shell',
      ctokenId: 4000,
      cap: '10000000000000000',
      endBlock: 202800,
      leaseStart: 0,
      leaseEnd: 7,
      pending: false
    }
  ]
}

export default config
