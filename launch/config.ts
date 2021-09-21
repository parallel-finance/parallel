import BN from 'bn.js';

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
        liquidateIncentive: new BN('1100000000000000000'),
        rateModel: {
          jumpModel: {
            baseRate: new BN('20000000000000000'),
            jumpRate: new BN('100000000000000000'),
            fullRate: new BN('320000000000000000'),
            jumpUtilization: 8e5
          }
        },
        state: 'Pending'
      }
    },
    {
      name: 'Parallel Kusama',
      symbol: 'XKSM',
      assetId: 1000,
      decimal: 12,
      marketOption: {
        closeFactor: 50e4,
        collateralFactor: 50e4,
        reserveFactor: 15e4,
        liquidateIncentive: new BN('1100000000000000000'),
        rateModel: {
          jumpModel: {
            baseRate: new BN('20000000000000000'),
            jumpRate: new BN('100000000000000000'),
            fullRate: new BN('320000000000000000'),
            jumpUtilization: 8e5
          }
        },
        state: 'Pending'
      }
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
        liquidateIncentive: new BN('1100000000000000000'),
        rateModel: {
          jumpModel: {
            baseRate: new BN('20000000000000000'),
            jumpRate: new BN('100000000000000000'),
            fullRate: new BN('320000000000000000'),
            jumpUtilization: 8e5
          }
        },
        state: 'Pending'
      }
    }
  ]
};

export default config;
