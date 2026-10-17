import { readFile, writeFile } from 'node:fs/promises'
import { generateRealtime } from './realtime-generator.mjs'

const contracts = [
  ['../../api/gameplay.schema.json', '../src/game/schema.ts', 'api/gameplay.schema.json'],
  ['../../api/gameplay-v3.schema.json', '../src/game/v3/schema.ts', 'api/gameplay-v3.schema.json'],
  ['../../api/gameplay-v4.schema.json', '../src/game/v4/schema.ts', 'api/gameplay-v4.schema.json'],
  ['../../api/gameplay-v5.schema.json', '../src/game/v5/schema.ts', 'api/gameplay-v5.schema.json'],
  ['../../api/gameplay-v6.schema.json', '../src/game/v6/schema.ts', 'api/gameplay-v6.schema.json'],
  ['../../api/gameplay-v7.schema.json', '../src/game/v7/schema.ts', 'api/gameplay-v7.schema.json'],
  ['../../api/gameplay-v8.schema.json', '../src/game/v8/schema.ts', 'api/gameplay-v8.schema.json'],
]

for (const [specPath, targetPath, sourceLabel] of contracts) {
  const schema = JSON.parse(await readFile(new URL(specPath, import.meta.url), 'utf8'))
  await writeFile(new URL(targetPath, import.meta.url), generateRealtime(schema, sourceLabel))
}
