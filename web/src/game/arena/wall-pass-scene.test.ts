import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = {
  children: NodeView[]
  visible: boolean
  alpha: number
  shapes: string[]
}

const layers = (scene: ArenaScene): NodeView[] => scene.root.children as unknown as NodeView[]

describe('Wall Pass arena presentation', () => {
  it('keeps blocks opaque while showing the cue only on the current holder', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'wallPass' }]
    snapshot.state.actors[0].wallPassEquipped = true
    snapshot.state.actors[0].position = { x: 2560, y: 2560 }
    snapshot.state.arena.blocks = [{ column: 2, row: 2 }]
    scene.update(snapshot, 0)
    const [, blocks, pickups, , , actors] = layers(scene)
    expect(blocks.children[0]).toMatchObject({ visible: true, alpha: 1 })
    expect(pickups.children[0].shapes).toHaveLength(19)
    expect(actors.children[0].children[24]).toMatchObject({ visible: true, alpha: 1 })
    expect(layers(scene).indexOf(blocks)).toBeLessThan(layers(scene).indexOf(actors))
    scene.setReducedMotion(true)
    scene.render(500)
    expect(actors.children[0].children[24]).toMatchObject({ visible: true, alpha: 1 })

    snapshot.state.actors[0].wallPassEquipped = false
    snapshot.room.presentationBaseline = 'heart-replaced-wall-pass'
    scene.update(snapshot, 100)
    expect(blocks.children[0]).toMatchObject({ visible: true, alpha: 1 })
    expect(actors.children[0].children[24]).toMatchObject({ visible: false })
  })
})
