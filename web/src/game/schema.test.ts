import Ajv, { type ValidateFunction } from 'ajv'
import { describe, expect, it } from 'vitest'
import gameplaySchema from '../../../api/gameplay.schema.json'
import type { GameClientMessage, GameServerMessage } from './schema'

const token = 'a'.repeat(64)
const roomId = 'b'.repeat(32)
const roomCode = 'ABCDEFGH'
const maximumCounter = 2_147_483_647

const clients = [
  { type: 'authenticate', protocolVersion: 0, token },
  { type: 'createRoom', sequence: 1 },
  { type: 'joinRoom', sequence: maximumCounter, roomCode },
  { type: 'ready', sequence: 2, roomId, roundId: 1, ready: true },
  { type: 'movement', sequence: 3, roomId, roundId: 1, direction: null },
  { type: 'placeBomb', sequence: 4, roomId, roundId: 1, actionId: 2 },
  { type: 'leaveRoom', sequence: 5, roomId, roundId: 1 },
] satisfies readonly GameClientMessage[]

const room: Extract<GameServerMessage, { type: 'roomState' }>['room'] = {
  roomId,
  roomCode,
  roundId: 1,
  revision: maximumCounter,
  tick: 9_007_199_254_740_991,
  rulesVersion: 1,
  mapId: 'fixed_v1',
  seats: [
    { seat: 'one', occupied: true, ready: true },
    { seat: 'two', occupied: true, ready: false },
  ],
  phase: { type: 'waiting' },
}

const servers = [
  { type: 'authenticated', protocolVersion: 1 },
  { type: 'incompatibleProtocol', supportedVersion: 1 },
  { type: 'roomState', room, youSeat: 'one' },
  { type: 'leftRoom', roomId, sequence: 1 },
  { type: 'error', code: 'invalidMessage', sequence: null },
] satisfies readonly GameServerMessage[]

// @ts-expect-error direction is required even when stopping movement
const missingDirection: GameClientMessage = { type: 'movement', sequence: 1, roomId, roundId: 1 }

const wrongDirection: GameClientMessage = {
  type: 'movement', sequence: 1, roomId, roundId: 1,
  // @ts-expect-error only cardinal protocol directions are accepted
  direction: 'up',
}

const wrongVersion: GameServerMessage = {
  type: 'incompatibleProtocol',
  // @ts-expect-error the advertised supported gameplay version is fixed at one
  supportedVersion: 2,
}

// @ts-expect-error error sequence is required even when it is null
const missingErrorSequence: GameServerMessage = { type: 'error', code: 'invalidMessage' }

const compileTimeExamples = [
  missingDirection,
  wrongDirection,
  wrongVersion,
  missingErrorSequence,
]

const ajv = new Ajv({ allErrors: true }).addSchema(gameplaySchema)

function schemaValidator<T>(definition: string): ValidateFunction<T> {
  const reference = `${gameplaySchema.$id}#/$defs/${definition}`
  const validator = ajv.getSchema<T>(reference)
  if (!validator) throw new Error(`AJV did not resolve ${reference}`)
  return validator
}

const validateClient = schemaValidator<GameClientMessage>('GameClientMessage')
const validateServer = schemaValidator<GameServerMessage>('GameServerMessage')

function valid(validator: ValidateFunction, value: unknown): boolean {
  return validator(value)
}

function without(value: object, property: string): Record<string, unknown> {
  const copy = structuredClone(value) as Record<string, unknown>
  delete copy[property]
  return copy
}

function withRoom(overrides: Record<string, unknown>): Record<string, unknown> {
  return { ...room, ...overrides }
}

describe('gameplay client schema', () => {
  it('keeps compile-time negative examples in the checked module', () => {
    expect(compileTimeExamples).toHaveLength(4)
  })

  it('accepts every command and requires every declared field', () => {
    for (const message of clients) {
      expect(valid(validateClient, message), JSON.stringify(validateClient.errors)).toBe(true)
      for (const property of Object.keys(message)) {
        expect(valid(validateClient, without(message, property))).toBe(false)
      }
      expect(valid(validateClient, { ...message, unexpected: true })).toBe(false)
      expect(valid(validateClient, { ...message, type: 'unknown' })).toBe(false)
      expect(valid(validateClient, { ...message, type: null })).toBe(false)
      const record = message as unknown as Record<string, unknown>
      for (const property of ['sequence', 'roundId', 'actionId']) {
        if (!Object.hasOwn(record, property)) continue
        expect(valid(validateClient, { ...message, [property]: 1 })).toBe(true)
        expect(valid(validateClient, { ...message, [property]: maximumCounter })).toBe(true)
        for (const value of [0, maximumCounter + 1, 1.5, '1', null]) {
          expect(valid(validateClient, { ...message, [property]: value })).toBe(false)
        }
      }
      if (Object.hasOwn(record, 'roomId')) {
        expect(valid(validateClient, { ...message, roomId: 'b'.repeat(31) })).toBe(false)
        expect(valid(validateClient, { ...message, roomId: 'B'.repeat(32) })).toBe(false)
      }
    }
  })

  it('enforces canonical strings, bounded integers, and required nullable direction', () => {
    for (const protocolVersion of [0, 2, 65_535]) {
      expect(valid(validateClient, { type: 'authenticate', protocolVersion, token })).toBe(true)
    }
    for (const numericJson of ['1.0', '1e0']) {
      const message: unknown = JSON.parse(`{"type":"authenticate","protocolVersion":${numericJson},"token":"${token}"}`)
      expect(valid(validateClient, message)).toBe(true)
    }
    for (const protocolVersion of [-1, 65_536, 1.5, '1', null]) {
      expect(valid(validateClient, { type: 'authenticate', protocolVersion, token })).toBe(false)
    }
    for (const badToken of ['a'.repeat(63), 'a'.repeat(65), 'A'.repeat(64), null]) {
      expect(valid(validateClient, { type: 'authenticate', protocolVersion: 1, token: badToken })).toBe(false)
    }
    for (const sequence of [1, maximumCounter]) {
      expect(valid(validateClient, { type: 'createRoom', sequence })).toBe(true)
    }
    for (const sequence of [0, maximumCounter + 1, 1.5, '1', null]) {
      expect(valid(validateClient, { type: 'createRoom', sequence })).toBe(false)
    }
    for (const badRoomCode of ['ABCDEFGI', 'abcdefgh', 'ABCDEFG', 'ABCDEFGH2', null]) {
      expect(valid(validateClient, { type: 'joinRoom', sequence: 1, roomCode: badRoomCode })).toBe(false)
    }
    expect(valid(validateClient, clients[4])).toBe(true)
    expect(valid(validateClient, without(clients[4], 'direction'))).toBe(false)
    expect(valid(validateClient, { ...clients[4], direction: 'up' })).toBe(false)
    expect(valid(validateClient, { ...clients[3], ready: 'true' })).toBe(false)
  })

  it('keeps sequence, round, and action counters distinct and bounded', () => {
    expect(valid(validateClient, { ...clients[5], sequence: 7, roundId: 8, actionId: 9 })).toBe(true)
    for (const property of ['sequence', 'roundId', 'actionId']) {
      expect(valid(validateClient, without(clients[5], property))).toBe(false)
      expect(valid(validateClient, { ...clients[5], [property]: 0 })).toBe(false)
      expect(valid(validateClient, { ...clients[5], [property]: maximumCounter + 1 })).toBe(false)
    }
    expect(valid(validateClient, { ...clients[5], roomId: 'b'.repeat(31) })).toBe(false)
    expect(valid(validateClient, { ...clients[5], roomId: 'B'.repeat(32) })).toBe(false)
  })
})

describe('gameplay server schema', () => {
  it('accepts every lifecycle response and rejects missing, extra, and mistyped top-level fields', () => {
    for (const message of servers) {
      expect(valid(validateServer, message), JSON.stringify(validateServer.errors)).toBe(true)
      for (const property of Object.keys(message)) {
        expect(valid(validateServer, without(message, property))).toBe(false)
      }
      expect(valid(validateServer, { ...message, unexpected: true })).toBe(false)
      expect(valid(validateServer, { ...message, type: 'unknown' })).toBe(false)
    }
    expect(valid(validateServer, { type: 'authenticated', protocolVersion: 2 })).toBe(false)
    expect(valid(validateServer, { type: 'incompatibleProtocol', supportedVersion: 2 })).toBe(false)
  })

  it('validates every room phase, both draw reasons, and interruption without a result', () => {
    const phases = [
      { type: 'waiting' },
      { type: 'countdown', endsAt: 0 },
      { type: 'running' },
      { type: 'result', endedAt: 3, result: { type: 'winner', seat: 'two' } },
      { type: 'result', endedAt: 4, result: { type: 'draw', reason: 'mutualElimination' } },
      { type: 'result', endedAt: 5, result: { type: 'draw', reason: 'timeout' } },
      { type: 'interrupted', endedAt: 6, reason: 'participantLeft' },
    ]
    for (const phase of phases) {
      const message = { type: 'roomState', room: withRoom({ phase }), youSeat: 'two' }
      expect(valid(validateServer, message)).toBe(true)
      for (const property of Object.keys(phase)) {
        expect(valid(validateServer, { ...message, room: withRoom({ phase: without(phase, property) }) })).toBe(false)
      }
      expect(valid(validateServer, { ...message, room: withRoom({ phase: { ...phase, extra: true } }) })).toBe(false)
    }
    expect(valid(validateServer, {
      type: 'roomState', room: withRoom({ phase: { ...phases[6], result: { type: 'winner', seat: 'one' } } }),
      youSeat: 'one',
    })).toBe(false)
  })

  it('rejects missing and extra nested fields, invalid bounds, and invalid room collections', () => {
    const roomState = { type: 'roomState', room, youSeat: 'one' }
    expect(valid(validateServer, { ...roomState, room: without(room, 'tick') })).toBe(false)
    expect(valid(validateServer, { ...roomState, room: withRoom({ extra: true }) })).toBe(false)
    expect(valid(validateServer, { ...roomState, room: withRoom({ tick: 9_007_199_254_740_992 }) })).toBe(false)
    expect(valid(validateServer, { ...roomState, room: withRoom({ roundId: 0 }) })).toBe(false)
    expect(valid(validateServer, { ...roomState, room: withRoom({ seats: [room.seats[0]] }) })).toBe(false)
    expect(valid(validateServer, {
      ...roomState,
      room: withRoom({ seats: [...room.seats, { seat: 'one', occupied: false, ready: false }] }),
    })).toBe(false)
    expect(valid(validateServer, {
      ...roomState,
      room: withRoom({ seats: [{ seat: 'one', occupied: true }, room.seats[1]] }),
    })).toBe(false)
    expect(valid(validateServer, {
      ...roomState,
      room: withRoom({ seats: [{ seat: 'one', occupied: null, ready: true }, room.seats[1]] }),
    })).toBe(false)
    expect(valid(validateServer, { type: 'error', code: 'invalidMessage' })).toBe(false)
    expect(valid(validateServer, { type: 'error', code: 'unknown', sequence: null })).toBe(false)
  })

  it('accepts every fixed error and interruption code', () => {
    const errors = gameplaySchema.$defs.ErrorCode.enum
    const interruptions = gameplaySchema.$defs.InterruptReason.enum
    for (const code of errors) expect(valid(validateServer, { type: 'error', code, sequence: 1 })).toBe(true)
    for (const reason of interruptions) {
      const phase = { type: 'interrupted', endedAt: 0, reason }
      expect(valid(validateServer, { type: 'roomState', room: withRoom({ phase }), youSeat: 'one' })).toBe(true)
    }
  })
})
