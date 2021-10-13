import { options } from '@parallel-finance/api'
import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'
import config from './config'

function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

async function main() {
  const api = await ApiPromise.create(
    options({
      types: {
        'Compact<TAssetBalance>': 'Compact<Balance>',
        RelaychainBlockNumberOf: 'u32'
      },
      provider: new WsProvider('ws://localhost:9947')
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

  console.log('Wait for block producing')
  do await sleep(1000)
  while (!(await chainHeight()))

  const keyring = new Keyring({ type: 'sr25519', ss58Format: 110 })
  const signer = keyring.addFromUri('//Dave')

  let call = []

  for (const { name, symbol, assetId, decimal, marketOption, balances } of config.assets) {
    console.log(`Create ${name}(${symbol}) asset.`)
    call.push(
      api.tx.sudo.sudo(api.tx.assets.forceCreate(assetId, signer.address, true, 1)),
      api.tx.sudo.sudo(api.tx.assets.forceSetMetadata(assetId, name, symbol, decimal, false)),
      api.tx.sudo.sudo(api.tx.loans.addMarket(assetId, api.createType('Market', marketOption))),
      api.tx.sudo.sudo(api.tx.loans.activeMarket(assetId))
    )
    call.push(...balances.map(([account, amount]) => api.tx.assets.mint(assetId, account, amount)))
  }

  call.push(
    api.tx.sudo.sudo(api.tx.liquidStaking.setLiquidCurrency(config.liquidAsset)),
    api.tx.sudo.sudo(api.tx.liquidStaking.setStakingCurrency(config.stakingAsset))
  )

  console.log('Submit batches.')
  await api.tx.utility.batchAll(call).signAndSend(signer)
  process.exit(0)
}

main()
