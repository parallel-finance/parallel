import fs from 'fs'
import shell from 'shelljs'
import dotenv from 'dotenv'
import config from './config.json'
import { blake2AsU8a } from '@polkadot/util-crypto'
import { stringToU8a, bnToU8a, BN, u8aConcat, u8aToHex } from '@polkadot/util'
import { decodeAddress, encodeAddress } from '@polkadot/keyring'
import '@polkadot/api-augment'
import { options } from '@parallel-finance/api'
import { KeyringPair } from '@polkadot/keyring/types'
import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'

const EMPTY_U8A_32 = new Uint8Array(32)
const BN_EIGHTEEN = new BN(18)
const GiftPalletId = 'par/gift'

dotenv.config()

const createAddress = (id: string) =>
  encodeAddress(u8aConcat(stringToU8a(`modl${id}`), EMPTY_U8A_32).subarray(0, 32))

export const sovereignAccountOf = (paraId: number): string =>
  encodeAddress(
    u8aConcat(stringToU8a('para'), bnToU8a(paraId, 32, true), EMPTY_U8A_32).subarray(0, 32)
  )

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

  for (const {
    paraId,
    image,
    chain,
    ctokenId,
    leaseStart,
    leaseEnd,
    cap,
    endBlock,
    pending
  } of config.crowdloans) {
    call.push(
      api.tx.sudo.sudo(
        api.tx.crowdloans.createVault(paraId, ctokenId, leaseStart, leaseEnd, 'XCM', cap, endBlock)
      )
    )
    if (!pending) {
      call.push(api.tx.sudo.sudo(api.tx.crowdloans.open(paraId)))
    }
  }

  for (const { pool, liquidityAmounts, lptokenReceiver, liquidityProviderToken } of config.pools) {
    call.push(
      api.tx.sudo.sudo(
        api.tx.amm.createPool(pool, liquidityAmounts, lptokenReceiver, liquidityProviderToken)
      )
    )
  }

  const { members, chainIds, bridgeTokens } = config.bridge
  members.forEach(member => call.push(api.tx.sudo.sudo(api.tx.bridgeMembership.addMember(member))))
  chainIds.forEach(chainId => call.push(api.tx.sudo.sudo(api.tx.bridge.registerChain(chainId))))
  bridgeTokens.map(
    ({ assetId, id, external, fee }) =>
      call.push(api.tx.sudo.sudo(api.tx.bridge.registerBridgeToken(
        assetId,
        { id, external, fee },
      )))
  )

  call.push(
    api.tx.sudo.sudo(api.tx.liquidStaking.updateMarketCap('10000000000000000')),
    api.tx.sudo.sudo(api.tx.xcmHelper.updateXcmFees('50000000000')),
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
  const chain = await api.rpc.system.chain().then(c => c.toString())

  console.log('Wait for relaychain to produce blocks')
  do await sleep(1000)
  while (!(await chainHeight(api)))

  const keyring = new Keyring({ type: 'sr25519', ss58Format: 2 })
  const signer = keyring.addFromUri(`${process.env.RELAY_CHAIN_SUDO_KEY || ''}`)

  for (const { paraId, image, derivativeIndex, chain, ctokenId } of config.crowdloans) {
    const state = exec(
      `docker run --rm ${image} export-genesis-state --chain ${chain}`
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

  console.log('Start new auction.')
  const call = []
  call.push(api.tx.sudo.sudo(api.tx.auctions.newAuction(config.auctionDuration, config.leaseIndex)))
  call.push(
    ...config.crowdloans.map(({ derivativeIndex }) =>
      api.tx.balances.transfer(subAccountId(signer, derivativeIndex), '100000000000000')
    )
  )
  call.push(
    ...config.crowdloans.map(({ paraId, derivativeIndex, cap, endBlock, leaseStart, leaseEnd }) =>
      api.tx.utility.asDerivative(
        derivativeIndex,
        api.tx.crowdloan.create(paraId, cap, leaseStart, leaseEnd, endBlock, null)
      )
    )
  )

  let relayAsset = config.assets.find(a => a.assetId === config.relayAsset)
  if (relayAsset && relayAsset.balances.length) {
    call.push(
      ...relayAsset.balances.map(([account, balance]) =>
        api.tx.balances.transfer(sovereignAccountOf(config.paraId), balance)
      )
    )
  }

  await api.tx.utility.batchAll(call).signAndSend(signer, { nonce: await nextIndex(api, signer) })
}

relay()
  .then(para)
  .then(() => process.exit(0))
  .catch(err => {
    console.error(err)
    process.exit(1)
  })
