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
        options: { relayWs, paraWs }
      } = actionParameters
      const relayApi = await getRelayApi(relayWs.toString())
      const api = await getApi(paraWs.toString())

      const block_hash = await api.rpc.chain.getBlockHash()
      console.log("parachain block hash: "+block_hash.toString());

      const validationDataOp =
        (await api.query.liquidStaking.validationData.at(block_hash.toString())) as unknown as Option<PersistedValidationData>
      const validationData = validationDataOp.unwrap()

      // const relayParentNumber = validationData.relayParentNumber
      console.log(JSON.stringify(validationData, null, 4))
      const relayBlockHash = await relayApi.rpc.chain.getBlockHash()
      console.log('relaychain block hash: ' + relayBlockHash.toString())

      const staking_address = 'JBh7nK81VPFHBgZdd2R5sasicKTvRWFzLgpUd59jYchWmqn'
      const accountBytes = decodeAddress(staking_address)
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

      // console.log(storageKey)
      // console.log(JSON.stringify(proof, null, 4))
      const proof_bytes = proof.proof

      const ledgerOp = (await relayApi.query.staking.ledger(
        staking_address
      )) as unknown as Option<StakingLedger>
      // console.log(JSON.stringify(ledgerOp, null, 4))
      const ledger = ledgerOp.unwrap()

      const keyring = new Keyring({ type: 'sr25519' })
      const signer = keyring.addFromUri(`${process.env.PARA_CHAIN_SUDO_KEY || '//Dave'}`)
      await api.tx.sudo
        .sudo(api.tx.liquidStaking.setStakingLedger(0, ledger, proof_bytes))
        .signAndSend(signer, ({ events = [], status }) => {
          if (status.isInBlock) {
            console.log('Successful with hash ' + status.asInBlock.toHex())
          } else {
            console.log('Status of transfer: ' + status.type)
          }

          events.forEach(({ phase, event: { data, method, section } }) => {
            console.log(phase.toString() + ' : ' + section + '.' + method + ' ' + data.toString())
          })
        })

      process.exit(0)
    })
}
