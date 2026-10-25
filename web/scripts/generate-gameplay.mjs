import { readFile, writeFile } from 'node:fs/promises'
import { discoverGameplayContracts } from './gameplay-contract-inventory.mjs'
import { generateGameplay } from './gameplay-generator.mjs'

for (const { version, spec, target, sourceLabel } of await discoverGameplayContracts()) {
  const schema = JSON.parse(await readFile(spec, 'utf8'))
  await writeFile(target, generateGameplay(schema, sourceLabel, version))
}
