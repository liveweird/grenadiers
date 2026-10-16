import { readFile, writeFile } from 'node:fs/promises'
import { generateRealtime } from './realtime-generator.mjs'

const spec = new URL('../../api/realtime.schema.json', import.meta.url)
const target = new URL('../src/realtime/schema.ts', import.meta.url)
const schema = JSON.parse(await readFile(spec, 'utf8'))
await writeFile(target, generateRealtime(schema))
