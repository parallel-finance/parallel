import { createXcm, getApi, getRelayApi, nextNonce, sovereignRelayOf } from '../../utils'
import { Command, CreateCommandParameters, program } from '@caporal/core'
import { Keyring } from '@polkadot/api'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('accept hrmp channel from specific chain')
    .argument('<source>', 'paraId of source chain', {
      validator: program.NUMBER
    })
    .argument('<target>', 'paraId of target chain', {
      validator: program.NUMBER
    })
    .option('-r, --relay-ws [url]', 'the relaychain API endpoint', {
      default: 'wss://rpc.polkadot.io'
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
        args: { source, target },
        options: { relayWs, paraWs, dryRun }
      } = actionParameters
      const relayApi = await getRelayApi(relayWs.toString())
      const encoded = relayApi.tx.hrmp.hrmpAcceptOpenChannel(source.valueOf() as number).toHex()
      const api = await getApi(paraWs.toString())
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
        createXcm(`0x${encoded.slice(6)}`, sovereignRelayOf(target.valueOf() as number))
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
