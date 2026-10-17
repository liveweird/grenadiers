import { describe, expect, it } from 'vitest'
import type { GameClientState } from '../v4/client'
import type { EventKind, WireEvent } from '../v4/schema'
import { snapshot } from '../v4/testSupport'
import { AudioCursor } from './cursor'

const baseline = { roomId: 'b'.repeat(32), revision: 20, tick: 20 }

function event(tick: number, index: number, type: EventKind['type']): WireEvent {
  const kinds = {
    bombPlaced: { type: 'bombPlaced', bomb: {} }, bombDetonated: { type: 'bombDetonated', bomb: {} },
    pickupCollected: { type: 'pickupCollected', actor: 1, pickup: 'bombCapacity' },
    actorEliminated: { type: 'actorEliminated', id: 1, seat: 'one' },
    roundEnded: { type: 'roundEnded', result: { type: 'timeout' } },
  }
  return { tick, index, kind: kinds[type] as EventKind }
}

function state(revision: number, events: WireEvent[] = [], roundId = 1): GameClientState {
  const current = snapshot(revision, roundId)
  current.events = events
  return {
    connection: 'authenticated', room: current.room, snapshot: current, youSeat: 'one', failure: null,
    admissionPending: false, leavePending: false, reconnectAttempt: 0,
    admissionBaseline: baseline,
  }
}

describe('accepted-event audio cursor', () => {
  it('seeds silently, orders and coalesces a bounded snapshot, and rejects old revisions', () => {
    const cursor = new AudioCursor()
    expect(cursor.consume(state(20), true)).toEqual([])
    const events = Array.from({ length: 64 }, (_, index) => event(21, index % 4,
      ['bombPlaced', 'pickupCollected', 'bombDetonated', 'actorEliminated'][index % 4] as EventKind['type']))
    expect(cursor.consume(state(21, events), true)).toEqual(['elimination', 'detonation', 'pickup', 'placement'])
    expect(cursor.consume(state(21, events), true)).toEqual([])
    expect(cursor.consume(state(19, [event(19, 0, 'bombPlaced')]), true)).toEqual([])
  })

  it('discards muted events and requires a later completed tick after enabling', () => {
    const cursor = new AudioCursor()
    cursor.consume(state(20), false)
    expect(cursor.consume(state(21, [event(21, 0, 'bombPlaced')]), false)).toEqual([])
    expect(cursor.consume(state(21, [event(21, 0, 'bombPlaced')]), true)).toEqual([])
    expect(cursor.consume(state(22, [event(22, 0, 'pickupCollected')]), true)).toEqual(['pickup'])
  })

  it('seeds round, room, and fresh admission boundaries without replay', () => {
    const cursor = new AudioCursor()
    cursor.consume(state(20), true)
    expect(cursor.consume(state(30, [event(20, 0, 'bombPlaced')], 2), true)).toEqual([])
    const later = state(31, [event(31, 0, 'bombDetonated')], 2)
    expect(cursor.consume(later, true)).toEqual(['detonation'])
    const resumed = state(41, [event(41, 0, 'actorEliminated')], 2)
    resumed.admissionBaseline = { roomId: resumed.room!.roomId, revision: 40, tick: 40 }
    expect(cursor.consume(resumed, true)).toEqual([])
    const afterResume = state(42, [event(42, 0, 'bombPlaced')], 2)
    afterResume.admissionBaseline = resumed.admissionBaseline
    expect(cursor.consume(afterResume, true)).toEqual(['placement'])
    const other = state(3, [event(3, 0, 'pickupCollected')])
    other.room!.roomId = 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'
    other.snapshot!.room.roomId = other.room!.roomId
    other.admissionBaseline = { roomId: other.room!.roomId, revision: 2, tick: 2 }
    expect(cursor.consume(other, true)).toEqual([])
  })
})
