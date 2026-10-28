import { readFile, readdir } from 'node:fs/promises'
import { pathToFileURL } from 'node:url'

const scriptsDirectory = pathToFileURL(`${import.meta.dirname}/`)

const contracts = Object.freeze([
  {
    filename: 'realtime.schema.json',
    version: 2,
    target: 'schema.ts',
  },
  {
    filename: 'realtime-v1.schema.json',
    version: 1,
    target: 'v1/schema.ts',
  },
])

export async function discoverRealtimeContracts({
  apiDirectory = new URL('../../api/', scriptsDirectory),
  targetDirectory = new URL('../src/realtime/', scriptsDirectory),
} = {}) {
  const entries = (await readdir(apiDirectory, { withFileTypes: true }))
    .filter((entry) => /^realtime(?:-v\d+)?\.schema\.json$/.test(entry.name))
  const filenames = new Set(entries.map((entry) => {
    if (!entry.isFile()) throw new Error(`Realtime contract must be a regular file: ${entry.name}`)
    return entry.name
  }))
  const expected = new Set(contracts.map(({ filename }) => filename))
  const unexpected = [...filenames].filter((filename) => !expected.has(filename))
  const missing = [...expected].filter((filename) => !filenames.has(filename))
  if (unexpected.length > 0) throw new Error(`Unsupported realtime contracts: ${unexpected.join(', ')}`)
  if (missing.length > 0) throw new Error(`Missing realtime contracts: ${missing.join(', ')}`)

  return Promise.all(contracts.map(async ({ filename, version, target }) => {
    const spec = new URL(filename, apiDirectory)
    let schema
    try {
      schema = JSON.parse(await readFile(spec, 'utf8'))
    } catch (cause) {
      throw new Error(`Cannot read realtime contract ${filename}`, { cause })
    }
    const expectedId = `https://grenadiers.local/schemas/realtime-v${version}.json`
    const welcome = schema?.$defs?.Welcome?.properties
    if (schema?.$id !== expectedId
      || welcome?.protocolVersion?.const !== version
      || welcome?.maxPlayers?.const !== (version === 1 ? 2 : 4)) {
      throw new Error(`Realtime contract identity mismatch: ${filename}`)
    }
    return {
      version,
      spec,
      target: new URL(target, targetDirectory),
      sourceLabel: `api/${filename}`,
    }
  }))
}
