import { createXcm, getApi, nextNonce, sovereignAccountOf } from '../../utils'
import { Command, CreateCommandParameters, program } from '@caporal/core'
import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('open hrmp channel to specific chain')
    .argument('<source>', 'paraId of source chain', {
      validator: program.NUMBER
    })
    .argument('<target>', 'paraId of target chain', {
      validator: program.NUMBER
    })
    .option('-r, --relay-ws [url]', 'the relaychain API endpoint', {
      default: 'wss://kusama-rpc.polkadot.io'
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

      const encoded = await ApiPromise.create({
        provider: new WsProvider(relayWs.toString())
      })
        .then(api => api.tx.hrmp.hrmpInitOpenChannel(target.valueOf() as number, 8, 102400).toHex())
        .then(hex => `0x${hex.slice(6)}`)
      const api = await getApi(paraWs.toString())
      const signer = new Keyring({ type: 'sr25519' }).addFromUri(
        `${process.env.PARA_CHAIN_SUDO_KEY || '//Dave'}`
      )
      await api.tx.sudo
        .sudo(
          api.tx.ormlXcm.sendAsSovereign(
            {
              V1: {
                parents: 1,
                interior: 'Here'
              }
            },
            createXcm(encoded, sovereignAccountOf(source.valueOf() as number))
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
