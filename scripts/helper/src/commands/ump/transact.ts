import { createXcm, getApi, nextNonce, sovereignRelayOf } from '../../utils'
import { Command, CreateCommandParameters, program } from '@caporal/core'
import { Keyring } from '@polkadot/api'
import { u32 } from '@polkadot/types'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('open hrmp channel to specific chain')
    .option('-p, --para-ws [url]', 'the parachain API endpoint', {
      default: 'wss://rpc.parallel.fi'
    })
    .option('-e, --encoded-call-data [hex]', 'the hex encoded call data', {
      default: '0x0001081234'
    })
    .option('-d, --dry-run [boolean]', 'whether to execute using PARA_CHAIN_SUDO_KEY', {
      validator: program.BOOLEAN,
      default: true
    })
    .action(async actionParameters => {
      const {
        logger,
        options: { paraWs, dryRun, encodedCallData }
      } = actionParameters
      const api = await getApi(paraWs.toString())
      const paraId = (await api.query.parachainInfo.parachainId()) as unknown as u32
      const signer = new Keyring({ type: 'sr25519' }).addFromUri(
        `${process.env.PARA_CHAIN_SUDO_KEY || '//Dave'}`
      )
      const proposal = api.tx.ormlXcm.sendAsSovereign(
        {
          V1: {
            parents: 1,
            interior: 'Here'
          }
        },
        createXcm(
          `${encodedCallData.toString()}`,
          sovereignRelayOf(paraId.toNumber()),
          'SovereignAccount'
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
