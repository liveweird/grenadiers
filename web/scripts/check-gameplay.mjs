import { readFile } from 'node:fs/promises'
import { generateRealtime } from './realtime-generator.mjs'

const contracts = [
  ['../../api/gameplay.schema.json', '../src/game/schema.ts', 'api/gameplay.schema.json'],
  ['../../api/gameplay-v3.schema.json', '../src/game/v3/schema.ts', 'api/gameplay-v3.schema.json'],
]

for (const [specPath, targetPath, sourceLabel] of contracts) {
  const schema = JSON.parse(await readFile(new URL(specPath, import.meta.url), 'utf8'))
  const generated = generateRealtime(schema, sourceLabel)
  const checkedIn = await readFile(new URL(targetPath, import.meta.url), 'utf8')
  if (generated !== checkedIn) {
    console.error(`${sourceLabel} types are stale. Run cd web && npm run gen:gameplay.`)
    process.exitCode = 1
  } else {
    console.log(`Gameplay types match ${sourceLabel}.`)
  }
}
