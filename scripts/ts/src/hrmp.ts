import { ActionParameters } from '@caporal/core'
import '@polkadot/api-augment'
import { ApiPromise, Keyring, options, WsProvider } from '@parallel-finance/api'
import { nextNonce, sovereignAccountOf } from './utils'

const getApi = async (endpoint: string): Promise<ApiPromise> => {
  return ApiPromise.create(
    options({
      types: {
        'Compact<TAssetBalance>': 'Compact<Balance>'
      },
      provider: new WsProvider(endpoint)
    })
  )
}

const createXcm = (api: ApiPromise, encoded: string, sovereignAccount: string) => {
  return {
    V2: [
      {
        WithdrawAsset: [
          {
            id: {
              Concrete: {
                parents: 0,
                interior: 'Here'
              }
            },
            fun: {
              Fungible: '1000000000000'
            }
          }
        ]
      },
      {
        BuyExecution: {
          fees: {
            id: {
              Concrete: {
                parents: 0,
                interior: 'Here'
              }
            },
            fun: {
              Fungible: '1000000000000'
            }
          },
          weightLimit: 'Unlimited'
        }
      },
      {
        Transact: {
          originType: 'Native',
          requireWeightAtMost: '1000000000',
          call: {
            encoded
          }
        }
      },
      {
        DepositAsset: {
          assets: {
            Wild: 'All'
          },
          maxAssets: 1,
          beneficiary: {
            parents: 0,
            interior: {
              X1: {
                AccountId32: {
                  network: 'Any',
                  id: sovereignAccount
                }
              }
            }
          }
        }
      }
    ]
  }
}

export const open = async ({ logger, args: { source, target } }: ActionParameters) => {
  const encoded = await ApiPromise.create({
    provider: new WsProvider('ws://localhost:9944')
  })
    .then(api => api.tx.hrmp.hrmpInitOpenChannel(target.valueOf() as number, 8, 102400).toHex())
    .then(hex => `0x${hex.slice(6)}`)
  const api = await getApi('ws://localhost:9948')
  const signer = new Keyring({ type: 'sr25519', ss58Format: 110 }).addFromUri('//Dave')
  await api.tx.sudo
    .sudo(
      api.tx.ormlXcm.sendAsSovereign(
        {
          V1: {
            parents: 1,
            interior: 'Here'
          }
        },
        createXcm(api, encoded, sovereignAccountOf(source.valueOf() as number))
      )
    )
    .signAndSend(signer, { nonce: await nextNonce(api, signer) })
    .then(() => process.exit(0))
    .catch(err => {
      logger.error(err)
      process.exit(1)
    })
}

export const accept = async ({ logger, args: { source, target } }: ActionParameters) => {
  const encoded = await ApiPromise.create({
    provider: new WsProvider('ws://localhost:9944')
  })
    .then(api => api.tx.hrmp.hrmpAcceptOpenChannel(source.valueOf() as number).toHex())
    .then(hex => `0x${hex.slice(6)}`)
  const api = await getApi('ws://localhost:9948')
  const signer = new Keyring({ type: 'sr25519', ss58Format: 110 }).addFromUri('//Dave')
  await api.tx.sudo
    .sudo(
      api.tx.polkadotXcm.send(
        {
          V1: {
            parents: 1,
            interior: 'Here'
          }
        },
        createXcm(api, encoded, sovereignAccountOf(target.valueOf() as number))
      )
    )
    .signAndSend(signer, { nonce: await nextNonce(api, signer) })
    .then(() => process.exit(0))
    .catch(err => {
      logger.error(err)
      process.exit(1)
    })
}
