import Ajv, { type ErrorObject, type ValidateFunction } from 'ajv'
import { describe, expect, it } from 'vitest'
import gameplayV11Schema from '../../../../api/gameplay-v11.schema.json'
import gameplayV12Schema from '../../../../api/gameplay-v12.schema.json'
import gameplayV13Schema from '../../../../api/gameplay-v13.schema.json'
import realtimeSchema from '../../../../api/realtime.schema.json'
import {
  admitted as liveAdmitted, room as liveRoom, roomId, snapshot as liveSnapshot,
} from '../../game/client/testSupport'
import {
  admitted as archivedAdmitted, room as archivedRoom, snapshot as archivedSnapshot,
} from '../../game/validation/archived-v11-fixtures'
import generatedGameplayV11 from './gameplay-v11.mjs'
import generatedGameplayV12 from './gameplay-v12.mjs'
import generatedGameplayV13 from './gameplay-v13.mjs'
import generatedRealtime from './realtime.mjs'

type GeneratedValidator = ((value: unknown) => boolean) & { errors?: ErrorObject[] | null }
type SnapshotFixture = {
  room: { roomCode: string; revision: number; settings: { enabledItems: string[] } }
}
type FixtureSet = {
  admitted: () => unknown
  room: (revision: number) => unknown
  snapshot: (revision: number) => SnapshotFixture
}

function originalGameplayValidator(schema: typeof gameplayV11Schema): ValidateFunction {
  const ajv = new Ajv({ allErrors: true }).addSchema(schema)
  const reference = `${schema.$id}#/$defs/GameServerMessage`
  const validate = ajv.getSchema(reference)
  if (!validate) throw new Error(`AJV did not resolve ${reference}`)
  return validate
}

function gameplayVariants(version: 11 | 12 | 13, fixtures: FixtureSet) {
  return [
    ['authenticated', { type: 'authenticated', protocolVersion: version }],
    ['incompatibleProtocol', { type: 'incompatibleProtocol', supportedVersion: version }],
    ['admitted', fixtures.admitted()],
    ['roomState', { type: 'roomState', room: fixtures.room(2) }],
    ['snapshot', fixtures.snapshot(2)],
    ['leftRoom', { type: 'leftRoom', roomId, sequence: 1 }],
    ['seatReplaced', { type: 'seatReplaced', roomId }],
    ['error', { type: 'error', code: 'invalidMessage', sequence: null }],
  ] as const
}

const gameplayRoutes = [
  {
    version: 11, schema: gameplayV11Schema, generated: generatedGameplayV11,
    fixtures: { admitted: archivedAdmitted, room: archivedRoom, snapshot: archivedSnapshot },
  },
  {
    version: 12, schema: gameplayV12Schema, generated: generatedGameplayV12,
    fixtures: { admitted: liveAdmitted, room: liveRoom, snapshot: liveSnapshot },
  },
  {
    version: 13, schema: gameplayV13Schema, generated: generatedGameplayV13,
    fixtures: { admitted: liveAdmitted, room: liveRoom, snapshot: liveSnapshot },
  },
] as const

const realtimeVariants = [
  ['welcome', { type: 'welcome', protocolVersion: 1, maxPlayers: 2 }],
  ['pong', { type: 'pong', sequence: 0 }],
  ['error', { type: 'error', code: 'invalidMessage', message: 'Invalid client message' }],
] as const

function gameplayBoundaryCases(version: 11 | 12 | 13, fixtures: FixtureSet): unknown[] {
  const missing = { type: 'authenticated' }
  const additional = { type: 'authenticated', protocolVersion: version, unexpected: true }
  const duplicate = fixtures.snapshot(2)
  duplicate.room.settings.enabledItems = ['heart', 'heart']
  const unicode = fixtures.snapshot(2)
  unicode.room.roomCode = 'ŻÓŁWIK🐢'
  const counter = fixtures.snapshot(2)
  counter.room.revision = 2_147_483_648
  const nested = fixtures.snapshot(2) as unknown as Record<string, unknown>
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
  generated: GeneratedValidator,
  original: ValidateFunction,
  cases: readonly unknown[],
): void {
  for (const value of cases) {
    expect(generated(value)).toBe(original(value))
    expect(errors(generated)).toEqual(errors(original))
  }
}

function expectValidParity(
  generated: GeneratedValidator,
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
  it.each(gameplayRoutes)('matches the original gameplay v$version compile route for all variants and malformed boundaries', (route) => {
    const original = originalGameplayValidator(route.schema)
    expectValidParity(route.generated, original, gameplayVariants(route.version, route.fixtures))
    expectParity(route.generated, original, gameplayBoundaryCases(route.version, route.fixtures))
    expect(route.generated({ type: 'nonsense' })).toBe(false)
    expect(route.generated.errors).toHaveLength(26)
  })

  it('matches the original realtime root compile for all variants and malformed boundaries', () => {
    const original = new Ajv({ allErrors: true }).compile(realtimeSchema)
    expectValidParity(generatedRealtime, original, realtimeVariants)
    expectParity(generatedRealtime, original, realtimeBoundaryCases())
  })

  it.each([
    ...gameplayRoutes.map((route) => [
      `gameplay v${route.version}`, route.generated, originalGameplayValidator(route.schema),
      { type: 'nonsense' }, gameplayVariants(route.version, route.fixtures)[0][1],
    ] as const),
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
