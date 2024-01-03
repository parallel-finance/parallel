import { getApi, getCouncilThreshold, nextNonce } from '../../utils'
import { Command, CreateCommandParameters, program } from '@caporal/core'
import { Keyring } from '@polkadot/api'
import { readFile } from '../../utils'
import BigNumber from 'bignumber.js'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('set farming reward speed')
    .argument('<input>', 'path to reward csv', {
      validator: program.STRING
    })
    .option('-p, --para-ws [url]', 'the parachain API endpoint', {
      default: 'wss://polkadot-parallel-rpc.parallel.fi'
    })
    .option('-d, --dry-run [boolean]', 'whether to execute using PARA_CHAIN_SUDO_KEY', {
      validator: program.BOOLEAN,
      default: true
    })
    .action(async actionParameters => {
      const {
        logger,
        args: { input },
        options: { paraWs, dryRun }
      } = actionParameters
      const api = await getApi(paraWs.toString())
      const signer = new Keyring({ type: 'sr25519' }).addFromUri(
        `${process.env.PARA_CHAIN_SUDO_KEY || '//Dave'}`
      )
      const isParallel = (await api.rpc.system.chain()).toString() === 'Parallel'
      const payer = isParallel
        ? 'p8B3QXweBQKzu8DhkggwJqFkUVQ53kB1RejtFQ8q3JMSFqqMd'
        : 'hJFHzsKENPsaqPJT2k6D4VYUKz2eFxxW7AVfG9zvL3Q1R7sFp'
      const rewardAsset = isParallel ? '1' : '0'
      const lockDuration = '0'
      const blockNumber = (await api.rpc.chain.getHeader()).number.toBn()

      const rewards = (await readFile(input.toString(), 'utf8'))
        .split(/\r?\n/)
        .slice(1)
        .map(row => row.split(',').filter(Boolean))
        .filter(cols => cols.length >= 4)
        .map(([assetId, assetName, amount, rewardDuration]) => [
          assetId,
          assetName,
          new BigNumber(amount).multipliedBy('1000000000000').toString(),
          new BigNumber(rewardDuration).toString()
        ])

      const proposal = api.tx.utility.batchAll(
        await Promise.all(
          rewards.map(async ([assetId, assetName, amount, rewardDuration]) => {
            logger.info(
              ` assetId: ${assetId}, assetName: ${assetName}, amount: ${amount}, rewardDuration: ${rewardDuration} `
            )
            // eslint-disable-next-line
            const pool = (await api.query.farming.pools(assetId, rewardAsset, null)) as any
            if (pool && pool.unwrapOrDefault().periodFinish.toBn().gt(blockNumber)) {
              amount = '0'
            }
            return api.tx.farming.dispatchReward(
              assetId,
              rewardAsset,
              lockDuration,
              { Id: payer },
              amount,
              rewardDuration
            )
          })
        )
      )

      const tx = api.tx.generalCouncil.propose(
        await getCouncilThreshold(api),
        proposal,
        proposal.length
      )

      if (dryRun) {
        return logger.info(`hex-encoded call: ${tx.toHex()}`)
      }

      await tx
        .signAndSend(signer, { nonce: await nextNonce(api, signer) })
        .then(() => process.exit(0))
        .catch(err => {
          logger.error(err.message)
          process.exit(1)
        })
    })
}
