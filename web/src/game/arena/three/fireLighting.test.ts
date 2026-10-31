import { describe, expect, it } from 'vitest'
import { gameSnapshot } from '../../../test/game-snapshot'
import type { SnapshotMessage } from '../types'
import { createFireLighting } from './fireLighting'

const cell = (column: number, row: number) => ({ column, row })

function emptySnapshot(): SnapshotMessage {
  const snapshot = gameSnapshot()
  snapshot.state.flames = []
  snapshot.state.oilBarrels = []
  snapshot.state.arena.blocks = []
  return snapshot
}

describe('public fire lighting', () => {
  it('groups contiguous fire and bounds a long ray to four covering anchors', () => {
    const snapshot = emptySnapshot()
    snapshot.state.flames = Array.from({ length: 9 }, (_, offset) => ({
      cell: cell(offset + 2, 5), expiresAt: 130,
    }))

    const model = createFireLighting(snapshot)
    const columns = model.anchors.map(({ position }) => position[0] - 0.5)
    expect(model.anchors).toHaveLength(4)
    expect(columns).toContain(2)
    expect(columns).toContain(10)
    expect(new Set(model.anchors.map(({ id }) => id)).size).toBe(4)
    expect(model.anchors.reduce((sum, anchor) => sum + anchor.intensity, 0)).toBeLessThanOrEqual(2.4)
  })

  it('is deterministic under public row reordering and ignores bombs and expired flames', () => {
    const snapshot = emptySnapshot()
    snapshot.state.flames = [
      { cell: cell(9, 7), expiresAt: 130 },
      { cell: cell(2, 2), expiresAt: 100 },
      { cell: cell(5, 4), expiresAt: 130 },
      { cell: cell(3, 2), expiresAt: 130 },
    ]
    const first = createFireLighting(snapshot)
    snapshot.state.flames.reverse()
    snapshot.state.bombs = [{
      id: 999, cell: cell(5, 5), owner: 20, range: 8, placedAt: 1,
      trigger: { type: 'remote' }, blast: 'power',
    }]
    const second = createFireLighting(snapshot)

    expect(second.anchors).toEqual(first.anchors)
    expect(second.sample(5.5, 4.5)).toBe(first.sample(5.5, 4.5))
    expect(second.anchors.map(({ id }) => id)).not.toContain('fire:32')
  })

  it('treats only burning Oil as persistent public fire and unlit Oil as an occluder', () => {
    const snapshot = emptySnapshot()
    snapshot.state.flames = [{ cell: cell(2, 3), expiresAt: 130 }]
    snapshot.state.oilBarrels = [
      { cell: cell(3, 3), state: 'unlit' },
      { cell: cell(8, 3), state: 'burning' },
    ]
    const model = createFireLighting(snapshot)

    expect(model.anchors.map(({ id }) => id)).toEqual(['fire:47', 'fire:53'])
    expect(model.sample(4.5, 3.5)).toBe(0)
    expect(model.sample(8.5, 3.5)).toBeGreaterThan(0.7)
  })

  it.each([
    ['hard wall', (snapshot: SnapshotMessage) => {
      snapshot.state.arena.terrain[3 + 3 * snapshot.state.arena.width] = 'wall'
    }],
    ['lair', (snapshot: SnapshotMessage) => {
      snapshot.state.lairs = [{ id: 1, cell: cell(3, 3), spawnRemainingTicks: null, livingMobs: 0 }]
    }],
  ] as const)('occludes through a public %s', (_name, addBlocker) => {
    const snapshot = emptySnapshot()
    snapshot.state.flames = [{ cell: cell(2, 3), expiresAt: 130 }]
    addBlocker(snapshot)
    expect(createFireLighting(snapshot).sample(4.5, 3.5)).toBe(0)
  })

  it('recomputes occlusion after a soft block is publicly destroyed', () => {
    const snapshot = emptySnapshot()
    snapshot.state.flames = [{ cell: cell(2, 3), expiresAt: 130 }]
    snapshot.state.arena.blocks = [cell(3, 3)]
    expect(createFireLighting(snapshot).sample(4.25, 3.5)).toBe(0)

    snapshot.state.arena.blocks = []
    expect(createFireLighting(snapshot).sample(4.25, 3.5)).toBeGreaterThan(0)
  })

  it('caps overlapping local warmth and rejects non-finite or off-board samples', () => {
    const snapshot = emptySnapshot()
    snapshot.state.flames = [cell(5, 5), cell(6, 5), cell(5, 6), cell(6, 6)]
      .map((fireCell) => ({ cell: fireCell, expiresAt: 130 }))
    const model = createFireLighting(snapshot)
    expect(model.sample(6, 6)).toBeGreaterThan(0)
    expect(model.sample(6, 6)).toBeLessThanOrEqual(1)
    expect(model.sample(Number.NaN, 1)).toBe(0)
    expect(model.sample(-1, 1)).toBe(0)
  })
})
