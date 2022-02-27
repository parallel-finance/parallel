import config from './config.json'
import '@polkadot/api-augment'
import { options } from '@parallel-finance/api'
import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'
import {
  chainHeight,
  createAddress,
  nextNonce,
  sleep,
  sovereignAccountOf,
  subAccountId,
  exec
} from '../utils'

const GiftPalletId = 'par/gift'

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

  for (const { name, symbol, assetId, decimal, balances } of config.assets) {
    console.log(`Create ${name}(${symbol}) asset`)
    call.push(
      api.tx.sudo.sudo(api.tx.assets.forceCreate(assetId, signer.address, true, 1)),
      api.tx.sudo.sudo(api.tx.assets.forceSetMetadata(assetId, name, symbol, decimal, false))
    )
    call.push(...balances.map(([account, amount]) => api.tx.assets.mint(assetId, account, amount)))
  }

  for (const { assetId, marketConfig } of config.markets) {
    console.log(`Create market for asset ${assetId}, ptokenId is ${marketConfig.ptokenId}`)
    call.push(
      api.tx.sudo.sudo(api.tx.loans.addMarket(assetId, api.createType('Market', marketConfig))),
      api.tx.sudo.sudo(api.tx.loans.activateMarket(assetId))
    )
  }

  for (const {
    paraId,
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
  bridgeTokens.map(({ assetId, id, external, fee }) =>
    call.push(api.tx.sudo.sudo(api.tx.bridge.registerBridgeToken(assetId, { id, external, fee })))
  )

  call.push(
    api.tx.sudo.sudo(api.tx.liquidStaking.updateMarketCap(config.liquidMarketCap)),
    api.tx.sudo.sudo(api.tx.xcmHelper.updateXcmFees(config.xcmFees)),
    api.tx.balances.transfer(createAddress(GiftPalletId), config.gift)
  )

  console.log('Submit parachain batches.')
  await api.tx.utility.batchAll(call).signAndSend(signer, { nonce: await nextNonce(api, signer) })
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

  for (const { paraId, image, derivativeIndex, chain } of config.crowdloans) {
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
      .signAndSend(signer, { nonce: await nextNonce(api, signer) })
  }

  console.log('Wait parathread to be onboarded.')
  await sleep(360000)

  console.log('Start new auction.')
  const call = []
  call.push(api.tx.sudo.sudo(api.tx.auctions.newAuction(config.auctionDuration, config.leaseIndex)))
  call.push(
    ...config.crowdloans.map(({ derivativeIndex }) =>
      api.tx.balances.transfer(subAccountId(signer, derivativeIndex), config.deposit)
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

  const relayAsset = config.assets.find(a => a.assetId === config.relayAsset)
  if (relayAsset && relayAsset.balances.length) {
    call.push(
      ...relayAsset.balances.map(([, balance]) =>
        api.tx.balances.transfer(sovereignAccountOf(config.paraId), balance)
      )
    )
  }

  await api.tx.utility.batchAll(call).signAndSend(signer, { nonce: await nextNonce(api, signer) })
}

export default async function run(): Promise<void> {
  await relay()
    .then(para)
    .then(() => process.exit(0))
    .catch(err => {
      console.error(err)
      process.exit(1)
    })
}
