import axios from 'axios'
import https from 'https'
import { Command, CreateCommandParameters, program } from '@caporal/core'
import { Keyring } from '@polkadot/api'
import { blake2AsHex } from '@polkadot/util-crypto'
import { getApi, getCouncilThreshold, nextNonce } from '../../utils'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('runtime upgrade via democracy')
    .option('-p, --para-ws [url]', 'the parachain API endpoint', {
      default: 'wss://heiko-rpc.parallel.fi'
    })
    .option('-r, --runtime-name [name]', 'runtime name', {
      default: 'heiko'
    })
    .option('-v, --runtime-version [version]', 'runtime version', {
      default: 'v1.8.5'
    })
    .option('-b, --blake256-hash [hash]', "runtime code's blake256 hash", {
      default: '0xe1caf000a36540de68a34ed2ce3d70eccd56b05fefda895dd308ee73c53fed40'
    })
    .option('-d, --dry-run [boolean]', 'whether to execute using PARA_CHAIN_SUDO_KEY', {
      validator: program.BOOLEAN,
      default: true
    })
    .action(async actionParameters => {
      const {
        logger,
        options: { paraWs, dryRun, runtimeVersion, runtimeName, blake256Hash }
      } = actionParameters
      const api = await getApi(paraWs.toString())
      const signer = new Keyring({ type: 'sr25519' }).addFromUri(
        `${process.env.PARA_CHAIN_SUDO_KEY || '//Dave'}`
      )
      const url = `https://github.com/parallel-finance/parallel/releases/download/${runtimeVersion.toString()}/${runtimeName.toString()}_runtime.compact.compressed.wasm`
      const res = await axios.get(url, {
        responseType: 'arraybuffer',
        httpsAgent: new https.Agent({ keepAlive: true })
      })
      const code = new Uint8Array(res.data)
      const codeHash = blake2AsHex(code, 256)
      if (codeHash !== blake256Hash.toString()) {
        return logger.error("Runtime code doesn't match blake256Hash")
      }

      const encoded = api.tx.parachainSystem.authorizeUpgrade(codeHash).method.toHex()
      const encodedHash = blake2AsHex(encoded)

      const external = api.tx.democracy.externalProposeMajority({ Legacy: encodedHash })

      const tx = api.tx.utility.batchAll([
        api.tx.preimage.notePreimage(encoded),
        api.tx.generalCouncil.propose(await getCouncilThreshold(api), external, external.length)
      ])
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
