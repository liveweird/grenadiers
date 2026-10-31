import { describe, expect, it } from 'vitest'
import type { GameClientState } from './clientTypes'
import { initialGameClientState } from './clientTypes'
import { RoundOpeningTracker } from './roundOpening'
import { room, snapshot } from './testSupport'

const baseline = { roomId: 'b'.repeat(32), revision: 1, tick: 1, physicalTick: 1 }

function state(revision: number, phase: 'waiting' | 'countdown' | 'running'): GameClientState {
  const current = room(revision, phase === 'countdown'
    ? { type: 'countdown', kind: 'initial', endsAt: revision + 180 }
    : { type: phase })
  const wire = phase === 'running' ? snapshot(revision) : null
  if (wire) wire.room = current
  return {
    ...initialGameClientState(), connection: 'authenticated', room: current, snapshot: wire,
    youSeat: 'one', admissionBaseline: baseline,
  }
}

describe('round opening presentation tracker', () => {
  it('latches a snapshot-free countdown through a long wait and emits on first running', () => {
    const tracker = new RoundOpeningTracker()
    expect(tracker.consume(state(1, 'waiting'), 0)).toBeNull()
    expect(tracker.consume(state(2, 'countdown'), 20)).toBeNull()
    expect(tracker.consume(state(3, 'countdown'), 20_000)).toBeNull()
    const cue = tracker.consume(state(4, 'running'), 21_000)
    expect(cue).toEqual({ key: `${baseline.roomId}:1:1:1:1`, bornAt: 21_000 })
  })

  it('retains a cue across coalesced running callbacks and expires it', () => {
    const tracker = new RoundOpeningTracker()
    tracker.consume(state(1, 'waiting'), 0)
    tracker.consume(state(2, 'countdown'), 20)
    const first = tracker.consume(state(3, 'running'), 100)
    expect(tracker.consume(state(4, 'running'), 200)).toEqual(first)
    expect(tracker.consume(state(5, 'running'), 1_000)).toBeNull()
  })

  it('keeps the countdown armed until the matching running world is accepted', () => {
    const tracker = new RoundOpeningTracker()
    tracker.consume(state(1, 'waiting'), 0)
    tracker.consume(state(2, 'countdown'), 10)
    const roomOnly = state(3, 'running')
    roomOnly.snapshot = null
    expect(tracker.consume(roomOnly, 20)).toBeNull()
    expect(tracker.consume(state(4, 'running'), 40)).toEqual({
      key: `${baseline.roomId}:1:1:1:1`, bornAt: 40,
    })
  })

  it('does not open for initial running or result admission', () => {
    const tracker = new RoundOpeningTracker()
    expect(tracker.consume(state(1, 'running'), 0)).toBeNull()
    const result = state(2, 'running')
    result.room!.phase = { type: 'result', holdUntil: 182 }
    result.snapshot!.room = result.room!
    expect(new RoundOpeningTracker().consume(result, 0)).toBeNull()
  })

  it('clears pending countdowns across disconnect, room, admission, and backward updates', () => {
    const mutations: Array<(value: GameClientState) => void> = [
      (value) => { value.connection = 'disconnected' },
      (value) => { value.room!.roomId = 'c'.repeat(32) },
      (value) => { value.admissionBaseline = { ...baseline, revision: 2 } },
      (value) => { value.room!.roundId = 0 },
      (value) => { value.room!.revision = 0; value.room!.tick = 0 },
    ]
    for (const mutate of mutations) {
      const tracker = new RoundOpeningTracker()
      tracker.consume(state(1, 'waiting'), 0)
      tracker.consume(state(2, 'countdown'), 5)
      const changed = state(3, 'countdown')
      mutate(changed)
      tracker.consume(changed, 10)
      expect(tracker.consume(state(4, 'running'), 20)).toBeNull()
    }
  })

  it('does not invent a cue when the countdown callback was skipped', () => {
    const tracker = new RoundOpeningTracker()
    tracker.consume(state(1, 'waiting'), 0)
    expect(tracker.consume(state(2, 'running'), 20)).toBeNull()
  })

  it('silently seeds a reconnect admission already inside countdown', () => {
    const tracker = new RoundOpeningTracker()
    tracker.consume(state(1, 'running'), 0)
    const reconnecting = state(2, 'countdown')
    reconnecting.connection = 'reconnecting'
    reconnecting.admissionPending = true
    tracker.consume(reconnecting, 10)
    const resumed = state(3, 'countdown')
    resumed.admissionBaseline = { ...baseline, revision: 3, tick: 3 }
    expect(tracker.consume(resumed, 20)).toBeNull()
    const periodic = state(4, 'countdown')
    periodic.admissionBaseline = resumed.admissionBaseline
    expect(tracker.consume(periodic, 25)).toBeNull()
    const live = state(5, 'running')
    live.admissionBaseline = resumed.admissionBaseline
    expect(tracker.consume(live, 30)).toBeNull()
  })

  it('keeps a late first countdown admission quiet until a later genuine edge', () => {
    const tracker = new RoundOpeningTracker()
    expect(tracker.consume(state(1, 'countdown'), 0)).toBeNull()
    expect(tracker.consume(state(2, 'countdown'), 10)).toBeNull()
    expect(tracker.consume(state(3, 'running'), 20)).toBeNull()
    expect(tracker.consume(state(4, 'waiting'), 30)).toBeNull()
    expect(tracker.consume(state(5, 'countdown'), 40)).toBeNull()
    expect(tracker.consume(state(6, 'running'), 50)).toEqual({
      key: `${baseline.roomId}:1:1:1:1`, bornAt: 50,
    })
  })

  it('arms when consecutive countdown observations advance to a newer round', () => {
    const tracker = new RoundOpeningTracker()
    tracker.consume(state(1, 'countdown'), 0)
    const nextCountdown = state(2, 'countdown')
    nextCountdown.room!.roundId = 2
    tracker.consume(nextCountdown, 10)
    const live = state(3, 'running')
    live.room!.roundId = 2
    live.snapshot!.room = live.room!
    expect(tracker.consume(live, 20)).toEqual({
      key: `${baseline.roomId}:1:1:1:2`, bornAt: 20,
    })
  })
})
