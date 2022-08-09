import '@polkadot/api-augment'
import { ApiPromise, WsProvider } from '@polkadot/api'
import shell from 'shelljs'
import fs from 'fs'
import { blake2AsU8a } from '@polkadot/util-crypto'
import { stringToU8a, bnToU8a, u8aConcat, u8aToHex } from '@polkadot/util'
import { decodeAddress, encodeAddress } from '@polkadot/keyring'
import { KeyringPair } from '@polkadot/keyring/types'
import { Index } from '@polkadot/types/interfaces'
import { promisify } from 'util'

const EMPTY_U8A_32 = new Uint8Array(32)
const XCM_FEE = 2500000000

export const readFile = promisify(fs.readFile)

export const exec = (cmd: string): shell.ShellString => {
  console.log(`$ ${cmd}`)
  const res = shell.exec(cmd, { silent: true })
  if (res.code !== 0) {
    console.error('Error: Command failed with code', res.code)
    console.log(res)
  }
  return res
}

export const sleep = (ms: number): Promise<void> => new Promise(resolve => setTimeout(resolve, ms))

export const chainHeight = async (api: ApiPromise): Promise<number> => {
  const {
    block: {
      header: { number: height }
    }
  } = await api.rpc.chain.getBlock()
  return height.toNumber()
}

export const createAddress = (id: string): string =>
  encodeAddress(u8aConcat(stringToU8a(`modl${id}`), EMPTY_U8A_32).subarray(0, 32))

export const sovereignRelayOf = (paraId: number): string =>
  encodeAddress(
    u8aConcat(stringToU8a('para'), bnToU8a(paraId, 32, true), EMPTY_U8A_32).subarray(0, 32)
  )

export const sovereignParaOf = (paraId: number): string =>
  encodeAddress(
    u8aConcat(stringToU8a('sibl'), bnToU8a(paraId, 32, true), EMPTY_U8A_32).subarray(0, 32)
  )

export const subAccountId = (address: string, index: number): string => {
  const seedBytes = stringToU8a('modlpy/utilisuba')
  const whoBytes = decodeAddress(address)
  const indexBytes = bnToU8a(index, 16).reverse()
  const combinedBytes = new Uint8Array(seedBytes.length + whoBytes.length + indexBytes.length)
  combinedBytes.set(seedBytes)
  combinedBytes.set(whoBytes, seedBytes.length)
  combinedBytes.set(indexBytes, seedBytes.length + whoBytes.length)

  const entropy = blake2AsU8a(combinedBytes, 256)
  return encodeAddress(entropy)
}

export const nextNonce = async (api: ApiPromise, signer: KeyringPair): Promise<Index> => {
  return await api.rpc.system.accountNextIndex(signer.address)
}

export const createXcm = (encoded: string, refundAccount: string, originType = 'Native') => {
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
              Fungible: XCM_FEE
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
              Fungible: XCM_FEE
            }
          },
          weightLimit: 'Unlimited'
        }
      },
      {
        Transact: {
          originType,
          requireWeightAtMost: '3000000000',
          call: {
            encoded
          }
        }
      },
      'RefundSurplus',
      {
        DepositAsset: {
          assets: {
            Wild: {
              AllOf: {
                id: {
                  Concrete: {
                    parents: 0,
                    interior: 'Here'
                  }
                },
                fun: 'Fungible'
              }
            }
          },
          maxAssets: 1,
          beneficiary: {
            parents: 0,
            interior: {
              X1: {
                AccountId32: {
                  network: 'Any',
                  id: u8aToHex(decodeAddress(refundAccount))
                }
              }
            }
          }
        }
      }
    ]
  }
}

export const getApi = async (endpoint: string): Promise<ApiPromise> => {
  return ApiPromise.create({
    provider: new WsProvider(endpoint)
  })
}

export const getRelayApi = async (endpoint: string): Promise<ApiPromise> => {
  return ApiPromise.create({
    provider: new WsProvider(endpoint)
  })
}

export const calcWeightPerSecond = (precision: number,price: number):number => {
  const WEIGHT_PER_SECOND = 10**12;
  // for fixed weigher always 600_000_000
  const weight = 600_000_000; 
  //assume we charge 0.02$ at most for each xcm reserved based transfer
  const max_fee = 0.02;
  /// fee = (weight_per_second * weight)/WEIGHT_PER_SECOND/(10**precision) * price
  /// so weight_per_second = max_fee*WEIGHT_PER_SECOND*(10**precision)/weight/price
  const weight_per_second = max_fee*WEIGHT_PER_SECOND*(10**precision)/weight/price
  /// to avoid price sharply increased later so that we charge too much 
  /// just add some soft limit here
  return Math.min(1000*WEIGHT_PER_SECOND,Math.floor(weight_per_second));
}
