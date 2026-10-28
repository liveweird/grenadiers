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

test('a source label changes only the generated provenance header', async () => {
  const schema = JSON.parse(await readFile(schemaUrl, 'utf8'))
  const diagnostic = generateRealtime(schema)
  const gameplay = generateRealtime(schema, 'api/gameplay.schema.json')
  assert.match(gameplay, /^\/\/ This file was auto-generated from api\/gameplay\.schema\.json\. Do not edit\.\n/)
  assert.equal(gameplay.split('\n').slice(2).join('\n'), diagnostic.split('\n').slice(2).join('\n'))
})

test('nested bounded strings and arrays through references and unions retain their TypeScript constraints', async () => {
  const schema = {
    $defs: {
      Identifier: { type: 'string', minLength: 0, maxLength: 64, pattern: '^[\\p{L}\\p{N}_-]+$' },
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
          values: {
            type: 'array',
            minItems: 0,
            maxItems: 10,
            uniqueItems: true,
            items: { oneOf: [{ type: 'string', maxLength: 20 }, { type: 'number' }] },
          },
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

test('invalid string and array constraints fail closed', () => {
  for (const [keyword, value] of [
    ['minLength', -1],
    ['minLength', 1.5],
    ['minLength', '1'],
    ['maxLength', Number.MAX_SAFE_INTEGER + 1],
  ]) {
    assert.throws(
      () => generateRealtime({ $defs: { Value: { type: 'string', [keyword]: value } } }),
      new RegExp(`${keyword} must be a non-negative safe integer`),
    )
  }
  for (const [keyword, value] of [
    ['minItems', -1],
    ['minItems', 1.5],
    ['minItems', '1'],
    ['maxItems', Number.MAX_SAFE_INTEGER + 1],
  ]) {
    assert.throws(
      () => generateRealtime({ $defs: { Value: { type: 'array', items: { type: 'string' }, [keyword]: value } } }),
      new RegExp(`${keyword} must be a non-negative safe integer`),
    )
  }
  assert.throws(
    () => generateRealtime({ $defs: { Value: { type: 'string', minLength: 2, maxLength: 1 } } }),
    /minLength must not exceed maxLength/,
  )
  assert.throws(
    () =>
      generateRealtime({
        $defs: { Value: { type: 'array', items: { type: 'string' }, minItems: 2, maxItems: 1 } },
      }),
    /minItems must not exceed maxItems/,
  )
  assert.throws(
    () => generateRealtime({ $defs: { Value: { type: 'string', pattern: 1 } } }),
    /pattern must be a string/,
  )
  assert.throws(
    () => generateRealtime({ $defs: { Value: { type: 'string', pattern: '\\a' } } }),
    /pattern must be a valid ECMAScript Unicode regular expression/,
  )
  assert.throws(
    () => generateRealtime({ $defs: { Value: { type: 'array', items: { type: 'string' }, uniqueItems: 1 } } }),
    /uniqueItems must be a boolean/,
  )
})

test('constraint keywords are rejected on the wrong schema shape', () => {
  for (const [type, constraint] of [
    ['number', { minLength: 1 }],
    ['string', { minItems: 1 }],
    ['boolean', { pattern: 'true' }],
    ['object', { maxLength: 1, additionalProperties: false }],
  ]) {
    const keyword = Object.keys(constraint)[0]
    assert.throws(
      () => generateRealtime({ $defs: { Value: { type, ...constraint } } }),
      new RegExp(`keyword "${keyword}" is not supported for this schema shape`),
    )
  }
  assert.throws(
    () => generateRealtime({ $defs: { Value: { $ref: '#/$defs/Target', minItems: 1 }, Target: { type: 'string' } } }),
    /keyword "minItems" is not supported for this schema shape/,
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

test('scalar array literals generate tuple types while object members stay unsupported', () => {
  const generated = generateRealtime({
    $defs: { Lineup: { enum: [['human', 'cpu'], ['human', 'human', 'cpu']] } },
  })
  assert.match(generated, /\["human", "cpu"\] \| \["human", "human", "cpu"\]/)
  assert.throws(
    () => generateRealtime({ $defs: { Bad: { const: ['human', { type: 'cpu' }] } } }),
    /array literals may contain only scalar literals/,
  )
})
