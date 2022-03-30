#!/usr/bin/env ts-node

import { program } from '@caporal/core'
import dotenv from 'dotenv'
import path from 'path'

dotenv.config()

program.bin('parallel-helper').discover(path.join(__dirname, 'commands'))

program.run().catch(err => {
  console.error(err.message)
  process.exit(1)
})
