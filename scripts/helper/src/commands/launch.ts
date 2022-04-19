import getConfig from '../config'
import '@polkadot/api-augment'
import { Keyring } from '@polkadot/api'
import {
  chainHeight,
  createAddress,
  nextNonce,
  sleep,
  sovereignRelayOf,
  subAccountId,
  exec,
  getApi,
  getRelayApi
} from '../utils'
import { ActionParameters, Command, CreateCommandParameters } from '@caporal/core'

const GiftPalletId = 'par/gift'

async function para({ logger, options: { paraWs, network } }: ActionParameters) {
  const config = getConfig(network.valueOf() as string)
  const api = await getApi(paraWs.toString())

  logger.info('Wait for parachain to produce blocks')
  do await sleep(1000)
  while (!(await chainHeight(api)))

  const keyring = new Keyring({ type: 'sr25519' })
  const signer = keyring.addFromUri(`${process.env.PARA_CHAIN_SUDO_KEY || '//Dave'}`)
  const call = []

  for (const { name, symbol, assetId, decimal, balances } of config.assets) {
    logger.info(`Create ${name}(${symbol}) asset`)
    call.push(
      api.tx.sudo.sudo(api.tx.assets.forceCreate(assetId, signer.address, true, 1)),
      api.tx.sudo.sudo(api.tx.assets.forceSetMetadata(assetId, name, symbol, decimal, false))
    )
    call.push(...balances.map(([account, amount]) => api.tx.assets.mint(assetId, account, amount)))
  }

  for (const { assetId, marketConfig } of config.markets) {
    logger.info(`Create market for asset ${assetId}, ptokenId is ${marketConfig.ptokenId}`)
    call.push(
      api.tx.sudo.sudo(api.tx.loans.addMarket(assetId, marketConfig)),
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
  bridgeTokens.map(({ assetId, id, external, fee, enable, outCap, outAmount, inCap, inAmount }) =>
    call.push(
      api.tx.sudo.sudo(
        api.tx.bridge.registerBridgeToken(assetId, {
          id,
          external,
          fee,
          enable,
          outCap,
          outAmount,
          inCap,
          inAmount
        })
      )
    )
  )

  call.push(
    api.tx.sudo.sudo(api.tx.liquidStaking.updateStakingLedgerCap(config.stakingLedgerCap)),
    api.tx.sudo.sudo(api.tx.liquidStaking.forceSetEraStartBlock(61)),
    api.tx.sudo.sudo(api.tx.liquidStaking.forceSetCurrentEra(3)),
    api.tx.balances.transfer(createAddress(GiftPalletId), config.gift)
  )

  for (const {
    assetId,
    rewardAssetId,
    lockDuration,
    coolDownDuration,
    rewardAmount,
    rewardDuration
  } of config.farmPools) {
    logger.info(`Create farming pool for asset ${assetId}`)
    call.push(
      api.tx.sudo.sudo(
        api.tx.farming.create(assetId, rewardAssetId, lockDuration, coolDownDuration)
      ),
      api.tx.sudo.sudo(api.tx.farming.setPoolStatus(assetId, rewardAssetId, lockDuration, true)),
      api.tx.sudo.sudo(
        api.tx.farming.dispatchReward(
          assetId,
          rewardAssetId,
          lockDuration,
          signer.address,
          rewardAmount,
          rewardDuration
        )
      )
    )
  }

  logger.info('Submit parachain batches.')
  await api.tx.utility.batchAll(call).signAndSend(signer, { nonce: await nextNonce(api, signer) })
}

async function relay({ logger, options: { relayWs, network } }: ActionParameters) {
  const config = getConfig(network.valueOf() as string)
  const api = await getRelayApi(relayWs.toString())

  logger.info('Wait for relaychain to produce blocks')
  do await sleep(1000)
  while (!(await chainHeight(api)))

  const keyring = new Keyring({ type: 'sr25519' })
  const signer = keyring.addFromUri(`${process.env.RELAY_CHAIN_SUDO_KEY || ''}`)

  for (const { paraId, image, derivativeIndex, chain } of config.crowdloans) {
    const state = exec(
      `docker run --rm ${image} export-genesis-state --chain ${chain}`
    ).stdout.trim()
    const wasm = exec(`docker run --rm ${image} export-genesis-wasm --chain ${chain}`).stdout.trim()

    logger.info(`Registering parathread: ${paraId}.`)
    await api.tx.sudo
      .sudo(
        api.tx.registrar.forceRegister(
          subAccountId(signer.address, derivativeIndex),
          config.paraDeposit,
          paraId,
          state,
          wasm
        )
      )
      .signAndSend(signer, { nonce: await nextNonce(api, signer) })
  }

  logger.info('Wait parathread to be onboarded.')
  await sleep(360000)

  logger.info('Start new auction.')
  const call = []
  call.push(api.tx.sudo.sudo(api.tx.auctions.newAuction(config.auctionDuration, config.leaseIndex)))
  call.push(
    ...config.crowdloans.map(({ derivativeIndex }) =>
      api.tx.balances.transfer(
        subAccountId(signer.address, derivativeIndex),
        config.crowdloanDeposit
      )
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
        api.tx.balances.transfer(sovereignRelayOf(config.paraId), balance)
      )
    )
  }

  await api.tx.utility.batchAll(call).signAndSend(signer, { nonce: await nextNonce(api, signer) })
}

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('run chain initialization scripts')
    .option('-r, --relay-ws [url]', 'the relaychain API endpoint', {
      default: 'ws://127.0.0.1:9944'
    })
    .option('-p, --para-ws [url]', 'the parachain API endpoint', {
      default: 'ws://127.0.0.1:9948'
    })
    .option('-n, --network [name]', 'the parachain network', {
      default: 'vanilla-dev'
    })
    .action(actionParameters => {
      const { logger } = actionParameters
      return relay(actionParameters)
        .then(() => para(actionParameters))
        .then(() => process.exit(0))
        .catch(err => {
          logger.error(err.message)
          process.exit(1)
        })
    })
}
