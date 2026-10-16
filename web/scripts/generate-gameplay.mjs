import { readFile, writeFile } from 'node:fs/promises'
import { generateRealtime } from './realtime-generator.mjs'

const spec = new URL('../../api/gameplay.schema.json', import.meta.url)
const target = new URL('../src/game/schema.ts', import.meta.url)
const schema = JSON.parse(await readFile(spec, 'utf8'))
await writeFile(target, generateRealtime(schema, 'api/gameplay.schema.json'))
