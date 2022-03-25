import { Command, CreateCommandParameters } from '@caporal/core'
import { PersistedValidationData, ReadProof, StakingLedger } from '@polkadot/types/interfaces'
import { blake2AsU8a, xxhashAsU8a } from '@polkadot/util-crypto'
import { getApi, getRelayApi } from '../utils'
import { decodeAddress } from '@polkadot/keyring'
import { u8aConcat, u8aToHex } from '@polkadot/util'
import { Keyring } from '@polkadot/api'
import { Option } from '@polkadot/types'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('Fetch Relaychain Ledger and update to Parachain')
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

      const blockHash = await api.rpc.chain.getBlockHash()
      logger.info('parachain block hash: ' + blockHash.toString())

      const maybeValidationData = (await api.query.liquidStaking.validationData.at(
        blockHash.toString()
      )) as unknown as Option<PersistedValidationData>
      const validationData = maybeValidationData.unwrap()

      // const relayParentNumber = validationData.relayParentNumber
      logger.info(JSON.stringify(validationData, null, 4))
      const relayBlockHash = await relayApi.rpc.chain.getBlockHash()
      logger.info('relaychain block hash: ' + relayBlockHash.toString())

      const controllerAddress = 'EgPaU9rG6nxwQ4k1HTDzUS9acjig14r1hBM3V9JERrgviHU'
      const accountBytes = decodeAddress(controllerAddress)
      const storageKey = u8aToHex(
        new Uint8Array([
          ...xxhashAsU8a('Staking', 128),
          ...xxhashAsU8a('Ledger', 128),
          ...u8aConcat(blake2AsU8a(accountBytes, 128), accountBytes)
        ])
      )
      const proof = (await relayApi.rpc.state.getReadProof(
        [storageKey],
        relayBlockHash
      )) as ReadProof

      // logger.info(storageKey)
      // logger.info(JSON.stringify(proof, null, 4))
      const proof_bytes = proof.proof

      const mnaybeLedger = (await relayApi.query.staking.ledger(
        controllerAddress
      )) as unknown as Option<StakingLedger>
      // logger.info(JSON.stringify(ledgerOp, null, 4))
      const ledger = mnaybeLedger.unwrap()

      const keyring = new Keyring({ type: 'sr25519' })
      const signer = keyring.addFromUri(`${process.env.PARA_CHAIN_SUDO_KEY || '//Dave'}`)
      await api.tx.sudo
        .sudo(
          api.tx.liquidStaking.setStakingLedger(
            api.consts.liquidStaking.derivativeIndex.toString(),
            ledger,
            proof_bytes
          )
        )
        .signAndSend(signer, ({ events = [], status }) => {
          if (status.isInBlock) {
            logger.info('Successful with hash ' + status.asInBlock.toHex())
          } else {
            logger.info('Status of transfer: ' + status.type)
          }

          events.forEach(({ phase, event: { data, method, section } }) => {
            logger.info(phase.toString() + ' : ' + section + '.' + method + ' ' + data.toString())
          })
        })

      process.exit(0)
    })
}
