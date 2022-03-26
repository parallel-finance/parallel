import { Command, CreateCommandParameters } from '@caporal/core'
import {
  ParaId,
  PersistedValidationData,
  ReadProof,
  StakingLedger
} from '@polkadot/types/interfaces'
import { blake2AsU8a, xxhashAsU8a } from '@polkadot/util-crypto'
import { getApi, getRelayApi, sovereignRelayOf, subAccountId } from '../utils'
import { decodeAddress } from '@polkadot/keyring'
import { u8aConcat, u8aToHex } from '@polkadot/util'
import { Keyring } from '@polkadot/api'
import { Option, u16 } from '@polkadot/types'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('Fetch relaychain ledger and update to parachain')
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

      const paraId = (await api.query.parachainInfo.parachainId()) as ParaId
      const derivativeIndex = (await api.consts.liquidStaking.derivativeIndex) as u16
      const controllerAddress = subAccountId(
        sovereignRelayOf(paraId.toNumber()),
        derivativeIndex.toNumber()
      )
      const accountBytes = decodeAddress(controllerAddress)
      const storageKey = u8aToHex(
        new Uint8Array([
          ...xxhashAsU8a('Staking', 128),
          ...xxhashAsU8a('Ledger', 128),
          ...u8aConcat(blake2AsU8a(accountBytes, 128), accountBytes)
        ])
      )

      const blockHash = await api.rpc.chain.getBlockHash()
      logger.info(`parachain block hash: ${blockHash.toString()}`)

      const maybeValidationData = (await api.query.liquidStaking.validationData.at(
        blockHash
      )) as unknown as Option<PersistedValidationData>
      const validationData = maybeValidationData.unwrap()
      logger.info(JSON.stringify(validationData, null, 4))

      const relayBlockHash = await relayApi.rpc.chain.getBlockHash(validationData.relayParentNumber)
      logger.info(`relaychain block hash: ${relayBlockHash.toString()}`)

      const proof = (await relayApi.rpc.state.getReadProof(
        [storageKey],
        relayBlockHash
      )) as ReadProof

      const maybeLedger = (await relayApi.query.staking.ledger.at(
        relayBlockHash,
        controllerAddress
      )) as unknown as Option<StakingLedger>

      const nonce = await api.rpc.system.accountNextIndex(signer.address)
      api.tx.liquidStaking
        .setStakingLedger(derivativeIndex, maybeLedger.unwrap(), proof.proof)
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
