import { readFile } from 'node:fs/promises'
import { expect, test } from 'vitest'

import { discoverGameplayContracts } from './gameplay-contract-inventory.mjs'
import { generateRealtime } from './realtime-generator.mjs'

test('all gameplay contracts remain byte-for-byte reproducible', async () => {
  for (const { spec, target, sourceLabel } of await discoverGameplayContracts()) {
    const schema = JSON.parse(await readFile(spec, 'utf8'))
    const generated = generateRealtime(schema, sourceLabel)
    expect(generated).toBe(await readFile(target, 'utf8'))
  }
})
