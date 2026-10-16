import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { fileURLToPath, pathToFileURL, URL } from 'node:url'
import { test } from 'vitest'
import { generateRealtime } from './realtime-generator.mjs'

const scriptsUrl = pathToFileURL(`${import.meta.dirname}/`)
const schemaUrl = new URL('../../api/realtime.schema.json', scriptsUrl)
const generatedUrl = new URL('../src/realtime/schema.ts', scriptsUrl)
const tscUrl = new URL('../node_modules/typescript/bin/tsc', scriptsUrl)

test('current realtime contract remains byte-for-byte reproducible', async () => {
  const schema = JSON.parse(await readFile(schemaUrl, 'utf8'))
  assert.equal(generateRealtime(schema), await readFile(generatedUrl, 'utf8'))
})

test('nested objects and union arrays retain their TypeScript constraints', async () => {
  const schema = {
    $defs: {
      Identifier: { type: 'string' },
      Empty: { type: 'object', additionalProperties: false },
      Envelope: {
        type: 'object',
        additionalProperties: false,
        required: ['type', 'payload', 'values', 'empty'],
        properties: {
          type: { const: 'event' },
          payload: {
            type: 'object',
            additionalProperties: false,
            required: ['id'],
            properties: {
              id: { $ref: '#/$defs/Identifier' },
              state: { enum: ['ready', 'waiting'] },
            },
          },
          values: { type: 'array', items: { oneOf: [{ type: 'string' }, { type: 'number' }] } },
          empty: { type: 'object', additionalProperties: false, properties: {} },
          note: { type: 'string' },
        },
      },
    },
  }
  const generated = generateRealtime(schema)
  const usage = `
const valid: Envelope = {
  type: 'event',
  payload: { id: 'player-1' },
  values: ['one', 2],
  empty: {},
}
void valid

// @ts-expect-error a closed empty object cannot be a primitive
const wrongEmptyAlias: Empty = 42
void wrongEmptyAlias

const wrongNested: Envelope = {
  type: 'event',
  payload: {
    // @ts-expect-error nested id must remain a string
    id: 123,
  },
  values: [],
  empty: {},
}
void wrongNested

const wrongArray: Envelope = {
  type: 'event',
  payload: { id: 'player-2' },
  values: [
    // @ts-expect-error array accepts strings and numbers, not booleans
    true,
  ],
  // @ts-expect-error a closed empty object cannot contain string-keyed properties
  empty: { unexpected: true },
}
void wrongArray
`

  const directory = await mkdtemp(`${tmpdir()}/grenadiers-realtime-generator-`)
  const source = new URL('contract.ts', pathToFileURL(`${directory}/`))
  try {
    await writeFile(source, generated + usage)
    const result = spawnSync(
      process.execPath,
      [fileURLToPath(tscUrl), '--ignoreConfig', '--noEmit', '--strict', '--skipLibCheck', fileURLToPath(source)],
      { encoding: 'utf8' },
    )
    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`)
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
})

test('unsupported schema constructs fail instead of widening generated types', () => {
  assert.throws(
    () => generateRealtime({ $defs: { Event: { anyOf: [{ type: 'string' }, { type: 'number' }] } } }),
    /Unsupported realtime schema.*keyword "anyOf"/,
  )
  assert.throws(
    () =>
      generateRealtime({
        $defs: {
          Event: { type: 'object', properties: { type: { const: 'event' } } },
        },
      }),
    /only closed objects with additionalProperties: false are supported/,
  )
})

test('references must resolve to local definitions', () => {
  assert.throws(
    () => generateRealtime({ $defs: { Event: { $ref: '#/$defs/Missing' } } }),
    /\$ref "#\/\$defs\/Missing" does not exist/,
  )
  assert.throws(
    () => generateRealtime({ $defs: { Event: { $ref: 'https://example.com/event.json' } } }),
    /must point directly into #\/\$defs/,
  )
})

test('definition names must be valid non-reserved TypeScript names', () => {
  for (const name of ['Record', 'string', 'class', 'interface']) {
    assert.throws(
      () => generateRealtime({ $defs: { [name]: { type: 'string' } } }),
      new RegExp(`definition name "${name}" is reserved by TypeScript`),
    )
  }
})

test('unguarded alias cycles fail before emitting uncompilable TypeScript', () => {
  assert.throws(
    () => generateRealtime({ $defs: { Event: { $ref: '#/$defs/Event' } } }),
    /unguarded alias cycle: Event -> Event/,
  )
  assert.throws(
    () =>
      generateRealtime({
        $defs: {
          First: { $ref: '#/$defs/Second' },
          Second: { oneOf: [{ $ref: '#/$defs/First' }, { type: 'string' }] },
        },
      }),
    /unguarded alias cycle: First -> Second -> First/,
  )
})
