import { sovereignParaOf, sovereignRelayOf } from '../../utils'
import { Command, CreateCommandParameters, program } from '@caporal/core'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand("display parachain's sovereign account")
    .argument('<parachain-id>', 'parachain id', {
      validator: program.NUMBER
    })
    .option('-s,--sibling [boolean]', 'sibling mode', {
      validator: program.BOOLEAN,
      default: false
    })
    .action(actionParameters => {
      const {
        logger,
        args: { parachainId },
        options: { sibling }
      } = actionParameters
      logger.info(
        sibling
          ? sovereignParaOf(parachainId.valueOf() as number)
          : sovereignRelayOf(parachainId.valueOf() as number)
      )
    })
}
