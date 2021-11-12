import fs from 'fs'
import shell from 'shelljs'
import { options } from '@parallel-finance/api'
import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'
import dotenv from 'dotenv'
import config from './config'
import '@parallel-finance/types'

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

async function para() {
  const api = await ApiPromise.create(
    options({
      types: {
        'Compact<TAssetBalance>': 'Compact<Balance>'
      },
      provider: new WsProvider('ws://localhost:9948')
    })
  )

  const chainHeight = async () => {
    const {
      block: {
        header: { number: height }
      }
    } = await api.rpc.chain.getBlock()
    return height.toNumber()
  }

  console.log('Wait for parachain to produce blocks')
  do await sleep(1000)
  while (!(await chainHeight()))

  const keyring = new Keyring({ type: 'sr25519', ss58Format: 110 })
  const signer = keyring.addFromUri('//Dave')

  let call = []

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
    call.push(api.tx.sudo.sudo(api.tx.crowdloans.createVault(paraId, ctokenId, 'XCM')))
  }

  call.push(
    api.tx.sudo.sudo(api.tx.liquidStaking.setLiquidCurrency(config.liquidAsset)),
    api.tx.sudo.sudo(api.tx.liquidStaking.setStakingCurrency(config.stakingAsset)),
    api.tx.sudo.sudo(api.tx.liquidStaking.updateStakingPoolCapacity('10000000000000000')),
    api.tx.sudo.sudo(api.tx.liquidStaking.updateXcmFeesCompensation('50000000000')),
    api.tx.sudo.sudo(api.tx.crowdloans.updateXcmFeesCompensation('50000000000'))
  )

  console.log('Submit parachain batches.')
  const nonce = await api.rpc.system.accountNextIndex(signer.address)
  await api.tx.utility.batchAll(call).signAndSend(signer, { nonce })
}

async function relay() {
  const api = await ApiPromise.create({
    provider: new WsProvider('ws://localhost:9944')
  })

  const chainHeight = async () => {
    const {
      block: {
        header: { number: height }
      }
    } = await api.rpc.chain.getBlock()
    return height.toNumber()
  }

  console.log('Wait for relaychain to produce blocks')
  do await sleep(1000)
  while (!(await chainHeight()))

  const keyring = new Keyring({ type: 'sr25519', ss58Format: 2 })
  const signer = keyring.addFromUri(`${process.env.RELAY_CHAIN_SUDO_KEY || ''}`)

  let call = []

  for (const { paraId, image, chain, ctokenId } of config.crowdloans) {
    const state = exec(
      `docker run --rm ${image} export-genesis-state --chain ${chain} --parachain-id ${paraId}`
    ).stdout.trim()
    const wasm = exec(`docker run --rm ${image} export-genesis-wasm --chain ${chain}`).stdout.trim()
    call.push(
      api.tx.sudo.sudo(api.tx.registrar.forceRegister(signer.address, 100, paraId, state, wasm))
    )
  }

  console.log('Submit relaychain batches.')
  await api.tx.utility
    .batchAll(call)
    .signAndSend(signer, { nonce: await api.rpc.system.accountNextIndex(signer.address) })
  console.log('Wait parathread to be onboarded.')
  await sleep(360000)

  const height = await chainHeight()

  call.push(api.tx.sudo.sudo(api.tx.auctions.newAuction(1000000, 0)))
  call.push(
    ...config.crowdloans.map(({ paraId }) =>
      api.tx.crowdloan.create(paraId, '1000000000000000000', 0, 7, height + 500000, null)
    )
  )

  await api.tx.utility
    .batchAll(call)
    .signAndSend(signer, { nonce: await api.rpc.system.accountNextIndex(signer.address) })
}

relay()
  .then(para)
  .then(() => process.exit(0))
  .catch(() => process.exit(1))
