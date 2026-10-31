import { describe, expect, it } from 'vitest'
import { gameSnapshot } from '../../../test/game-snapshot'
import type { PresentationDeathCue, SnapshotMessage } from '../types'
import { advanceBloodPools, clearBloodPools, initialBloodPoolState } from './bloodPoolState'

const cue = (key: string, x: number, y: number): PresentationDeathCue => ({
  key, appearance: 'mob', entityId: 1, position: { x, y }, heading: 'south',
  tick: 1, index: 0, bornAt: 0,
})

const withDeaths = (
  snapshot: SnapshotMessage, sequence: number, cues: PresentationDeathCue[], epoch = 1,
): SnapshotMessage => {
  snapshot.presentationDeaths = { epoch, sequence, cues }
  return snapshot
}

describe('blood pool state', () => {
  it('accepts latched safe cues on first mount and consumes only later unseen cues', () => {
    const snapshot = withDeaths(gameSnapshot(), 4, [cue('retained', 512, 512)])
    let state = advanceBloodPools(initialBloodPoolState(), snapshot)
    expect(state.pools.map(({ key }) => key)).toEqual(['blood:0:0'])
    snapshot.presentationDeaths = {
      epoch: 1, sequence: 5,
      cues: [cue('retained', 512, 512), cue('fresh', 1536, 2560)],
    }
    state = advanceBloodPools(state, snapshot)
    expect(state.pools).toMatchObject([
      { key: 'blood:0:0' },
      { key: 'blood:1:2', column: 1, row: 2, position: { x: 1536, y: 2560 } },
    ])
    expect(advanceBloodPools(state, snapshot).pools).toEqual(state.pools)
  })

  it('keeps the latest actual position per tile and bounds pools to valid board cells', () => {
    const snapshot = withDeaths(gameSnapshot(), 0, [])
    let state = advanceBloodPools(initialBloodPoolState(), snapshot)
    const { width, height } = snapshot.state.arena
    const cues = Array.from({ length: width * height }, (_, index) => cue(
      `cell:${index}`, (index % width) * 1024 + 256, Math.floor(index / width) * 1024 + 768,
    ))
    cues.push(cue('same-cell-latest', 900, 700), cue('outside', width * 1024, 0))
    snapshot.presentationDeaths = { epoch: 1, sequence: 1, cues }
    state = advanceBloodPools(state, snapshot)
    expect(state.pools).toHaveLength(width * height)
    expect(state.pools.find(({ key }) => key === 'blood:0:0')?.position).toEqual({ x: 900, y: 700 })
    expect(state.pools.every(({ column, row }) => column < width && row < height)).toBe(true)
  })

  it('persists through result snapshots but clears on every presentation discontinuity', () => {
    const snapshot = withDeaths(gameSnapshot(), 0, [])
    let state = advanceBloodPools(initialBloodPoolState(), snapshot)
    snapshot.presentationDeaths = { epoch: 1, sequence: 1, cues: [cue('death', 512, 512)] }
    state = advanceBloodPools(state, snapshot)
    snapshot.room.phase = { type: 'result' }
    snapshot.state.physicalTick = (snapshot.state.physicalTick ?? 0) + 20
    state = advanceBloodPools(state, snapshot)
    expect(state.pools).toHaveLength(1)

    snapshot.presentationDeaths = { epoch: 1, sequence: 0, cues: [cue('death', 512, 512)] }
    state = advanceBloodPools(state, snapshot)
    expect(state.pools).toEqual([])
    snapshot.presentationDeaths = { epoch: 2, sequence: 0, cues: [] }
    state = advanceBloodPools(state, snapshot)
    expect(state.pools).toEqual([])
    snapshot.presentationDeaths = { epoch: 2, sequence: 1, cues: [cue('new', 1536, 512)] }
    state = advanceBloodPools(state, snapshot)
    expect(state.pools).toHaveLength(1)
    snapshot.state.physicalTick -= 1
    state = advanceBloodPools(state, snapshot)
    expect(state.pools).toEqual([])
    snapshot.room.roundId += 1
    snapshot.state.physicalTick += 2
    snapshot.presentationDeaths = { epoch: 3, sequence: 3, cues: [cue('round-fresh', 512, 512)] }
    expect(advanceBloodPools(state, snapshot).pools.map(({ key }) => key)).toEqual(['blood:0:0'])
  })

  it('does not replay retained cues after suspension and clears without presentation', () => {
    const snapshot = withDeaths(gameSnapshot(), 0, [])
    let state = advanceBloodPools(initialBloodPoolState(), snapshot)
    snapshot.presentationDeaths = { epoch: 1, sequence: 1, cues: [cue('old', 512, 512)] }
    state = advanceBloodPools(state, snapshot)
    expect(state.pools).toHaveLength(1)
    state = clearBloodPools(state)
    expect(advanceBloodPools(state, snapshot).pools).toEqual([])
    snapshot.presentationDeaths = {
      epoch: 1, sequence: 2, cues: [cue('old', 512, 512), cue('fresh', 1536, 512)],
    }
    state = advanceBloodPools(state, snapshot)
    expect(state.pools.map(({ key }) => key)).toEqual(['blood:1:0'])
    delete snapshot.presentationDeaths
    state = advanceBloodPools(state, snapshot)
    expect(state.pools).toEqual([])
    snapshot.presentationDeaths = {
      epoch: 1, sequence: 2, cues: [cue('old', 512, 512), cue('fresh', 1536, 512)],
    }
    expect(advanceBloodPools(state, snapshot).pools).toEqual([])
  })

  it('bounds the replay fence during a long round', () => {
    const snapshot = withDeaths(gameSnapshot(), 0, [])
    let state = advanceBloodPools(initialBloodPoolState(), snapshot)
    for (let sequence = 1; sequence <= 200; sequence += 1) {
      snapshot.presentationDeaths = {
        epoch: 1, sequence, cues: [cue(`death:${sequence}`, 512 + sequence, 512)],
      }
      state = advanceBloodPools(state, snapshot)
    }
    expect(state.seen.size).toBe(128)
    expect(state.seen.has('death:72')).toBe(false)
    expect(state.seen.has('death:73')).toBe(true)
    expect(state.pools).toMatchObject([{ key: 'blood:0:0', position: { x: 712, y: 512 } }])
  })

  it('accepts a coalesced newer generation after a replacement-room tick reset', () => {
    const snapshot = withDeaths(gameSnapshot(), 0, [])
    snapshot.state.physicalTick = 100
    let state = advanceBloodPools(initialBloodPoolState(), snapshot)
    snapshot.presentationDeaths = { epoch: 1, sequence: 1, cues: [cue('old-room', 512, 512)] }
    state = advanceBloodPools(state, snapshot)
    expect(state.pools.map(({ key }) => key)).toEqual(['blood:0:0'])

    snapshot.room.roomId = 'replacement-room'
    snapshot.state.physicalTick = 1
    snapshot.presentationDeaths = {
      epoch: 2, sequence: 3, cues: [cue('new-room-fresh', 1536, 512)],
    }
    state = advanceBloodPools(state, snapshot)
    expect(state.pools.map(({ key }) => key)).toEqual(['blood:1:0'])

    snapshot.room.roomId = 'replayed-room'
    snapshot.state.physicalTick = 0
    snapshot.presentationDeaths = {
      epoch: 2, sequence: 3, cues: [cue('same-epoch-stale', 2560, 512)],
    }
    expect(advanceBloodPools(state, snapshot).pools).toEqual([])
  })
})
