import { sovereignAccountOf } from '../utils'
import { Command, CreateCommandParameters, program } from '@caporal/core'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand("display parachain's sovereign account")
    .argument('<parachain-id>', 'parachain id', {
      validator: program.NUMBER
    })
    .action(actionParameters => {
      const {
        logger,
        args: { parachainId }
      } = actionParameters
      logger.info(sovereignAccountOf(parachainId.valueOf() as number))
    })
}
