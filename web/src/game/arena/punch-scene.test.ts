import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = { children: NodeView[]; visible: boolean }
const actors = (scene: ArenaScene): NodeView[] =>
  (scene.root.children as unknown as NodeView[])[5].children

describe('Punch arena presentation', () => {
  it('keeps the holder glove static when an ordinary same-ID bomb movement arrives', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.actors[0].punchEquipped = true
    snapshot.state.bombs[0].cell = { column: 2, row: 1 }
    scene.update(snapshot, 0)
    const equipment = actors(scene)[0].children[13]
    expect(equipment).toMatchObject({ visible: true })
    expect(equipment.children).toHaveLength(3)
    expect(equipment.children.map(({ visible }) => visible)).toEqual([false, true, false])

    snapshot.state.physicalTick = 101
    snapshot.state.bombs[0].cell = { column: 5, row: 1 }
    scene.update(snapshot, 16)
    scene.render(500)
    expect(actors(scene)[0].children[13]).toBe(equipment)
    expect(equipment.children.map(({ visible }) => visible)).toEqual([false, true, false])
  })

  it('clears the static glove on replacement and removes a cloaked holder', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.actors[0].punchEquipped = true
    scene.update(snapshot, 0)
    snapshot.state.actors[0].punchEquipped = false
    snapshot.state.physicalTick = 101
    scene.update(snapshot, 16)
    expect(actors(scene)[0].children[13]).toMatchObject({ visible: false })

    snapshot.state.actors = [snapshot.state.actors[1]]
    scene.update(snapshot, 32)
    expect(actors(scene)).toHaveLength(1)
  })
})
