import { describe, expect, it } from 'vitest'
import type { GameClientState } from './clientTypes'
import type { PickupKind, WireEvent } from '../v11/schema'
import { snapshot } from './testSupport'
import { OwnPickupCursor } from './ownPickupCursor'

const baseline = { roomId: 'a'.repeat(32), revision: 20, tick: 20, physicalTick: 20 }

function state(revision: number, physicalTick = revision): GameClientState {
  const current = snapshot(revision)
  current.state.physicalTick = physicalTick
  return {
    connection: 'authenticated', room: current.room, snapshot: current, youSeat: 'one',
    failure: null, admissionPending: false, leavePending: false, settingsPending: false,
    reconnectAttempt: 0, admissionBaseline: baseline,
  }
}

function pickup(tick: number, index: number, seat: 'one' | 'two', kind: PickupKind): WireEvent {
  return {
    tick, index,
    kind: { type: 'pickupCollected', id: seat === 'one' ? 1 : 2, seat, cell: { column: 1, row: 1 }, kind },
  }
}

describe('own pickup cursor', () => {
  it('seeds silently, reports recipient pickups, keeps equal frames stable, and expires by physical time', () => {
    const cursor = new OwnPickupCursor()
    expect(cursor.consume(state(20))).toBeNull()
    const collected = state(21)
    collected.snapshot!.events = [pickup(21, 0, 'one', 'teleporter')]
    expect(cursor.consume(collected)).toEqual({ identity: '21:0', kind: 'teleporter' })
    expect(cursor.consume(collected)).toEqual({ identity: '21:0', kind: 'teleporter' })

    const opponentOnly = state(22)
    opponentOnly.snapshot!.events = [pickup(22, 0, 'two', 'heart')]
    expect(cursor.consume(opponentOnly)).toEqual({ identity: '21:0', kind: 'teleporter' })
    expect(cursor.consume(state(202, 202))).toBeNull()
  })

  it('accepts base upgrades and refreshes, while Oil does not become inventory feedback', () => {
    const cursor = new OwnPickupCursor()
    cursor.consume(state(20))
    const capacity = state(21)
    capacity.snapshot!.events = [pickup(21, 0, 'one', 'bombCapacity')]
    expect(cursor.consume(capacity)?.kind).toBe('bombCapacity')
    const refreshed = state(22)
    refreshed.snapshot!.events = [
      pickup(22, 0, 'one', 'oilBarrel'), pickup(22, 1, 'one', 'bombCapacity'),
    ]
    expect(cursor.consume(refreshed)).toEqual({ identity: '22:1', kind: 'bombCapacity' })
  })

  it('clears on death, disconnect, round change, departure, and fresh admission without replay', () => {
    const cursor = new OwnPickupCursor()
    cursor.consume(state(20))
    const collected = state(21)
    collected.snapshot!.events = [pickup(21, 0, 'one', 'heart')]
    expect(cursor.consume(collected)?.kind).toBe('heart')

    const dead = state(22)
    const own = dead.snapshot!.state.actors[0]
    if (own.type !== 'visible') throw new Error('visible actor expected')
    own.alive = false
    expect(cursor.consume(dead)).toBeNull()
    const disconnected = state(23)
    disconnected.connection = 'reconnecting'
    disconnected.admissionPending = true
    expect(cursor.consume(disconnected)).toBeNull()

    const resumed = state(24)
    resumed.admissionBaseline = { ...baseline, revision: 24, physicalTick: 24 }
    resumed.snapshot!.events = [pickup(24, 0, 'one', 'shield')]
    expect(cursor.consume(resumed)).toBeNull()
    const nextRound = state(25)
    nextRound.room!.roundId = 2
    nextRound.snapshot!.room.roundId = 2
    nextRound.snapshot!.events = [pickup(25, 0, 'one', 'shield')]
    expect(cursor.consume(nextRound)).toBeNull()
    expect(cursor.consume({ ...nextRound, room: null, snapshot: null })).toBeNull()
  })
})
