import { getApi, getCouncilThreshold, nextNonce } from '../../utils'
import { Command, CreateCommandParameters, program } from '@caporal/core'
import { Keyring } from '@polkadot/api'
import { readFile } from '../../utils'
import BigNumber from 'bignumber.js'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('set market reward speed')
    .argument('<input>', 'path to reward csv', {
      validator: program.STRING
    })
    .option('-p, --para-ws [url]', 'the parachain API endpoint', {
      default: 'wss://parallel-rpc.dwellir.com'
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

      const rewards = (await readFile(input.toString(), 'utf8'))
        .split(/\r?\n/)
        .slice(1)
        .map(row => row.split(',').filter(Boolean))
        .filter(cols => cols.length >= 4)
        .map(([assetId, assetName, borrowSpeed, supplySpeed]) => [
          assetId,
          assetName,
          new BigNumber(borrowSpeed).multipliedBy('1000000000000').toString(),
          new BigNumber(supplySpeed).multipliedBy('1000000000000').toString()
        ])

      const proposal = api.tx.utility.batchAll(
        rewards.map(([assetId, assetName, borrowSpeed, supplySpeed]) => {
          logger.info(
            ` assetId: ${assetId}, assetName: ${assetName}, borrowSpeed: ${borrowSpeed}, supplySpeed: ${supplySpeed} `
          )
          return api.tx.loans.updateMarketRewardSpeed(assetId, supplySpeed, borrowSpeed)
        })
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
