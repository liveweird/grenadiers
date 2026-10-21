import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = { children: NodeView[]; x: number; y: number; visible: boolean }
const layers = (scene: ArenaScene): NodeView[] => scene.root.children as unknown as NodeView[]

describe('Kick arena presentation', () => {
  it('moves a retained bomb root directly to its latest public cell', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    scene.update(snapshot, 0)
    const bomb = layers(scene)[3].children[0]
    snapshot.state.physicalTick = (snapshot.state.physicalTick ?? 0) + 1
    snapshot.state.bombs[0].cell = { column: 3, row: 4 }
    scene.update(snapshot, 16)
    expect(layers(scene)[3].children[0]).toBe(bomb)
    expect(bomb).toMatchObject({ x: 168, y: 216 })
    scene.render(17)
    expect(bomb).toMatchObject({ x: 168, y: 216 })
  })

  it('shows the Kick cue only on the current holder', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.actors[0].kickEquipped = true
    scene.update(snapshot, 0)
    const actors = layers(scene)[5]
    expect(actors.children[0].children[13]).toMatchObject({ visible: true })
    expect(actors.children[1].children[13]).toMatchObject({ visible: false })
    snapshot.state.actors[0].kickEquipped = false
    scene.update(snapshot, 16)
    expect(actors.children[0].children[13]).toMatchObject({ visible: false })
  })
})
