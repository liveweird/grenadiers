import Ajv, { type ValidateFunction } from 'ajv'
import { describe, expect, it } from 'vitest'
import gameplayV3Schema from '../../../../api/gameplay-v3.schema.json'
import gameplayV5Schema from '../../../../api/gameplay-v5.schema.json'
import type { GameClientMessage, GameServerMessage, GameState, RoomPhase, RoomView } from './schema'

const token = 'a'.repeat(64)
const roomId = 'b'.repeat(32)
const maximumCounter = 2_147_483_647
const maximumTick = 9_007_199_254_740_991

const clients = [
  { type: 'authenticate', protocolVersion: 5, token },
  { type: 'createRoom', sequence: 1 },
  { type: 'joinRoom', sequence: 2, roomCode: 'ABCDEFGH' },
  { type: 'resumeRoom', sequence: 3, roomId, resumeToken: 'c'.repeat(64) },
  { type: 'ready', sequence: 3, roomId, roundId: 1, ready: true },
  { type: 'movement', sequence: 4, roomId, roundId: 1, direction: null },
  { type: 'placeBomb', sequence: 5, roomId, roundId: 1, actionId: 1 },
  { type: 'leaveRoom', sequence: 6, roomId, roundId: 1 },
  { type: 'rematch', sequence: 7, roomId, roundId: 1, ready: false },
] satisfies readonly GameClientMessage[]

const completedRound = {
  roundId: 2,
  roundNumber: 1,
  endedAt: 180,
  result: { type: 'winner', seat: 'one' },
} as const

const room: RoomView = {
  roomId,
  roomCode: 'ABCDEFGH',
  roundId: 3,
  revision: maximumCounter,
  tick: maximumTick,
  rulesVersion: 2,
  mapId: 'generated_v1',
  roundNumber: 1,
  scores: { one: 1, two: 0 },
  lastRound: completedRound,
  seats: [
    { seat: 'one', occupied: true, ready: false, rematchReady: true, connection: { type: 'connected' } },
    { seat: 'two', occupied: true, ready: false, rematchReady: false, connection: { type: 'disconnected', expiresAt: maximumTick } },
  ],
  phase: { type: 'result', holdUntil: 360 },
}

const state: GameState = {
  runningUpdates: 0,
  remainingTicks: 10_800,
  arena: { width: 15, height: 13, terrain: Array.from({ length: 195 }, () => 'floor'), blocks: [] },
  actors: [
    { id: 0, seat: 'one', position: { x: 0, y: 0 }, heading: 'east', speedLevel: 1, alive: true, bombCapacity: 1, bombRange: 1, liveBombCount: 0 },
    { id: 1, seat: 'two', position: { x: 15_359, y: 13_311 }, heading: 'west', speedLevel: 1, alive: true, bombCapacity: 1, bombRange: 1, liveBombCount: 0 },
  ],
  bombs: [],
  flames: [],
  pickups: [],
}

const servers = [
  { type: 'authenticated', protocolVersion: 5 },
  { type: 'incompatibleProtocol', supportedVersion: 0 },
  { type: 'admitted', sequence: 1, room, youSeat: 'one', resumeToken: 'd'.repeat(64), lastBombActionId: null, state: null },
  { type: 'roomState', room },
  { type: 'snapshot', room, state, events: [] },
  { type: 'seatReplaced', roomId },
  { type: 'leftRoom', roomId, sequence: 1 },
  { type: 'error', code: 'invalidMessage', sequence: null },
] satisfies readonly GameServerMessage[]

const wrongAuthenticated: GameServerMessage = {
  type: 'authenticated',
  // @ts-expect-error v5 authentication is fixed to protocol version 5
  protocolVersion: 3,
}

// @ts-expect-error rematch readiness is required
const missingRematchReady: GameClientMessage = { type: 'rematch', sequence: 1, roomId, roundId: 1 }

const compileTimeExamples = [wrongAuthenticated, missingRematchReady]
const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV5Schema)

function validator<T>(definition: string): ValidateFunction<T> {
  const reference = `${gameplayV5Schema.$id}#/$defs/${definition}`
  const resolved = ajv.getSchema<T>(reference)
  if (!resolved) throw new Error(`AJV did not resolve ${reference}`)
  return resolved
}

const validateClient = validator<GameClientMessage>('GameClientMessage')
const validateServer = validator<GameServerMessage>('GameServerMessage')

function without(value: object, property: string): Record<string, unknown> {
  const copy = structuredClone(value) as Record<string, unknown>
  delete copy[property]
  return copy
}

function serverWithRoom(overrides: Record<string, unknown>): unknown {
  return { type: 'roomState', room: { ...room, ...overrides } }
}

function valid(validator: ValidateFunction, value: unknown): boolean {
  return validator(value)
}

describe('gameplay v5 client contract', () => {
  it('accepts all commands and closes every command object', () => {
    expect(compileTimeExamples).toHaveLength(2)
    for (const message of clients) {
      expect(valid(validateClient, message), JSON.stringify(validateClient.errors)).toBe(true)
      for (const field of Object.keys(message)) {
        expect(valid(validateClient, without(message, field)), `${message.type}.${field}`).toBe(false)
      }
      expect(valid(validateClient, { ...message, extra: true })).toBe(false)
      expect(valid(validateClient, { ...message, type: null })).toBe(false)
    }
  })

  it('accepts bounded cross-version auth input and strict counters', () => {
    for (const protocolVersion of [0, 2, 3, 65_535]) {
      expect(valid(validateClient, { type: 'authenticate', protocolVersion, token })).toBe(true)
    }
    for (const resumeToken of ['a'.repeat(63), 'a'.repeat(65), 'A'.repeat(64), 'g'.repeat(64)]) {
      expect(valid(validateClient, { ...clients[3], resumeToken })).toBe(false)
    }
    for (const protocolVersion of [-1, 65_536, 1.5, '3', null]) {
      expect(valid(validateClient, { type: 'authenticate', protocolVersion, token })).toBe(false)
    }
    for (const field of ['sequence', 'roundId'] as const) {
      for (const value of [0, maximumCounter + 1, 1.5, null]) {
        expect(valid(validateClient, { ...clients[8], [field]: value })).toBe(false)
      }
    }
  })
})

describe('gameplay v5 server contract', () => {
  it('fixes successful authentication to v5 while incompatibility remains bounded', () => {
    for (const message of servers) {
      expect(valid(validateServer, message), JSON.stringify(validateServer.errors)).toBe(true)
      for (const field of Object.keys(message)) {
        expect(valid(validateServer, without(message, field))).toBe(false)
      }
      expect(valid(validateServer, { ...message, extra: true })).toBe(false)
    }
    for (const protocolVersion of [0, 1, 2, 3, 4, 6, 65_535]) {
      expect(valid(validateServer, { type: 'authenticated', protocolVersion })).toBe(false)
      expect(valid(validateServer, { type: 'incompatibleProtocol', supportedVersion: protocolVersion })).toBe(true)
    }
    for (const supportedVersion of [-1, 65_536, null]) {
      expect(valid(validateServer, { type: 'incompatibleProtocol', supportedVersion })).toBe(false)
    }
  })

  it('keeps identity and capability personal and requires admitted nullables', () => {
    const admitted = servers[2]
    expect(valid(validateServer, { ...admitted, lastBombActionId: 1 })).toBe(true)
    expect(valid(validateServer, { ...admitted, lastBombActionId: 0 })).toBe(false)
    expect(valid(validateServer, { ...admitted, events: [] })).toBe(false)
    expect(valid(validateServer, without(admitted, 'state'))).toBe(false)
    expect(valid(validateServer, without(admitted, 'lastBombActionId'))).toBe(false)
    expect(valid(validateServer, { type: 'roomState', room, youSeat: 'one' })).toBe(false)
    expect(valid(validateServer, { type: 'roomState', room, resumeToken: 'e'.repeat(64) })).toBe(false)
    expect(valid(validateServer, { type: 'snapshot', room, state, events: [], matchSeed: 1 })).toBe(false)
    expect(valid(validateServer, { type: 'error', code: 'resumeRejected', sequence: null })).toBe(true)
    expect(valid(validateServer, { type: 'error', code: 'resumeRejected' })).toBe(false)
    expect(valid(validateServer, { type: 'error', code: 'resumeRejected', sequence: 0 })).toBe(false)
  })

  it('accepts every room phase and every terminal match result', () => {
    const phases: RoomPhase[] = [
      { type: 'waiting' },
      { type: 'countdown', kind: 'initial', endsAt: 180 },
      { type: 'countdown', kind: 'interRound', endsAt: 360 },
      { type: 'running' },
      { type: 'result', holdUntil: 540 },
      { type: 'matchResult', endedAt: 540, result: { type: 'winner', seat: 'one', reason: 'roundWins' }, rematchAt: 720 },
      { type: 'matchResult', endedAt: 540, result: { type: 'winner', seat: 'two', reason: 'forfeit' }, rematchAt: 720 },
      { type: 'matchResult', endedAt: 540, result: { type: 'abandoned' }, rematchAt: 720 },
      { type: 'matchResult', endedAt: 540, result: { type: 'interrupted', reason: 'connectionLost' }, rematchAt: 720 },
      { type: 'interrupted', endedAt: 1, reason: 'serverOverloaded' },
      { type: 'interrupted', endedAt: 2, reason: 'serverStopped' },
      { type: 'interrupted', endedAt: 3, reason: 'internalError' },
    ]
    for (const phase of phases) {
      const message = serverWithRoom({ phase })
      expect(valid(validateServer, message), JSON.stringify(validateServer.errors)).toBe(true)
      for (const field of Object.keys(phase)) {
        expect(valid(validateServer, serverWithRoom({ phase: without(phase, field) }))).toBe(false)
      }
      expect(valid(validateServer, serverWithRoom({ phase: { ...phase, extra: true } }))).toBe(false)
    }
  })

  it('requires and closes all new room metadata with exact nullability and bounds', () => {
    for (const field of ['roundNumber', 'scores', 'lastRound', 'seats', 'phase']) {
      expect(valid(validateServer, { type: 'roomState', room: without(room, field) })).toBe(false)
    }
    expect(valid(validateServer, serverWithRoom({ lastRound: null }))).toBe(true)
    for (const [field, value] of [
      ['roundNumber', -1], ['roundNumber', maximumCounter + 1], ['roundNumber', null],
      ['scores', null], ['lastRound', undefined], ['seats', null], ['phase', null],
    ] as const) {
      expect(valid(validateServer, serverWithRoom({ [field]: value }))).toBe(false)
    }
    for (const value of [-1, 4, 1.5, null]) {
      expect(valid(validateServer, serverWithRoom({ scores: { one: value, two: 0 } }))).toBe(false)
    }
    expect(valid(validateServer, serverWithRoom({ scores: { one: 1, two: 0, draw: 1 } }))).toBe(false)
    expect(valid(validateServer, serverWithRoom({ lastRound: { ...completedRound, roundNumber: 0 } }))).toBe(false)
    expect(valid(validateServer, serverWithRoom({ lastRound: { ...completedRound, extra: true } }))).toBe(false)
    expect(valid(validateServer, serverWithRoom({ seats: [{ ...room.seats[0], rematchReady: null }, room.seats[1]] }))).toBe(false)
    for (const connection of [
      { type: 'vacant' }, { type: 'connected' }, { type: 'expired' },
      { type: 'disconnected', expiresAt: maximumTick },
    ]) {
      expect(valid(validateServer, serverWithRoom({ seats: [{ ...room.seats[0], connection }, room.seats[1]] }))).toBe(true)
    }
    expect(valid(validateServer, serverWithRoom({ seats: [{ ...room.seats[0], connection: { type: 'disconnected' } }, room.seats[1]] }))).toBe(false)
    expect(valid(validateServer, serverWithRoom({ seats: [{ ...room.seats[0], connection: { type: 'connected', expiresAt: 1 } }, room.seats[1]] }))).toBe(false)
    expect(valid(validateServer, serverWithRoom({ metadata: true }))).toBe(false)
  })

  it('keeps every physical definition identical to v3', () => {
    const names = [
      'MapId', 'PickupKind', 'EntityId', 'Tick', 'Seat', 'Direction',
      'Cell', 'Position', 'RoundResult', 'WinnerResult', 'DrawResult', 'Terrain', 'GameState',
      'PickupView', 'ArenaView', 'ActorView', 'BombView', 'FlameView', 'WireEvent', 'EventKind',
      'BombPlacedEvent', 'BombDetonatedEvent', 'ActorEliminatedEvent', 'RoundEndedEvent',
      'PickupCollectedEventKind',
    ] as const
    for (const name of names) {
      expect(gameplayV5Schema.$defs[name]).toEqual(gameplayV3Schema.$defs[name])
    }
  })

  it('accepts forgiving rules while rejecting unknown rules versions', () => {
    for (const rulesVersion of [1, 2, 3]) {
      expect(valid(validateServer, serverWithRoom({ rulesVersion }))).toBe(true)
    }
    for (const rulesVersion of [0, 4, null]) {
      expect(valid(validateServer, serverWithRoom({ rulesVersion }))).toBe(false)
    }
    expect(gameplayV3Schema.$defs.RulesVersion.enum).toEqual([1, 2])
  })
})
