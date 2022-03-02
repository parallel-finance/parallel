import heikoConfig from './heiko.json'
import parallelConfig from './parallel.json'

export default function getConfig(network: string) {
  switch (network) {
    case 'vanilla-dev':
    case 'heiko-dev':
      return heikoConfig
    case 'kerria-dev':
    case 'parallel-dev':
      return parallelConfig
    default:
      throw new Error(`unsupported network detected: ${network}`)
  }
}
