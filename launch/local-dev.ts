// Import
import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'
import * as fs from 'fs'

function sleep(ms: number) {
	return new Promise(resolve => setTimeout(resolve, ms))
}

async function main() {
	// Construct
	const wsProvider = new WsProvider('ws://localhost:9944')
	const api = await ApiPromise.create({ provider: wsProvider })

	const keyring = new Keyring({ type: 'sr25519', ss58Format: 110 })
	const signer = keyring.addFromUri('//Alice')

	const chain = await api.rpc.system.chain().then(c => c.toString())

	console.log(chain)

	const paraId = 2085

	const state = fs
		.readFileSync(process.cwd() + '/../' + 'resources/heiko-dev-para-2085-genesis')
		.toString()
	const wasm = fs
		.readFileSync(process.cwd() + '/../' + 'resources/heiko-dev-para-2085.wasm')
		.toString()

	let nonce = await api.rpc.system.accountNextIndex(signer.address)

	console.log(`Registering parathread: ${paraId}.`)

	await api.tx.sudo
		.sudo(
			api.tx.parasSudoWrapper.sudoScheduleParaInitialize(paraId, {
				genesisHead: state,
				validationCode: wasm,
				parachain: true
			})
		)
		.signAndSend(signer, { nonce: nonce })

	console.log('Wait parathread to be onboarded.')

	await sleep(360000)

	console.log('🙌 The parachain should be producing blocks')
}

// run
main()
	.catch(console.error)
	.finally(() => process.exit())