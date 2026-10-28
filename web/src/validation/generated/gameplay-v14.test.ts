import Ajv, { type ErrorObject } from 'ajv'
import { describe, expect, it } from 'vitest'
import gameplayV14Schema from '../../../../api/gameplay-v14.schema.json'
import generated from './gameplay-v14.mjs'
import type { GameServerMessage } from '../../game/v14/schema'

type Validator = ((value: unknown) => boolean) & { errors?: ErrorObject[] | null }

function errors(validator: Validator): ErrorObject[] | null {
  return structuredClone(validator.errors ?? null)
}

describe('gameplay v14 standalone validator parity', () => {
  it('matches the canonical schema for valid and malformed server variants', () => {
    const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV14Schema)
    const original = ajv.getSchema(`${gameplayV14Schema.$id}#/$defs/GameServerMessage`) as Validator
    expect(original).toBeTypeOf('function')
    const validCases: GameServerMessage[] = [
      { type: 'authenticated', protocolVersion: 14 },
      { type: 'incompatibleProtocol', supportedVersion: 14 },
      { type: 'leftRoom', roomId: '0123456789abcdef0123456789abcdef', sequence: 1 },
      { type: 'seatReplaced', roomId: '0123456789abcdef0123456789abcdef' },
      { type: 'error', code: 'invalidMessage', sequence: null },
    ]
    const cases = [
      ...validCases,
      { type: 'authenticated' },
      { type: 'authenticated', protocolVersion: 13 },
      { type: 'nonsense' },
    ]
    for (const value of cases) {
      expect(generated(value)).toBe(original(value))
      expect(errors(generated)).toEqual(errors(original))
    }
  })
})
