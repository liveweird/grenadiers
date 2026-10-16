import { readFile } from 'node:fs/promises'
import openapiTS, { astToString, COMMENT_HEADER } from 'openapi-typescript'

const spec = new URL('../../api/openapi.json', import.meta.url)
const target = new URL('../src/api/schema.ts', import.meta.url)
const generated = COMMENT_HEADER + astToString(await openapiTS(spec))

if (generated !== await readFile(target, 'utf8')) {
  console.error('API types are stale. Run cd web && npm run gen:api.')
  process.exitCode = 1
} else {
  console.log('API types match api/openapi.json.')
}
