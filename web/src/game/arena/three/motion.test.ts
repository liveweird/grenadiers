import { describe, expect, it } from 'vitest'
import { gameSnapshot } from '../../../test/game-snapshot'
import { motionTiming, nextMotion, sampleMotion, unitsToGround } from './motion'

describe('snapshot motion', () => {
  it('derives bounded tick timing and starts from the currently displayed pose', () => {
    const snapshot = gameSnapshot()
    const first = motionTiming(snapshot, 0, null)
    snapshot.state.physicalTick = (snapshot.state.physicalTick ?? 0) + 3
    const timing = motionTiming(snapshot, 100, first)
    expect(timing).toMatchObject({ duration: 50, reset: false, running: true })
    const segment = nextMotion({ x: 1.5, z: 2 }, { x: 3, z: 2 }, timing, false)
    expect(sampleMotion(segment, 100)).toEqual({ x: 1.5, z: 2 })
    expect(sampleMotion(segment, 125)).toEqual({ x: 2.25, z: 2 })
    expect(sampleMotion(segment, 100_000)).toEqual({ x: 3, z: 2 })
    expect(sampleMotion(segment, 90)).toEqual({ x: 1.5, z: 2 })
  })

  it('resets for identity and backward ticks and clamps long gaps to 100ms', () => {
    const snapshot = gameSnapshot()
    const first = motionTiming(snapshot, 0, null)
    snapshot.state.physicalTick = (snapshot.state.physicalTick ?? 0) + 100
    const gap = motionTiming(snapshot, 10, first)
    expect(gap).toMatchObject({ duration: 100, reset: false })
    snapshot.state.physicalTick = 1
    expect(motionTiming(snapshot, 20, gap).reset).toBe(true)
    snapshot.room.presentationBaseline = 'resumed'
    expect(motionTiming(snapshot, 30, gap).reset).toBe(true)
  })

  it('snaps terminal, reduced, teleport, and freeze transitions without extrapolation', () => {
    const snapshot = gameSnapshot()
    const first = motionTiming(snapshot, 0, null)
    snapshot.room.phase = { type: 'result' }
    const terminal = motionTiming(snapshot, 20, first)
    const segment = nextMotion({ x: 1, z: 1 }, { x: 8, z: 9 }, terminal, false)
    expect(sampleMotion(segment, 20)).toEqual({ x: 8, z: 9 })
    expect(nextMotion({ x: 1, z: 1 }, { x: 2, z: 2 }, first, true).snap).toBe(true)
  })

  it('projects only recipient-filtered public coordinates into ground space', () => {
    expect(unitsToGround({ x: 1536, y: 2560 })).toEqual({ x: 1.5, z: 2.5 })
  })

  it('supports legacy tick identity while excluding dead rows and retaining freeze edges', () => {
    const snapshot = gameSnapshot()
    delete snapshot.room.presentationBaseline
    delete snapshot.state.physicalTick
    snapshot.state.runningUpdates = 12
    snapshot.state.actors[1].alive = false
    const first = motionTiming(snapshot, 0, null)
    expect(first).toMatchObject({ identity: `${snapshot.room.roomId}:1:legacy`, tick: 12 })
    expect(first.actors.has(snapshot.state.actors[1].id)).toBe(false)
    const actor = snapshot.state.actors[0]
    const generation = first.actorEpochs.get(actor.id) ?? 0
    actor.voluntaryActionsFrozen = true
    snapshot.state.runningUpdates += 1
    const frozen = motionTiming(snapshot, 20, first)
    expect(frozen.actorEpochs.get(actor.id)).toBe(generation + 1)
    expect(frozen.actors.get(actor.id)?.frozen).toBe(true)
  })
})
