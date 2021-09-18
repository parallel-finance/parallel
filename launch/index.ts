import {options} from '@parallel-finance/api'
import {ApiPromise, Keyring, WsProvider} from '@polkadot/api'
import {assets} from './assets.json'

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

async function main() {

  const api = await ApiPromise.create(
    options({
      provider: new WsProvider('ws://localhost:9947'),
    })
  )

  const chainHeight = async () => {
    const {
      block: {
        header: {number: height},
      },
    } = await api.rpc.chain.getBlock()
    return height.toNumber()
  }

  do {
    await sleep(1000)
    console.log("Wait for block producing")
  } while (!(await chainHeight()))

  const keyring = new Keyring({type: 'sr25519', ss58Format: 110})
  const signer = keyring.addFromUri('//Dave')

  let call = []

  for (const {name, symbol, assetId, decimal, marketOption} of assets) {
    console.log(`Create ${name}(${symbol}) asset.`)
    call.push(
      api.tx.sudo.sudo(api.tx.assets.forceCreate(assetId, signer.address, true, 1)),
      api.tx.sudo.sudo(
        api.tx.assets.forceSetMetadata(assetId, name, symbol, decimal, false)
      ),
      api.tx.sudo.sudo(
        api.tx.loans.addMarket(
          assetId,
          api.createType('Market', marketOption)
        )
      )
    )
  }

  call.push(
    api.tx.sudo.sudo(api.tx.liquidStaking.setLiquidCurrency(101)),
    api.tx.sudo.sudo(api.tx.liquidStaking.setStakingCurrency(100))
  )

  console.log('Submit batches.')
  await api.tx.utility.batchAll(call).signAndSend(signer)
  process.exit(0)
}

main()
