import { readFile } from 'node:fs/promises'
import { discoverRealtimeContracts } from './realtime-contract-inventory.mjs'
import { generateRealtime } from './realtime-generator.mjs'

for (const { spec, target, sourceLabel } of await discoverRealtimeContracts()) {
  const schema = JSON.parse(await readFile(spec, 'utf8'))
  if (generateRealtime(schema, sourceLabel) !== await readFile(target, 'utf8')) {
    console.error(`${sourceLabel} types are stale. Run cd web && npm run gen:realtime.`)
    process.exitCode = 1
  } else {
    console.log(`Realtime types match ${sourceLabel}.`)
  }
}
