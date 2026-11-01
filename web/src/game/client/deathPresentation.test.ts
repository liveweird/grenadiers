import { describe, expect, it } from 'vitest'
import type { GameClientState } from './clientTypes'
import { initialGameClientState } from './clientTypes'
import { DeathPresentationTracker } from './deathPresentation'
import { room, snapshot } from './testSupport'
import type { Seat, Snapshot, VisibleActorView } from '../v19/schema'

const baseline = { roomId: 'b'.repeat(32), revision: 1, tick: 1, physicalTick: 1 }

function clientState(wire: Snapshot): GameClientState {
  return {
    ...initialGameClientState(), connection: 'authenticated', room: wire.room,
    snapshot: wire, youSeat: 'one', admissionBaseline: baseline,
  }
}

function eliminate(wire: Snapshot, actorIndex = 0, eventIndex = 0): void {
  const actor = wire.state.actors[actorIndex]
  if (actor?.type !== 'visible') throw new Error('expected visible actor')
  actor.alive = false
  wire.events = [{
    tick: wire.state.physicalTick, index: eventIndex,
    kind: { type: 'actorEliminated', id: actor.id, seat: actor.seat },
  }]
}

describe('death presentation tracker', () => {
  it('retains an event cue across a later callback before a React render', () => {
    const tracker = new DeathPresentationTracker()
    tracker.consume(clientState(snapshot(1)), 0)
    const killed = snapshot(2); eliminate(killed)
    tracker.consume(clientState(killed), 20)
    const latest = tracker.consume(clientState(snapshot(3)), 40)
    expect(latest.cues).toHaveLength(1)
    expect(latest.cues[0]).toMatchObject({ appearance: 'one', tick: 2, index: 0, bornAt: 20 })
  })

  it('never derives a death pose from a hidden cloaked actor or an admission baseline', () => {
    const tracker = new DeathPresentationTracker()
    tracker.consume(clientState(snapshot(1)), 0)
    const hidden = snapshot(2)
    hidden.state.actors[1] = { type: 'hidden', id: 2, seat: 'two', alive: false }
    hidden.events = [{ tick: 2, index: 0, kind: { type: 'actorEliminated', id: 2, seat: 'two' } }]
    expect(tracker.consume(clientState(hidden), 20).cues).toEqual([])

    const reconnect = clientState(hidden)
    reconnect.admissionBaseline = { ...baseline, revision: 2, tick: 2, physicalTick: 2 }
    const reset = tracker.consume(reconnect, 30)
    expect(reset.cues).toEqual([])
    expect(reset.epoch).toBe(1)
  })

  it('qualifies only a forward within-round mob removal whose lair remains', () => {
    const tracker = new DeathPresentationTracker()
    const initial = snapshot(1)
    initial.state.mobs = [{
      id: 90, lairId: 0, position: { x: 4096, y: 5120 }, heading: 'north', frozen: false,
    }]
    initial.state.lairs[0]!.livingMobs = 1
    tracker.consume(clientState(initial), 0)
    const killed = snapshot(2)
    killed.state.lairs[0]!.livingMobs = 0
    expect(tracker.consume(clientState(killed), 20).cues[0]).toMatchObject({
      appearance: 'mob', entityId: 90, position: { x: 4096, y: 5120 }, tick: 2,
    })

    const resetTracker = new DeathPresentationTracker()
    resetTracker.consume(clientState(initial), 0)
    const lairRemoved = snapshot(2)
    lairRemoved.state.lairs = lairRemoved.state.lairs.filter(({ id }) => id !== 0)
    expect(resetTracker.consume(clientState(lairRemoved), 20).cues).toEqual([])
    const nextRound = snapshot(3, 2)
    expect(resetTracker.consume(clientState(nextRound), 40).cues).toEqual([])
  })

  it('deduplicates repeated events and revisions while preserving a final running-to-result kill', () => {
    const tracker = new DeathPresentationTracker()
    tracker.consume(clientState(snapshot(1)), 0)
    const final = snapshot(2); eliminate(final, 1, 4)
    final.room = room(2, { type: 'result', holdUntil: 182 })
    final.state.physicalTick = 2
    final.events[0]!.tick = 2
    const ended = clientState(final)
    expect(tracker.consume(ended, 20).cues).toHaveLength(1)
    expect(tracker.consume(ended, 30).cues).toHaveLength(1)
    const repeated = snapshot(3); eliminate(repeated, 1, 4)
    repeated.events[0]!.tick = 2
    expect(tracker.consume(clientState(repeated), 40).cues).toHaveLength(1)
  })

  it('accepts a newly published elimination at the same physical tick', () => {
    const tracker = new DeathPresentationTracker()
    const initial = snapshot(1)
    tracker.consume(clientState(initial), 0)
    const departure = snapshot(2)
    departure.state.physicalTick = initial.state.physicalTick
    eliminate(departure, 0, 3)
    departure.events[0]!.tick = initial.state.physicalTick
    expect(tracker.consume(clientState(departure), 20).cues).toMatchObject([{
      entityId: 1, tick: initial.state.physicalTick, index: 3,
    }])
  })

  it.each(['result', 'matchResult'] as const)(
    'presents a mob and final actor killed by the same blast entering %s', (phase) => {
    const tracker = new DeathPresentationTracker()
    const initial = snapshot(1)
    initial.state.mobs = [{
      id: 90, lairId: 0, position: { x: 4096, y: 5120 }, heading: 'north', frozen: false,
    }]
    initial.state.lairs[0]!.livingMobs = 1
    tracker.consume(clientState(initial), 0)

    const terminal = snapshot(2)
    terminal.room = room(2, phase === 'result'
      ? { type: 'result', holdUntil: 182 }
      : { type: 'matchResult', endedAt: 2, rematchAt: 182,
          result: { type: 'winner', seat: 'two', reason: 'roundWins' } })
    terminal.state.lairs[0]!.livingMobs = 0
    eliminate(terminal, 0, 5)
    const deaths = tracker.consume(clientState(terminal), 20)

    expect(deaths.cues).toHaveLength(2)
    expect(deaths.cues).toEqual(expect.arrayContaining([
      expect.objectContaining({ appearance: 'one', entityId: 1, tick: 2, index: 5 }),
      expect.objectContaining({ appearance: 'mob', entityId: 90, position: { x: 4096, y: 5120 } }),
    ]))
  })

  it('does not infer a mob kill across a substantial publication gap', () => {
    const tracker = new DeathPresentationTracker()
    const initial = snapshot(1)
    initial.state.mobs = [{
      id: 90, lairId: 0, position: { x: 4096, y: 5120 }, heading: 'north', frozen: false,
    }]
    tracker.consume(clientState(initial), 0)
    const delayed = snapshot(62)
    expect(tracker.consume(clientState(delayed), 1010).cues).toEqual([])
  })

  it('copies every actor appearance and bounds retained cues', () => {
    const tracker = new DeathPresentationTracker()
    let current = snapshot(1)
    tracker.consume(clientState(current), 0)
    const seats: Seat[] = ['one', 'two', 'three', 'four']
    for (let revision = 2; revision <= 69; revision += 1) {
      current = snapshot(revision)
      const seat = seats[revision % seats.length]!
      const template = current.state.actors[0] as VisibleActorView
      current.state.actors = [{ ...template, id: revision, seat, alive: false }]
      current.events = [{
        tick: revision, index: 0,
        kind: { type: 'actorEliminated', id: revision, seat },
      }]
      tracker.consume(clientState(current), revision)
    }
    const result = tracker.consume(clientState(snapshot(70)), 70)
    expect(new Set(result.cues.map(({ appearance }) => appearance)))
      .toEqual(new Set(['one', 'two', 'three', 'four']))
    expect(result.cues).toHaveLength(64)
  })
})
