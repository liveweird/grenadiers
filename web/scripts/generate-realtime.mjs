import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { discoverRealtimeContracts } from './realtime-contract-inventory.mjs'
import { generateRealtime } from './realtime-generator.mjs'

for (const { spec, target, sourceLabel } of await discoverRealtimeContracts()) {
  const schema = JSON.parse(await readFile(spec, 'utf8'))
  await mkdir(dirname(fileURLToPath(target)), { recursive: true })
  await writeFile(target, generateRealtime(schema, sourceLabel))
}
