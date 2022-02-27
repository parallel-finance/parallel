#!/usr/bin/env ts-node

import { program } from '@caporal/core'
import dotenv from 'dotenv'
import launch from './launch'

dotenv.config()

program
  .bin('scripts')
  .command('launch', 'run chain initialization scripts')
  .action(async () => {
    await launch()
  })

program.run()
