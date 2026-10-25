import Ajv, { type ErrorObject, type ValidateFunction } from 'ajv'
import { describe, expect, it } from 'vitest'
import gameplaySchema from '../../../../api/gameplay-v11.schema.json'
import realtimeSchema from '../../../../api/realtime.schema.json'
import { admitted, room, roomId, snapshot } from '../../game/client/testSupport'
import generatedGameplay from './gameplay-v11.mjs'
import generatedRealtime from './realtime.mjs'

function originalGameplayValidator(): ValidateFunction {
  const ajv = new Ajv({ allErrors: true }).addSchema(gameplaySchema)
  const reference = `${gameplaySchema.$id}#/$defs/GameServerMessage`
  const validate = ajv.getSchema(reference)
  if (!validate) throw new Error(`AJV did not resolve ${reference}`)
  return validate
}

const gameplayVariants = [
  ['authenticated', { type: 'authenticated', protocolVersion: 11 }],
  ['incompatibleProtocol', { type: 'incompatibleProtocol', supportedVersion: 11 }],
  ['admitted', admitted()],
  ['roomState', { type: 'roomState', room: room(2) }],
  ['snapshot', snapshot(2)],
  ['leftRoom', { type: 'leftRoom', roomId, sequence: 1 }],
  ['seatReplaced', { type: 'seatReplaced', roomId }],
  ['error', { type: 'error', code: 'invalidMessage', sequence: null }],
] as const

const realtimeVariants = [
  ['welcome', { type: 'welcome', protocolVersion: 1, maxPlayers: 2 }],
  ['pong', { type: 'pong', sequence: 0 }],
  ['error', { type: 'error', code: 'invalidMessage', message: 'Invalid client message' }],
] as const

function gameplayBoundaryCases(): unknown[] {
  const missing = { type: 'authenticated' }
  const additional = { type: 'authenticated', protocolVersion: 11, unexpected: true }
  const duplicate = snapshot(2)
  duplicate.room.settings.enabledItems = ['heart', 'heart']
  const unicode = snapshot(2)
  unicode.room.roomCode = 'ŻÓŁWIK🐢'
  const counter = snapshot(2)
  counter.room.revision = 2_147_483_648
  const nested = snapshot(2) as unknown as Record<string, unknown>
  const state = nested.state as { actors: Array<Record<string, unknown>> }
  state.actors[0].position = { x: '512', y: 512 }
  return [{ type: 'nonsense' }, missing, additional, duplicate, unicode, counter, nested]
}

function realtimeBoundaryCases(): unknown[] {
  return [
    { type: 'nonsense' },
    { type: 'pong' },
    { type: 'pong', sequence: 1, unexpected: true },
    { type: 'pong', sequence: 2_147_483_648 },
    { type: 'error', code: 'invalidMessage', message: 'Nieprawidłowa wiadomość 🐢' },
    { type: 'welcome', protocolVersion: 1, maxPlayers: { nested: 2 } },
  ]
}

function errors(validate: { errors?: ErrorObject[] | null }): ErrorObject[] | null {
  return structuredClone(validate.errors ?? null)
}

function expectParity(
  generated: ((value: unknown) => boolean) & { errors?: ErrorObject[] | null },
  original: ValidateFunction,
  cases: readonly unknown[],
): void {
  for (const value of cases) {
    expect(generated(value)).toBe(original(value))
    expect(errors(generated)).toEqual(errors(original))
  }
}

function expectValidParity(
  generated: ((value: unknown) => boolean) & { errors?: ErrorObject[] | null },
  original: ValidateFunction,
  cases: readonly (readonly [string, unknown])[],
): void {
  for (const [label, value] of cases) {
    expect(original(value), `original compiler rejected valid ${label}`).toBe(true)
    expect(generated(value), `generated validator rejected valid ${label}`).toBe(true)
    expect(errors(generated)).toEqual(errors(original))
  }
}

describe('standalone validator parity', () => {
  it('matches the original gameplay compile route for all variants and malformed boundaries', () => {
    const original = originalGameplayValidator()
    expectValidParity(generatedGameplay, original, gameplayVariants)
    expectParity(generatedGameplay, original, gameplayBoundaryCases())
    expect(generatedGameplay({ type: 'nonsense' })).toBe(false)
    expect(generatedGameplay.errors).toHaveLength(26)
  })

  it('matches the original realtime root compile for all variants and malformed boundaries', () => {
    const original = new Ajv({ allErrors: true }).compile(realtimeSchema)
    expectValidParity(generatedRealtime, original, realtimeVariants)
    expectParity(generatedRealtime, original, realtimeBoundaryCases())
  })

  it.each([
    ['gameplay', generatedGameplay, originalGameplayValidator(), { type: 'nonsense' }, gameplayVariants[0][1]],
    ['realtime', generatedRealtime, new Ajv({ allErrors: true }).compile(realtimeSchema),
      { type: 'nonsense' }, realtimeVariants[0][1]],
  ] as const)('retains old %s errors across invalid-valid-invalid transitions', (_name, generated, original, invalid, valid) => {
    expect(generated(invalid)).toBe(false)
    expect(original(invalid)).toBe(false)
    const generatedRetained = generated.errors
    const originalRetained = original.errors
    expect(errors(generated)).toEqual(errors(original))

    expect(generated(valid)).toBe(true)
    expect(original(valid)).toBe(true)
    expect(generated.errors).toBeNull()
    expect(original.errors).toBeNull()
    expect(generatedRetained).toEqual(originalRetained)

    expect(generated(invalid)).toBe(false)
    expect(original(invalid)).toBe(false)
    expect(errors(generated)).toEqual(errors(original))
    expect(generated.errors).not.toBe(generatedRetained)
  })
})
