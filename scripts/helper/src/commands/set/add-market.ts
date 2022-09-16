import { getApi, nextNonce } from '../../utils'
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
      default: 'wss://rpc.parallel.fi'
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

      const markets = (await readFile(input.toString(), 'utf8'))
        .split(/\r\n/)
        .slice(1)
        .map(row => row.split(',').filter(Boolean))
        .filter(cols => cols.length >= 4)
        .map(
          ([
            assetId,
            collateralFactor,
            liquidationThreshold,
            reserveFactor,
            closeFactor,
            liquidateIncentive,
            liquidateIncentiveReservedFactor,
            baseRate,
            jumpRate,
            fullRate,
            jumpUtilization,
            state,
            supplyCap,
            borrowCap,
            ptokenId
          ]) => [
            assetId,
            new BigNumber(collateralFactor).multipliedBy('1000000').toString(),
            new BigNumber(liquidationThreshold).multipliedBy('1000000').toString(),
            new BigNumber(reserveFactor).multipliedBy('1000000').toString(),
            new BigNumber(closeFactor).multipliedBy('1000000').toString(),
            new BigNumber(liquidateIncentive).multipliedBy('1000000000000000000').toString(),
            new BigNumber(liquidateIncentiveReservedFactor).multipliedBy('1000000').toString(),
            new BigNumber(baseRate).multipliedBy('1000000000000000000').toString(),
            new BigNumber(jumpRate).multipliedBy('1000000000000000000').toString(),
            new BigNumber(fullRate).multipliedBy('1000000000000000000').toString(),
            new BigNumber(jumpUtilization).multipliedBy('1000000').toString(),
            state,
            new BigNumber(supplyCap).multipliedBy('1000000000000').toString(),
            new BigNumber(borrowCap).multipliedBy('1000000000000').toString(),
            ptokenId
          ]
        )

      const proposal = api.tx.utility.batchAll(
        markets.map(
          ([
            assetId,
            collateralFactor,
            liquidationThreshold,
            reserveFactor,
            closeFactor,
            liquidateIncentive,
            liquidateIncentiveReservedFactor,
            baseRate,
            jumpRate,
            fullRate,
            jumpUtilization,
            state,
            supplyCap,
            borrowCap,
            ptokenId
          ]) => {
            const rateModel = {
              Jump: {
                baseRate,
                jumpRate,
                fullRate,
                jumpUtilization
              }
            }
            const market = {
              collateralFactor,
              liquidationThreshold,
              reserveFactor,
              closeFactor,
              liquidateIncentive,
              liquidateIncentiveReservedFactor,
              rateModel,
              state,
              supplyCap,
              borrowCap,
              ptokenId
            }
            logger.info(JSON.stringify(market))

            return api.tx.loans.addMarket(assetId, market)
          }
        )
      )

      const tx = api.tx.generalCouncil.propose(3, proposal, proposal.length)

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
