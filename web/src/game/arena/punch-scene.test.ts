import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = { children: NodeView[]; visible: boolean; x: number; y: number }
const actors = (scene: ArenaScene): NodeView[] =>
  (scene.root.children as unknown as NodeView[])[5].children

describe('Punch arena presentation', () => {
  it('shows a holder glove and a fixed 0.2 second cue after a same-ID authoritative jump', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.actors[0].punchEquipped = true
    snapshot.state.bombs[0].cell = { column: 2, row: 1 }
    scene.update(snapshot, 0)
    expect(actors(scene)[0].children[13]).toMatchObject({ visible: true })
    expect(actors(scene)[0].children[13].children[1]).toMatchObject({ visible: true })
    expect(actors(scene)[0].children[13].children[2]).toMatchObject({ visible: false })

    snapshot.state.physicalTick = 101
    snapshot.state.bombs[0].cell = { column: 5, row: 1 }
    scene.update(snapshot, 16)
    expect(actors(scene)[0].children[13].children[2]).toMatchObject({ visible: true })
    scene.render(215)
    expect(actors(scene)[0].children[13].children[2]).toMatchObject({ visible: true })
    scene.render(216)
    expect(actors(scene)[0].children[13].children[2]).toMatchObject({ visible: false })
  })

  it('clears the cue on equipment replacement and removes a cloaked holder', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.actors[0].punchEquipped = true
    snapshot.state.bombs[0].cell = { column: 2, row: 1 }
    scene.update(snapshot, 0)
    snapshot.state.physicalTick = 101
    snapshot.state.bombs[0].cell = { column: 5, row: 1 }
    scene.update(snapshot, 16)

    snapshot.state.actors[0].punchEquipped = false
    snapshot.state.physicalTick = 102
    scene.update(snapshot, 32)
    expect(actors(scene)[0].children[13]).toMatchObject({ visible: false })
    expect(actors(scene)[0].children[13].children[2]).toMatchObject({ visible: false })
    snapshot.state.actors = [snapshot.state.actors[1]]
    scene.update(snapshot, 48)
    expect(actors(scene)).toHaveLength(1)
  })
})
