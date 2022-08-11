import { calcWeightPerSecond } from '../../utils'
import { Command, CreateCommandParameters, program } from '@caporal/core'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('calculate units_per_second for xcm reserved transfer')
    .argument('<precision>', 'precision of asset', {
      validator: program.NUMBER
    })
    .argument('<price>', 'price of asset', {
      validator: program.NUMBER
    })
    .action(actionParameters => {
      const {
        logger,
        args: { precision, price }
      } = actionParameters
      const precision_num = precision.valueOf() as number
      const price_num = price.valueOf() as number
      const result = calcWeightPerSecond(precision_num, price_num)
      logger.info(result.toString())
    })
}
