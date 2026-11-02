import Ajv, { type ErrorObject, type ValidateFunction } from 'ajv'
import { describe, expect, it } from 'vitest'
import clientFixtures from '../../../../api/fixtures/gameplay-v20-client-conformance.json'
import serverFixtures from '../../../../api/fixtures/gameplay-v20-server-conformance.json'
import gameplayV20Schema from '../../../../api/gameplay-v20.schema.json'
import validateServer from '../../validation/generated/gameplay-v20.mjs'
import { validatorOptions } from '../../validation/validatorOptions'
import { validateGameplayV20ServerMessage } from './validation'

type Validator = ((value: unknown) => boolean) & { errors?: ErrorObject[] | null }
const errors = (validator: Validator): ErrorObject[] | null => structuredClone(validator.errors ?? null)

describe('gameplay v20 contract validation', () => {
  it('accepts Bomb and rejects Equipment across the closed client corpus', () => {
    const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV20Schema)
    const validateClient = ajv.getSchema(`${gameplayV20Schema.$id}#/$defs/GameClientMessage`)
    for (const fixture of clientFixtures.validMessages) {
      expect(validateClient!(fixture.value), fixture.caseId).toBe(true)
    }
    for (const fixture of clientFixtures.invalidMessages) {
      expect(validateClient!(fixture.value), fixture.caseId).toBe(false)
    }
  })

  it('keeps the generated production validator in canonical Ajv parity', () => {
    const original = new Ajv(validatorOptions('gameplay-v20')).addSchema(gameplayV20Schema)
      .getSchema(`${gameplayV20Schema.$id}#/$defs/GameServerMessage`) as ValidateFunction
    for (const fixture of [...serverFixtures.validMessages, ...serverFixtures.invalidMessages]) {
      expect(validateServer(fixture.value), fixture.caseId).toBe(original(fixture.value))
      expect(errors(validateServer)).toEqual(errors(original))
    }
  })

  it('accepts the v20 corpus and rejects the old v19 server identity', () => {
    for (const fixture of serverFixtures.validMessages) {
      expect(validateGameplayV20ServerMessage(fixture.value), fixture.caseId)
        .toMatchObject({ valid: true })
    }
    expect(validateGameplayV20ServerMessage({ type: 'authenticated', protocolVersion: 19 }))
      .toMatchObject({ valid: false, source: 'schema' })
  })
})
