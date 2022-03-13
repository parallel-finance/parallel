import { Command, CreateCommandParameters } from '@caporal/core'
import { PersistedValidationData } from '@polkadot/types/interfaces'
import { blake2AsU8a, xxhashAsU8a } from '@polkadot/util-crypto'
import { getApi, getRelayApi } from '../utils'
import { decodeAddress } from '@polkadot/keyring'
import { u8aConcat, u8aToHex } from '@polkadot/util'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('run chain initialization scripts')
    .option('-r, --relay-ws [url]', 'the relaychain API endpoint', {
      default: 'wss://kusama-rpc.polkadot.io'
    })
    .option('-p, --para-ws [url]', 'the parachain API endpoint', {
      default: 'wss://heiko-rpc.parallel.fi'
    })
    .option('-a, --block-at [hash]', 'the parachain block hash', {
      default: '0x69a4182c5a7aef2ae76c58574fc51e71e297089f063af7e0f6efd3a150f67b47'
    })
    .action(async actionParameters => {
      const {
        options: { relayWs, paraWs, blockAt }
      } = actionParameters
      const relayApi = await getRelayApi(relayWs.toString())
      const api = await getApi(paraWs.toString())
      const validationData = (await api.query.parachainSystem.validationData.at(
        blockAt.toString()
      )) as unknown as PersistedValidationData
      console.log(JSON.stringify(validationData, null, 4))
      const relayBlockHash = await relayApi.rpc.chain.getBlockHash(validationData.relayParentNumber)
      const accountBytes = decodeAddress('CmNv7yFV13CMM6r9dJYgdi4UTJK7tzFEF17gmK9c3mTc2PG')
      const storageKey = u8aToHex(
        new Uint8Array([
          ...xxhashAsU8a('Staking', 128),
          ...xxhashAsU8a('Ledger', 128),
          ...u8aConcat(blake2AsU8a(accountBytes, 128), accountBytes)
        ])
      )
      const proof = await relayApi.rpc.state.getReadProof([storageKey], relayBlockHash)
      console.log(storageKey)
      console.log(JSON.stringify(proof, null, 4))
      process.exit(0)
    })
}
