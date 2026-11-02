import Ajv, { type ErrorObject, type ValidateFunction } from 'ajv'
import { describe, expect, it } from 'vitest'
import clientFixtures from '../../../../api/fixtures/gameplay-v21-client-conformance.json'
import serverFixtures from '../../../../api/fixtures/gameplay-v21-server-conformance.json'
import gameplayV21Schema from '../../../../api/gameplay-v21.schema.json'
import validateServer from '../../validation/generated/gameplay-v21.mjs'
import { validatorOptions } from '../../validation/validatorOptions'
import { validateGameplayV21ServerMessage } from './validation'

type Validator = ((value: unknown) => boolean) & { errors?: ErrorObject[] | null }
const errors = (validator: Validator): ErrorObject[] | null => structuredClone(validator.errors ?? null)

describe('gameplay v21 contract validation', () => {
  it('accepts only integer bomb-capacity multipliers from one through ten', () => {
    const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV21Schema)
    const validateClient = ajv.getSchema(`${gameplayV21Schema.$id}#/$defs/GameClientMessage`)
    for (const fixture of clientFixtures.validMessages) {
      expect(validateClient!(fixture.value), fixture.caseId).toBe(true)
    }
    for (const fixture of clientFixtures.invalidMessages) {
      expect(validateClient!(fixture.value), fixture.caseId).toBe(false)
    }
  })

  it('keeps the generated production validator in canonical Ajv parity', () => {
    const original = new Ajv(validatorOptions('gameplay-v21')).addSchema(gameplayV21Schema)
      .getSchema(`${gameplayV21Schema.$id}#/$defs/GameServerMessage`) as ValidateFunction
    for (const fixture of [...serverFixtures.validMessages, ...serverFixtures.invalidMessages]) {
      expect(validateServer(fixture.value), fixture.caseId).toBe(original(fixture.value))
      expect(errors(validateServer)).toEqual(errors(original))
    }
    let schemaInvalid = 0
    for (const fixture of serverFixtures.invalidMessages) {
      if (validateServer(fixture.value)) continue
      schemaInvalid += 1
      expect(validateServer.errors?.length, fixture.caseId).toBeGreaterThan(0)
      for (const error of validateServer.errors ?? []) {
        expect(error.instancePath, fixture.caseId).toBeTypeOf('string')
        expect(error.schemaPath, fixture.caseId).toMatch(/^#\//)
        expect(error.keyword, fixture.caseId).not.toBe('')
        expect(error.message, fixture.caseId).toBeUndefined()
      }
    }
    expect(schemaInvalid).toBeGreaterThan(0)
  })

  it('accepts the v21 corpus and rejects the retired v20 server identity', () => {
    for (const fixture of serverFixtures.validMessages) {
      expect(validateGameplayV21ServerMessage(fixture.value), fixture.caseId)
        .toMatchObject({ valid: true })
    }
    expect(validateGameplayV21ServerMessage({ type: 'authenticated', protocolVersion: 20 }))
      .toMatchObject({ valid: false, source: 'schema' })
  })
})
