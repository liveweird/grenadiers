import Ajv, { type ValidateFunction } from 'ajv'
import { describe, expect, it } from 'vitest'
import fixtures from '../../../../api/fixtures/gameplay-expanded-client-conformance.json'
import gameplayV10Schema from '../../../../api/gameplay-v10.schema.json'
import gameplayV11Schema from '../../../../api/gameplay-v11.schema.json'
import gameplayV8Schema from '../../../../api/gameplay-v8.schema.json'
import gameplayV9Schema from '../../../../api/gameplay-v9.schema.json'

const schemas = [gameplayV8Schema, gameplayV9Schema, gameplayV10Schema, gameplayV11Schema]
const ajv = new Ajv({ allErrors: true })
for (const schema of schemas) ajv.addSchema(schema)
const validators = schemas.map((schema) => {
  const reference = `${schema.$id}#/$defs/GameClientMessage`
  const validate = ajv.getSchema(reference)
  if (!validate) throw new Error(`AJV did not resolve ${reference}`)
  return validate
})

function record(value: unknown): Record<string, unknown> {
  return value as Record<string, unknown>
}

function incompatibleValue(value: unknown): unknown {
  if (value === null) return false
  if (Array.isArray(value)) return {}
  switch (typeof value) {
    case 'boolean': return 0
    case 'number': return 'wrong-type'
    case 'string': return 0
    case 'object': return []
    default: return null
  }
}

function expectRejected(caseId: string, value: unknown, validate: ValidateFunction): void {
  expect(validate(value), caseId).toBe(false)
}

describe('expanded gameplay client conformance', () => {
  it('derives coverage from every public client variant and manual action', () => {
    const variants = new Set(fixtures.validCases.map((fixture) => fixture.variant))
    expect([...variants].sort()).toEqual([
      'authenticate',
      'createRoom',
      'joinRoom',
      'leaveRoom',
      'manualAction',
      'movement',
      'ready',
      'rematch',
      'resumeRoom',
      'updateSettings',
    ])
    const manualActions = fixtures.validCases
      .filter((fixture) => fixture.variant === 'manualAction')
      .map((fixture) => record(fixture.value).action)
    expect(new Set(manualActions)).toEqual(new Set(['bomb', 'equipment']))
  })

  it('agrees on every shared valid and invalid case across v8 through v11', () => {
    for (const fixture of fixtures.validCases) {
      for (const validate of validators) {
        expect(validate(fixture.value), fixture.caseId).toBe(true)
      }
    }
    for (const fixture of fixtures.invalidCases) {
      for (const validate of validators) {
        expectRejected(fixture.caseId, fixture.value, validate)
      }
    }
  })

  it('rejects every single required-field removal', () => {
    for (const fixture of fixtures.validCases) {
      for (const field of Object.keys(fixture.value)) {
        const changed = structuredClone(record(fixture.value))
        delete changed[field]
        for (const validate of validators) expectRejected(fixture.caseId, changed, validate)
      }
      const originalSettings = record(fixture.value).settings
      if (originalSettings) {
        for (const field of Object.keys(record(originalSettings))) {
          const changed = structuredClone(record(fixture.value))
          delete record(changed.settings)[field]
          for (const validate of validators) expectRejected(fixture.caseId, changed, validate)
        }
      }
    }
  })

  it('rejects unknown fields and every systematic wrong-field type', () => {
    for (const fixture of fixtures.validCases) {
      const extra = structuredClone(record(fixture.value))
      extra.unexpected = true
      for (const validate of validators) expectRejected(fixture.caseId, extra, validate)

      for (const [field, original] of Object.entries(fixture.value)) {
        const changed = structuredClone(record(fixture.value))
        changed[field] = incompatibleValue(original)
        for (const validate of validators) expectRejected(fixture.caseId, changed, validate)
      }

      const originalSettings = record(fixture.value).settings
      if (originalSettings) {
        const extraSetting = structuredClone(record(fixture.value))
        record(extraSetting.settings).unexpected = true
        for (const validate of validators) expectRejected(fixture.caseId, extraSetting, validate)
        for (const [field, original] of Object.entries(record(originalSettings))) {
          const changed = structuredClone(record(fixture.value))
          record(changed.settings)[field] = incompatibleValue(original)
          for (const validate of validators) expectRejected(fixture.caseId, changed, validate)
        }
      }
    }
  })

  it('treats legal order, whitespace, escapes, and integral number forms identically', () => {
    const room = '0123456789abcdef0123456789abcdef'
    const canonical = `{"type":"movement","sequence":1,"roomId":"${room}","roundId":2,"direction":"west"}`
    const representations = [
      `{"direction":"west","roundId":2,"roomId":"${room}","sequence":1,"type":"movement"}`,
      ` { "type" : "movement", "sequence" : 1, "roomId" : "${room}", "roundId" : 2, "direction" : "west" }\n`,
      `{"\\u0074ype":"\\u006dovement","sequence":1,"roomId":"${room}","roundId":2,"direction":"\\u0077est"}`,
      `{"type":"movement","sequence":1.0,"roomId":"${room}","roundId":2e0,"direction":"west"}`,
    ]
    for (const raw of representations) {
      const value: unknown = JSON.parse(raw)
      expect(value).toEqual(JSON.parse(canonical))
      for (const validate of validators) expect(validate(value), 'legal-representation').toBe(true)
    }
  })
})
