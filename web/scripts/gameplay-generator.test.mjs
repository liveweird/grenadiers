import { readFile } from 'node:fs/promises'
import { expect, test } from 'vitest'

import { generateRealtime } from './realtime-generator.mjs'

const contracts = [
  ['../../api/gameplay.schema.json', '../src/game/schema.ts', 'api/gameplay.schema.json'],
  ['../../api/gameplay-v3.schema.json', '../src/game/v3/schema.ts', 'api/gameplay-v3.schema.json'],
]

test('both gameplay contracts remain byte-for-byte reproducible', async () => {
  for (const [specPath, targetPath, sourceLabel] of contracts) {
    const schema = JSON.parse(await readFile(new URL(specPath, import.meta.url), 'utf8'))
    const generated = generateRealtime(schema, sourceLabel)
    expect(generated).toBe(await readFile(new URL(targetPath, import.meta.url), 'utf8'))
  }
})
