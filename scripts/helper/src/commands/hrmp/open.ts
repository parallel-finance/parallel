import { createXcm, getApi, getRelayApi, nextNonce, sovereignRelayOf } from '../../utils'
import { Command, CreateCommandParameters, program } from '@caporal/core'
import { Keyring } from '@polkadot/api'
import { PolkadotRuntimeParachainsConfigurationHostConfiguration } from '@polkadot/types/lookup'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('open hrmp channel to specific chain')
    .argument('<source>', 'paraId of source chain', {
      validator: program.NUMBER
    })
    .argument('<target>', 'paraId of target chain', {
      validator: program.NUMBER
    })
    .option('-r, --relay-ws [url]', 'the relaychain API endpoint', {
      default: 'ws://127.0.0.1:9944'
    })
    .option('-p, --para-ws [url]', 'the parachain API endpoint', {
      default: 'ws://127.0.0.1:9948'
    })
    .action(async actionParameters => {
      const {
        logger,
        args: { source, target },
        options: { relayWs, paraWs }
      } = actionParameters
      const relayApi = await getRelayApi(relayWs.toString())
      const api = await getApi(paraWs.toString())
      const configuration =
        (await relayApi.query.configuration.activeConfig()) as unknown as PolkadotRuntimeParachainsConfigurationHostConfiguration
      const encoded = relayApi.tx.hrmp
        .hrmpInitOpenChannel(
          target.valueOf() as number,
          configuration.hrmpChannelMaxCapacity,
          configuration.hrmpChannelMaxMessageSize
        )
        .toHex()
      const signer = new Keyring({ type: 'sr25519' }).addFromUri(
        `${process.env.PARA_CHAIN_SUDO_KEY || '//Dave'}`
      )
      await api.tx.sudo
        .sudo(
          api.tx.polkadotXcm.send(
            {
              V1: {
                parents: 1,
                interior: 'Here'
              }
            },
            createXcm(`0x${encoded.slice(6)}`, sovereignRelayOf(source.valueOf() as number))
          )
        )
        .signAndSend(signer, { nonce: await nextNonce(api, signer) })
        .then(() => process.exit(0))
        .catch(err => {
          logger.error(err.message)
          process.exit(1)
        })
    })
}
