import { readFile, writeFile } from 'node:fs/promises'
import { discoverGameplayContracts } from './gameplay-contract-inventory.mjs'
import { generateRealtime } from './realtime-generator.mjs'

for (const { spec, target, sourceLabel } of await discoverGameplayContracts()) {
  const schema = JSON.parse(await readFile(spec, 'utf8'))
  await writeFile(target, generateRealtime(schema, sourceLabel))
}
