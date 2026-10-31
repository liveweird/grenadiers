import { describe, expect, it } from 'vitest'
import { gameSnapshot } from '../../../test/game-snapshot'
import { createArenaSceneStore } from './sceneStore'

const flame = (expiresAt = 200) => ({ cell: { column: 3, row: 3 }, expiresAt })

describe('public fire presentation history', () => {
  it('lights existing admission fire without replaying ignition, then latches fresh fire once', () => {
    const store = createArenaSceneStore()
    const snapshot = gameSnapshot()
    snapshot.state.flames = [flame()]
    snapshot.state.oilBarrels = []
    store.update(snapshot, 0)
    expect(store.read().fire?.cells[0].bornAt).toBeNull()
    expect(store.read().fire?.lighting.anchors).toHaveLength(1)
    snapshot.state.physicalTick! += 3
    snapshot.state.flames = [flame(203)]
    store.update(snapshot, 50)
    expect(store.read().fire?.cells[0].bornAt).toBe(50)
    snapshot.state.physicalTick! += 3
    store.update(snapshot, 100)
    expect(store.read().fire?.cells[0].bornAt).toBe(50)
    snapshot.state.flames = []
    store.update(snapshot, 150)
    expect(store.read().fire?.cells).toEqual([])
    expect(store.read().fire?.lighting.anchors).toEqual([])
  })

  it.each(['room', 'round', 'admission', 'backwards', 'gap', 'clock'] as const)(
    'suppresses stale ignition across %s boundaries', (boundary) => {
      const store = createArenaSceneStore()
      const snapshot = gameSnapshot()
      snapshot.state.flames = []; snapshot.state.oilBarrels = []
      store.update(snapshot, 100)
      snapshot.state.flames = [flame()]
      snapshot.state.physicalTick! += 3
      let now = 150
      if (boundary === 'room') snapshot.room.roomId = 'new-room'
      if (boundary === 'round') snapshot.room.roundId += 1
      if (boundary === 'admission') snapshot.room.presentationBaseline = 'new-admission'
      if (boundary === 'backwards') snapshot.state.physicalTick = 0
      if (boundary === 'gap') now = 1_000
      if (boundary === 'clock') now = 50
      store.update(snapshot, now)
      expect(store.read().fire?.cells[0].bornAt).toBeNull()
    },
  )

  it('deduplicates burning Oil and caps emissions, retaining authoritative fire in reduced motion', () => {
    const store = createArenaSceneStore()
    const snapshot = gameSnapshot()
    snapshot.state.flames = []; snapshot.state.oilBarrels = []
    store.update(snapshot, 0)
    snapshot.state.flames = [flame()]
    snapshot.state.oilBarrels = [{ cell: flame().cell, state: 'burning' }]
    snapshot.state.physicalTick! += 3
    store.update(snapshot, 50)
    expect(store.read().fire?.cells).toHaveLength(1)
    expect(store.read().fire?.cells[0]).toMatchObject({ oil: true, bornAt: 50 })
    store.setReducedMotion(true)
    expect(store.read().fire?.cells[0].bornAt).toBeNull()
    expect(store.read().fire?.lighting.anchors).toHaveLength(1)
    store.setReducedMotion(false)
    store.update(snapshot, 100)
    expect(store.read().fire?.cells[0].bornAt).toBeNull()
    store.clear()
    expect(store.read().fire).toBeNull()
    store.update(snapshot, 150)
    expect(store.read().fire?.cells[0].bornAt).toBeNull()
  })

  it('allows a final public blast cue and does not restart it on frozen result snapshots', () => {
    const store = createArenaSceneStore()
    const snapshot = gameSnapshot()
    snapshot.state.flames = []; snapshot.state.oilBarrels = []
    store.update(snapshot, 0)
    snapshot.state.physicalTick! += 3
    snapshot.room.phase = { type: 'matchResult' }
    snapshot.state.flames = [flame()]
    store.update(snapshot, 50)
    expect(store.read().fire?.cells[0].bornAt).toBe(50)
    store.update(snapshot, 100)
    expect(store.read().fire?.cells[0].bornAt).toBe(50)
    snapshot.state.flames = [flame(210)]
    store.update(snapshot, 150)
    expect(store.read().fire?.cells[0].bornAt).toBeNull()
  })

  it('bounds flame input and never infers fire from bombs or hidden actors', () => {
    const store = createArenaSceneStore()
    const snapshot = gameSnapshot()
    snapshot.state.flames = []; snapshot.state.oilBarrels = []
    store.update(snapshot, 0)
    expect(store.read().fire?.cells).toEqual([])
    snapshot.state.flames = Array.from({ length: 220 }, (_, index) => ({
      cell: { column: index % 15, row: Math.floor(index / 15) }, expiresAt: 200,
    }))
    store.update(snapshot, 50)
    expect(store.read().fire?.cells).toHaveLength(195)
  })

  it('releases expired ignition slots so later fire still receives a fresh burst', () => {
    const store = createArenaSceneStore()
    const snapshot = gameSnapshot()
    snapshot.state.flames = []; snapshot.state.oilBarrels = []
    store.update(snapshot, 0)
    snapshot.state.flames = Array.from({ length: 8 }, (_, column) => ({
      cell: { column: column + 1, row: 3 }, expiresAt: 300,
    }))
    snapshot.state.physicalTick! += 3
    store.update(snapshot, 50)
    expect(store.read().fire?.cells.filter(({ bornAt }) => bornAt !== null)).toHaveLength(8)
    snapshot.state.flames.push({ cell: { column: 9, row: 3 }, expiresAt: 310 })
    snapshot.state.physicalTick! += 12
    store.update(snapshot, 250)
    const bursts = store.read().fire!.cells.filter(({ bornAt }) => bornAt !== null)
    expect(bursts).toHaveLength(1)
    expect(bursts[0]).toMatchObject({ column: 9, bornAt: 250 })
  })
})
