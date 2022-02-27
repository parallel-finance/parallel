#!/usr/bin/env ts-node

import { program } from '@caporal/core'
import dotenv from 'dotenv'
import path from 'path'

dotenv.config()

program.bin('scripts').discover(path.join(__dirname, 'commands'))

program.run()
