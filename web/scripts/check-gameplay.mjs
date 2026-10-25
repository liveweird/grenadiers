import { readFile } from 'node:fs/promises'
import { discoverGameplayContracts } from './gameplay-contract-inventory.mjs'
import { generateRealtime } from './realtime-generator.mjs'

for (const { spec, target, sourceLabel } of await discoverGameplayContracts()) {
  const schema = JSON.parse(await readFile(spec, 'utf8'))
  const generated = generateRealtime(schema, sourceLabel)
  const checkedIn = await readFile(target, 'utf8')
  if (generated !== checkedIn) {
    console.error(`${sourceLabel} types are stale. Run cd web && npm run gen:gameplay.`)
    process.exitCode = 1
  } else {
    console.log(`Gameplay types match ${sourceLabel}.`)
  }
}
