import fs from 'fs'
import shell from 'shelljs'
import dotenv from 'dotenv'
import config from './config'
import { blake2AsU8a } from '@polkadot/util-crypto'
import { stringToU8a, bnToU8a, BN, u8aConcat, u8aToHex } from '@polkadot/util'
import { decodeAddress, encodeAddress } from '@polkadot/keyring'
import { options } from '@parallel-finance/api'
import { KeyringPair } from '@polkadot/keyring/types'
import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'

const EMPTY_U8A_32 = new Uint8Array(32)
const BN_EIGHTEEN = new BN(18)
const GiftPalletId = 'par/gift'
const XcmFeesPalletId = 'par/fees'

const createAddress = (id: string) =>
  encodeAddress(u8aConcat(stringToU8a(`modl${id}`), EMPTY_U8A_32).subarray(0, 32))

dotenv.config()

function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

function exec(cmd: string) {
  console.log(`$ ${cmd}`)
  const res = shell.exec(cmd, { silent: true })
  if (res.code !== 0) {
    console.error('Error: Command failed with code', res.code)
    console.log(res)
  }
  return res
}

async function chainHeight(api: ApiPromise) {
  const {
    block: {
      header: { number: height }
    }
  } = await api.rpc.chain.getBlock()
  return height.toNumber()
}

async function nextIndex(api: ApiPromise, signer: KeyringPair) {
  return await api.rpc.system.accountNextIndex(signer.address)
}

function downwardTransfer(api: ApiPromise, paraId: number, account: string, amount: string) {
  return api.tx.xcmPallet.reserveTransferAssets(
    api.createType('XcmVersionedMultiLocation', {
      V1: api.createType('MultiLocationV1', {
        parents: 0,
        interior: api.createType('JunctionsV1', {
          X1: api.createType('JunctionV1', {
            Parachain: api.createType('Compact<u32>', paraId)
          })
        })
      })
    }),
    api.createType('XcmVersionedMultiLocation', {
      V1: api.createType('MultiLocationV1', {
        parents: 0,
        interior: api.createType('JunctionsV1', {
          X1: api.createType('JunctionV1', {
            AccountId32: {
              network: api.createType('NetworkId', 'Any'),
              id: account
            }
          })
        })
      })
    }),
    api.createType('XcmVersionedMultiAssets', {
      V1: [
        api.createType(' XcmV1MultiAsset', {
          id: api.createType('XcmAssetId', {
            Concrete: api.createType('MultiLocationV1', {
              parents: 0,
              interior: api.createType('JunctionsV1', 'Here')
            })
          }),
          fun: api.createType('FungibilityV1', {
            Fungible: amount
          })
        })
      ]
    }),
    0
  )
}

function subAccountId(signer: KeyringPair, index: number) {
  let seedBytes = stringToU8a('modlpy/utilisuba')
  let whoBytes = decodeAddress(signer.address)
  let indexBytes = bnToU8a(index, 16).reverse()
  let combinedBytes = new Uint8Array(seedBytes.length + whoBytes.length + indexBytes.length)
  combinedBytes.set(seedBytes)
  combinedBytes.set(whoBytes, seedBytes.length)
  combinedBytes.set(indexBytes, seedBytes.length + whoBytes.length)

  let entropy = blake2AsU8a(combinedBytes, 256)
  return encodeAddress(entropy)
}

async function para() {
  const api = await ApiPromise.create(
    options({
      types: {
        'Compact<TAssetBalance>': 'Compact<Balance>'
      },
      provider: new WsProvider('ws://localhost:9948')
    })
  )

  console.log('Wait for parachain to produce blocks')
  do await sleep(1000)
  while (!(await chainHeight(api)))

  const keyring = new Keyring({ type: 'sr25519', ss58Format: 110 })
  const signer = keyring.addFromUri('//Dave')
  const call = []

  for (const { name, symbol, assetId, decimal, marketOption, balances } of config.assets) {
    console.log(`Create ${name}(${symbol}) asset, ptokenId is ${marketOption.ptokenId}`)
    call.push(
      api.tx.sudo.sudo(api.tx.assets.forceCreate(assetId, signer.address, true, 1)),
      api.tx.sudo.sudo(api.tx.assets.forceSetMetadata(assetId, name, symbol, decimal, false)),
      api.tx.sudo.sudo(api.tx.loans.addMarket(assetId, api.createType('Market', marketOption))),
      api.tx.sudo.sudo(api.tx.loans.activateMarket(assetId))
    )
    call.push(...balances.map(([account, amount]) => api.tx.assets.mint(assetId, account, amount)))
  }

  for (const { paraId, image, chain, ctokenId } of config.crowdloans) {
    call.push(api.tx.sudo.sudo(api.tx.crowdloans.createVault(paraId, ctokenId, 'XCM', 'Payer')))
  }

  call.push(
    api.tx.sudo.sudo(api.tx.liquidStaking.setLiquidCurrency(config.liquidAsset)),
    api.tx.sudo.sudo(api.tx.liquidStaking.setStakingCurrency(config.stakingAsset)),
    api.tx.sudo.sudo(api.tx.liquidStaking.updateStakingPoolCapacity('10000000000000000')),
    api.tx.sudo.sudo(api.tx.liquidStaking.updateXcmFeesCompensation('50000000000')),
    api.tx.sudo.sudo(api.tx.crowdloans.updateXcmFees('20000000000')),
    api.tx.balances.transfer(createAddress(GiftPalletId), '1000000000000000')
  )

  console.log('Submit parachain batches.')
  const nonce = await api.rpc.system.accountNextIndex(signer.address)
  await api.tx.utility.batchAll(call).signAndSend(signer, { nonce })
}

async function relay() {
  const api = await ApiPromise.create({
    provider: new WsProvider('ws://localhost:9944')
  })

  console.log('Wait for relaychain to produce blocks')
  do await sleep(1000)
  while (!(await chainHeight(api)))

  const keyring = new Keyring({ type: 'sr25519', ss58Format: 2 })
  const signer = keyring.addFromUri(`${process.env.RELAY_CHAIN_SUDO_KEY || ''}`)

  for (const { paraId, image, derivativeIndex, chain, ctokenId } of config.crowdloans) {
    const state = exec(
      `docker run --rm ${image} export-genesis-state --chain ${chain} --parachain-id ${paraId}`
    ).stdout.trim()
    const wasm = exec(`docker run --rm ${image} export-genesis-wasm --chain ${chain}`).stdout.trim()

    console.log(`Registering parathread: ${paraId}.`)
    await api.tx.sudo
      .sudo(
        api.tx.registrar.forceRegister(
          subAccountId(signer, derivativeIndex),
          config.leaseIndex,
          paraId,
          state,
          wasm
        )
      )
      .signAndSend(signer, { nonce: await nextIndex(api, signer) })
  }

  console.log('Wait parathread to be onboarded.')
  await sleep(360000)

  const height = await chainHeight(api)

  console.log('Start new auction.')
  const call = []
  call.push(api.tx.sudo.sudo(api.tx.auctions.newAuction(1000000, config.leaseIndex)))
  call.push(
    ...config.crowdloans.map(({ derivativeIndex }) =>
      api.tx.balances.transfer(subAccountId(signer, derivativeIndex), '100000000000000')
    )
  )
  call.push(
    ...config.crowdloans.map(({ paraId, derivativeIndex, cap, leaseStart, leaseEnd }) =>
      api.tx.utility.asDerivative(
        derivativeIndex,
        api.tx.crowdloan.create(paraId, cap, leaseStart, leaseEnd, height + 500000, null)
      )
    )
  )
  call.push(
    downwardTransfer(api, config.paraId, createAddress(XcmFeesPalletId), '1000000000000000')
  )

  await api.tx.utility.batchAll(call).signAndSend(signer, { nonce: await nextIndex(api, signer) })
}

Promise.all([relay(), para()])
  .then(() => process.exit(0))
  .catch(err => {
    console.error(err)
    process.exit(1)
  })
