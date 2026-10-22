import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = { children: NodeView[]; destroyed: boolean; visible: boolean }

describe('Invisible Bombs retained-scene privacy', () => {
  it('destroys all bomb drawings and reveals only a fresh current bomb row', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    scene.update(snapshot, 0)
    const layer = scene.root.children[3] as unknown as NodeView
    const hiddenRoot = layer.children[0]
    const hiddenChildren = [...hiddenRoot.children]

    snapshot.state.physicalTick = 101
    snapshot.state.bombs = []
    scene.update(snapshot, 20)
    scene.render(200)
    expect(layer.children).toHaveLength(0)
    expect(hiddenRoot.destroyed).toBe(true)
    expect(hiddenChildren.every((child) => child.destroyed)).toBe(true)

    snapshot.state.physicalTick = 102
    snapshot.state.bombs = [{
      id: 30, cell: { column: 4, row: 1 }, owner: 10, range: 2, placedAt: 1,
      trigger: { type: 'timed', remainingTicks: 100, paused: false }, blast: 'standard',
    }]
    scene.update(snapshot, 40)
    expect(layer.children).toHaveLength(1)
    expect(layer.children[0]).not.toBe(hiddenRoot)
  })

  it('renders the distinct pickup without creating a bomb drawing', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.bombs = []
    snapshot.state.pickups = [{ cell: { column: 3, row: 1 }, kind: 'invisibleBombs' }]
    scene.update(snapshot, 0)
    expect((scene.root.children[2] as unknown as NodeView).children).toHaveLength(1)
    expect((scene.root.children[3] as unknown as NodeView).children).toHaveLength(0)
  })
})
