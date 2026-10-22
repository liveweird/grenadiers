import { describe, expect, it } from 'vitest'
import type { GameClientState } from '../client/client'
import type { EventKind, WireEvent } from '../v9/schema'
import { snapshot } from '../client/testSupport'
import { AudioCursor } from './cursor'

const baseline = { roomId: 'b'.repeat(32), revision: 20, tick: 20, physicalTick: 20 }

type AudibleEventType = 'bombPlaced' | 'bombDetonated' | 'pickupCollected' | 'actorEliminated'

function event(tick: number, index: number, type: AudibleEventType): WireEvent {
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
  current.state.physicalTick = revision
  current.events = events
  return {
    connection: 'authenticated', room: current.room, snapshot: current, youSeat: 'one', failure: null,
    admissionPending: false, leavePending: false, settingsPending: false, reconnectAttempt: 0,
    admissionBaseline: baseline,
  }
}

describe('accepted-event audio cursor', () => {
  it('seeds silently, orders and coalesces a bounded snapshot, and rejects old revisions', () => {
    const cursor = new AudioCursor()
    expect(cursor.consume(state(20), true)).toEqual([])
    const events = Array.from({ length: 64 }, (_, index) => event(21, index % 4,
      ['bombPlaced', 'pickupCollected', 'bombDetonated', 'actorEliminated'][index % 4] as AudibleEventType))
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

  it('uses physical time for event completion independently of lifecycle room time', () => {
    const cursor = new AudioCursor()
    cursor.consume(state(20), true)
    const lifecycleOnly = state(21, [event(21, 0, 'bombPlaced')])
    lifecycleOnly.snapshot!.state.physicalTick = 20
    lifecycleOnly.room!.tick = 900
    lifecycleOnly.snapshot!.room.tick = 900
    expect(cursor.consume(lifecycleOnly, true)).toEqual([])

    const physicallyComplete = state(22, [event(21, 0, 'bombPlaced')])
    physicallyComplete.snapshot!.state.physicalTick = 21
    expect(cursor.consume(physicallyComplete, true)).toEqual(['placement'])
  })

  it('seeds round, room, and fresh admission boundaries without replay', () => {
    const cursor = new AudioCursor()
    cursor.consume(state(20), true)
    expect(cursor.consume(state(30, [event(20, 0, 'bombPlaced')], 2), true)).toEqual([])
    const later = state(31, [event(31, 0, 'bombDetonated')], 2)
    expect(cursor.consume(later, true)).toEqual(['detonation'])
    const resumed = state(41, [event(41, 0, 'actorEliminated')], 2)
    resumed.admissionBaseline = { roomId: resumed.room!.roomId, revision: 40, tick: 40, physicalTick: 40 }
    expect(cursor.consume(resumed, true)).toEqual([])
    const afterResume = state(42, [event(42, 0, 'bombPlaced')], 2)
    afterResume.admissionBaseline = resumed.admissionBaseline
    expect(cursor.consume(afterResume, true)).toEqual(['placement'])
    const other = state(3, [event(3, 0, 'pickupCollected')])
    other.room!.roomId = 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'
    other.snapshot!.room.roomId = other.room!.roomId
    other.admissionBaseline = { roomId: other.room!.roomId, revision: 2, tick: 2, physicalTick: 2 }
    expect(cursor.consume(other, true)).toEqual([])
  })

  it('plays only recipient-local cloak events and does not invent an omitted opponent cue', () => {
    const owner = new AudioCursor()
    const opponent = new AudioCursor()
    owner.consume(state(20), true)
    opponent.consume(state(20), true)
    expect(owner.consume(state(21, [event(21, 0, 'pickupCollected')]), true)).toEqual(['pickup'])
    expect(opponent.consume(state(21, []), true)).toEqual([])
    expect(opponent.consume(state(22, []), true)).toEqual([])
  })

  it('keeps public sounds while blind bomb cues, admission, and reveal stay quiet', () => {
    const cursor = new AudioCursor()
    cursor.consume(state(20), true)
    expect(cursor.consume(state(21, [
      event(21, 0, 'pickupCollected'), event(21, 1, 'actorEliminated'),
    ]), true)).toEqual(['elimination', 'pickup'])
    expect(cursor.consume(state(22, []), true)).toEqual([])
    const resumed = state(30, [], 2)
    resumed.admissionBaseline = { roomId: resumed.room!.roomId, revision: 29, tick: 29, physicalTick: 29 }
    expect(cursor.consume(resumed, true)).toEqual([])
    expect(cursor.consume(state(31, [], 2), true)).toEqual([])
  })
})
