import { Command, CreateCommandParameters } from '@caporal/core'
import { EraIndex, PersistedValidationData, ReadProof } from '@polkadot/types/interfaces'
import { xxhashAsU8a } from '@polkadot/util-crypto'
import { getApi, getRelayApi } from '../../utils'
import { u8aToHex } from '@polkadot/util'
import { Keyring } from '@polkadot/api'
import { Option } from '@polkadot/types'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('Fetch relaychain era and update to parachain')
    .option('-r, --relay-ws [url]', 'the relaychain API endpoint', {
      default: 'ws://127.0.0.1:9944'
    })
    .option('-p, --para-ws [url]', 'the parachain API endpoint', {
      default: 'ws://127.0.0.1:9948'
    })
    .action(async actionParameters => {
      const {
        logger,
        options: { relayWs, paraWs }
      } = actionParameters
      const relayApi = await getRelayApi(relayWs.toString())
      const api = await getApi(paraWs.toString())
      const keyring = new Keyring({ type: 'sr25519' })
      const signer = keyring.addFromUri(`${process.env.PARA_CHAIN_SUDO_KEY || '//Eve'}`)

      const storageKey = u8aToHex(
        new Uint8Array([...xxhashAsU8a('Staking', 128), ...xxhashAsU8a('CurrentEra', 128)])
      )

      const blockHash = await api.rpc.chain.getBlockHash()
      logger.info(`parachain block hash: ${blockHash.toString()}`)

      const maybeValidationData =
        (await api.query.liquidStaking.validationData()) as unknown as Option<PersistedValidationData>
      const validationData = maybeValidationData.unwrap()
      logger.info(JSON.stringify(validationData, null, 4))

      const relayBlockHash = await relayApi.rpc.chain.getBlockHash()
      logger.info(`relaychain block hash: ${relayBlockHash.toString()}`)

      const proof = (await relayApi.rpc.state.getReadProof(
        [storageKey],
        relayBlockHash
      )) as ReadProof
      logger.info(`proof: ${JSON.stringify(proof.proof)}`)

      const maybeEraIndex =
        (await relayApi.query.staking.currentEra()) as unknown as Option<EraIndex>

      const nonce = await api.rpc.system.accountNextIndex(signer.address)
      api.tx.liquidStaking
        .setCurrentEra(maybeEraIndex.unwrap(), proof.proof)
        .signAndSend(signer, { nonce }, ({ events, status }) => {
          if (status.isInBlock) {
            events.forEach(({ event }) => {
              if (api.events.system.ExtrinsicFailed.is(event)) {
                const [dispatchError] = event.data
                let errorInfo

                if (dispatchError.isModule) {
                  const decoded = api.registry.findMetaError(dispatchError.asModule)

                  errorInfo = `${decoded.section}.${decoded.name}`
                } else {
                  errorInfo = dispatchError.toString()
                }
                logger.error(errorInfo)
                process.exit(1)
              }

              if (api.events.system.ExtrinsicSuccess.is(event)) {
                process.exit(0)
              }
            })
          }
        })
    })
}
