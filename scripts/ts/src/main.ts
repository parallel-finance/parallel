#!/usr/bin/env ts-node

import { ActionParameters, program } from '@caporal/core'
import dotenv from 'dotenv'
import launch from './launch'
import * as hrmp from './hrmp'

dotenv.config()

program
  .bin('scripts')
  // launch
  .command('launch', 'run chain initialization scripts')
  .action(async (parameters: ActionParameters) => {
    await launch(parameters)
  })
  // hrmp-open
  .command('hrmp-open', 'open hrmp channel to specific chain')
  .argument('<source>', 'paraId of source chain', {
    validator: program.NUMBER
  })
  .argument('<target>', 'paraId of target chain', {
    validator: program.NUMBER
  })
  .action(async (parameters: ActionParameters) => {
    await hrmp.open(parameters)
  })
  // hrmp-accept
  .command('hrmp-accept', 'accept hrmp channel from specific chain')
  .argument('<source>', 'paraId of source chain', {
    validator: program.NUMBER
  })
  .argument('<target>', 'paraId of target chain', {
    validator: program.NUMBER
  })
  .action(async (parameters: ActionParameters) => {
    await hrmp.accept(parameters)
  })

program.run()
