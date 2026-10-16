import { readFile } from 'node:fs/promises'
import { generateRealtime } from './realtime-generator.mjs'

const spec = new URL('../../api/realtime.schema.json', import.meta.url)
const target = new URL('../src/realtime/schema.ts', import.meta.url)
const schema = JSON.parse(await readFile(spec, 'utf8'))

if (generateRealtime(schema) !== await readFile(target, 'utf8')) {
  console.error('Realtime types are stale. Run cd web && npm run gen:realtime.')
  process.exitCode = 1
} else {
  console.log('Realtime types match api/realtime.schema.json.')
}
