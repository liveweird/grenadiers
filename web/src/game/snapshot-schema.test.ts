import Ajv, { type ValidateFunction } from 'ajv'
import { describe, expect, it } from 'vitest'
import gameplaySchema from '../../../api/gameplay.schema.json'
import type { GameServerMessage } from './schema'

type SnapshotMessage = Extract<GameServerMessage, { type: 'snapshot' }>

const maximumTick = 9_007_199_254_740_991
const terrain: Array<'floor' | 'wall'> = Array.from(
  { length: 195 },
  (_, index) => index % 2 === 0 ? 'wall' : 'floor',
)
const bomb = {
  id: 30,
  cell: { column: 4, row: 5 },
  owner: 10,
  range: 8,
  placedAt: 40,
  detonatesAt: 220,
} as const

const snapshot: SnapshotMessage = {
  type: 'snapshot',
  room: {
    roomId: 'b'.repeat(32),
    roomCode: 'ABCDEFGH',
    roundId: 1,
    revision: 2,
    tick: maximumTick,
    rulesVersion: 1,
    mapId: 'fixed_v1',
    seats: [
      { seat: 'one', occupied: true, ready: true },
      { seat: 'two', occupied: true, ready: true },
    ],
    phase: { type: 'running' },
  },
  state: {
    runningUpdates: 1,
    remainingTicks: 10_799,
    arena: {
      width: 15,
      height: 13,
      terrain,
      blocks: [{ column: 1, row: 1 }, { column: 2, row: 1 }],
    },
    actors: [
      {
        id: 10,
        seat: 'one',
        position: { x: 1_536, y: 1_536 },
        heading: 'east',
        speedLevel: 0,
        alive: true,
        bombCapacity: 1,
        bombRange: 1,
        liveBombCount: 1,
      },
      {
        id: 20,
        seat: 'two',
        position: { x: 13_824, y: 11_776 },
        heading: 'west',
        speedLevel: 4,
        alive: false,
        bombCapacity: 8,
        bombRange: 8,
        liveBombCount: 0,
      },
    ],
    bombs: [bomb],
    flames: [{ cell: { column: 3, row: 5 }, expiresAt: maximumTick }],
  },
  events: [
    { tick: 40, index: 0, kind: { type: 'bombPlaced', bomb } },
    { tick: 40, index: 1, kind: { type: 'bombDetonated', bomb } },
    { tick: 40, index: 2, kind: { type: 'actorEliminated', id: 20, seat: 'two' } },
    { tick: 40, index: 3, kind: { type: 'roundEnded', result: { type: 'winner', seat: 'one' } } },
  ],
}

const wrongOwner: SnapshotMessage = {
  ...snapshot,
  state: {
    ...snapshot.state,
    bombs: [{
      ...bomb,
      // @ts-expect-error bomb owners are numeric entity identifiers
      owner: 'one',
    }],
  },
}

const wrongPosition: SnapshotMessage = {
  ...snapshot,
  state: {
    ...snapshot.state,
    actors: [{
      ...snapshot.state.actors[0],
      position: {
        ...snapshot.state.actors[0].position,
        // @ts-expect-error fixed-point coordinates are numbers
        x: '1536',
      },
    }, snapshot.state.actors[1]],
  },
}

const wrongEvent: SnapshotMessage = {
  ...snapshot,
  events: [{
    tick: 40,
    index: 0,
    // @ts-expect-error bomb events require their complete bomb payload
    kind: { type: 'bombPlaced' },
  }],
}

const compileTimeExamples = [wrongOwner, wrongPosition, wrongEvent]
const ajv = new Ajv({ allErrors: true }).addSchema(gameplaySchema)
const snapshotReference = `${gameplaySchema.$id}#/$defs/Snapshot`

function snapshotValidator(): ValidateFunction<SnapshotMessage> {
  const validator = ajv.getSchema<SnapshotMessage>(snapshotReference)
  if (!validator) throw new Error(`AJV did not resolve ${snapshotReference}`)
  return validator
}

const validateSnapshot = snapshotValidator()

function valid(value: unknown): boolean {
  return validateSnapshot(value)
}

function copy(): SnapshotMessage {
  return structuredClone(snapshot)
}

function cells(count: number): Array<{ column: number; row: number }> {
  return Array.from({ length: count }, (_, index) => ({
    column: index % 15,
    row: Math.floor(index / 15) % 13,
  }))
}

function terrainCells(count: number): Array<'floor' | 'wall'> {
  return Array.from({ length: count }, (_, index) => terrain[index % terrain.length])
}

function expectInvalid(value: unknown): void {
  expect(valid(value), JSON.stringify(validateSnapshot.errors)).toBe(false)
}

describe('gameplay snapshot generated types', () => {
  it('accepts the complete nested snapshot and checks negative assignments', () => {
    const message: GameServerMessage = snapshot
    expect(message.type).toBe('snapshot')
    expect(compileTimeExamples).toHaveLength(3)
  })
})

describe('gameplay snapshot schema', () => {
  it('accepts every event kind and same-tick placement followed by detonation', () => {
    expect(valid(snapshot), JSON.stringify(validateSnapshot.errors)).toBe(true)
    expect(snapshot.events[0].tick).toBe(snapshot.events[1].tick)
    expect(snapshot.events[0].kind.type).toBe('bombPlaced')
    expect(snapshot.events[1].kind.type).toBe('bombDetonated')
  })

  it('rejects missing and extra fields throughout the nested snapshot', () => {
    for (const field of Object.keys(snapshot)) {
      const missing = copy()
      Reflect.deleteProperty(missing, field)
      expectInvalid(missing)
    }

    const missingActorField = copy()
    Reflect.deleteProperty(missingActorField.state.actors[0], 'alive')
    expectInvalid(missingActorField)

    const missingEventField = copy()
    Reflect.deleteProperty(missingEventField.events[0], 'kind')
    expectInvalid(missingEventField)

    expectInvalid({ ...copy(), youSeat: 'one' })
    expectInvalid({ ...copy(), state: { ...copy().state, score: 1 } })
    expectInvalid({ ...copy(), state: { ...copy().state, items: [] } })
    expectInvalid({ ...copy(), state: { ...copy().state, arena: { ...copy().state.arena, seed: 1 } } })
    const event = copy().events[0]
    expectInvalid({ ...copy(), events: [{ ...event, kind: { ...event.kind, extra: true } }] })
  })

  it('rejects wrong owner and position types and out-of-board coordinates', () => {
    const owner = copy()
    owner.state.bombs[0].owner = '10' as unknown as number
    expectInvalid(owner)

    const position = copy()
    position.state.actors[0].position.x = '1536' as unknown as number
    expectInvalid(position)

    for (const badPosition of [{ x: -1 }, { x: 15_360 }, { y: -1 }, { y: 13_312 }]) {
      const message = copy()
      Object.assign(message.state.actors[0].position, badPosition)
      expectInvalid(message)
    }
    for (const badCell of [{ column: -1 }, { column: 15 }, { row: -1 }, { row: 13 }]) {
      const message = copy()
      Object.assign(message.state.bombs[0].cell, badCell)
      expectInvalid(message)
    }
  })

  it('enforces exact and maximum array sizes plus unique cell elements', () => {
    for (const count of [1, 3]) {
      const actors = Array.from({ length: count }, () => snapshot.state.actors[0])
      expectInvalid({ ...copy(), state: { ...copy().state, actors } })
    }
    for (const count of [194, 196]) {
      expectInvalid({ ...copy(), state: { ...copy().state, arena: { ...copy().state.arena, terrain: terrainCells(count) } } })
    }
    expectInvalid({ ...copy(), state: { ...copy().state, arena: { ...copy().state.arena, blocks: cells(196) } } })
    expectInvalid({ ...copy(), state: { ...copy().state, bombs: Array.from({ length: 17 }, () => bomb) } })
    expectInvalid({ ...copy(), state: { ...copy().state, flames: Array.from({ length: 196 }, () => snapshot.state.flames[0]) } })
    expectInvalid({ ...copy(), events: Array.from({ length: 65 }, () => snapshot.events[0]) })
    expectInvalid({ ...copy(), state: { ...copy().state, arena: { ...copy().state.arena, blocks: cells(1).concat(cells(1)) } } })
    expect(valid({ ...copy(), state: { ...copy().state, bombs: [], flames: [] }, events: [] })).toBe(true)
  })

  it('enforces state, actor, bomb, event, and JavaScript-safe tick bounds', () => {
    for (const [field, value] of [['runningUpdates', 10_801], ['remainingTicks', -1]] as const) {
      expectInvalid({ ...copy(), state: { ...copy().state, [field]: value } })
    }
    for (const [field, value] of [
      ['speedLevel', 5], ['bombCapacity', 0], ['bombRange', 9], ['liveBombCount', 9],
    ] as const) {
      const message = copy()
      Object.assign(message.state.actors[0], { [field]: value })
      expectInvalid(message)
    }
    const badRange = copy()
    badRange.state.bombs[0].range = 9
    expectInvalid(badRange)

    for (const index of [-1, 64]) expectInvalid({ ...copy(), events: [{ ...copy().events[0], index }] })
    expectInvalid({ ...copy(), room: { ...copy().room, tick: maximumTick + 1 } })
    expectInvalid({ ...copy(), events: [{ ...copy().events[0], tick: maximumTick + 1 }] })
    expectInvalid({ ...copy(), state: { ...copy().state, flames: [{ ...copy().state.flames[0], expiresAt: maximumTick + 1 }] } })
    expectInvalid({ ...copy(), state: { ...copy().state, bombs: [{ ...bomb, detonatesAt: maximumTick + 1 }] } })
  })
})
