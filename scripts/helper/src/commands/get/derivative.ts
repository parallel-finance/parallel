import { subAccountId } from '../../utils'
import { Command, CreateCommandParameters, program } from '@caporal/core'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('display derivative account address')
    .argument('<address>', 'address of source account')
    .argument('<index>', 'derivative index', {
      validator: program.NUMBER
    })
    .action(actionParameters => {
      const {
        logger,
        args: { address, index }
      } = actionParameters
      logger.info(subAccountId(address.toString(), index.valueOf() as number))
    })
}
