import { Command, CreateCommandParameters } from '@caporal/core'
import { PersistedValidationData, ReadProof,StakingLedger } from '@polkadot/types/interfaces'
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

//      const block_hash = await api.rpc.chain.getBlockHash()
  //    console.log("parachain block hash: "+block_hash.toString());
      
    //  const validationData = (await api.query.parachainSystem.validationData.at(block_hash.toString())) as unknown as PersistedValidationData
//	const relayParentNumber = validationData.relayParentNumber
  //    console.log(JSON.stringify(validationData, null, 4))
      
//      const relayBlockHash = await relayApi.rpc.chain.getBlockHash(relayParentNumber)
      const relayBlockHash = await relayApi.rpc.chain.getBlockHash()
//const relayBlockHash = await relayApi.rpc.chain.getFinalizedHead()  
    console.log("relaychain block hash: "+relayBlockHash.toString());

      let address_2012_0 = 'JBh7nK81VPFHBgZdd2R5sasicKTvRWFzLgpUd59jYchWmqn';
      const accountBytes = decodeAddress(address_2012_0)
      const storageKey = u8aToHex(
        new Uint8Array([
          ...xxhashAsU8a('Staking', 128),
          ...xxhashAsU8a('Ledger', 128),
          ...u8aConcat(blake2AsU8a(accountBytes, 128), accountBytes)
        ])
      )
      const proof = (await relayApi.rpc.state.getReadProof([storageKey], relayBlockHash)) as ReadProof 
       
      // console.log(storageKey)
      // console.log(JSON.stringify(proof, null, 4))
      let proof_bytes = proof.proof;

      const ledgerOp = (await relayApi.query.staking.ledger(address_2012_0) )as unknown as Option<StakingLedger>
      // console.log(JSON.stringify(ledgerOp, null, 4))
      const ledger = ledgerOp.unwrap()

      const keyring = new Keyring({ type: 'sr25519' })
      const signer = keyring.addFromUri(`${process.env.PARA_CHAIN_SUDO_KEY || '//Dave'}`)
      await api.tx.sudo.sudo(api.tx.liquidStaking.setStakingLedger(0, ledger, proof_bytes)).signAndSend(signer, ({ events = [], status }) => {
        if (status.isInBlock) {
          console.log('Successful with hash ' + status.asInBlock.toHex());
        } else {
          console.log('Status of transfer: ' + status.type);
        }
      
        events.forEach(({ phase, event: { data, method, section } }) => {
          console.log(phase.toString() + ' : ' + section + '.' + method + ' ' + data.toString());
        });
      });

      process.exit(0)
    })
}
