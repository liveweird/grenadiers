import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = { children: NodeView[]; visible: boolean; x: number; y: number }
const actors = (scene: ArenaScene): NodeView[] =>
  (scene.root.children as unknown as NodeView[])[5].children

describe('Bracelet arena presentation', () => {
  it('keeps static holder gear while ordinary interpolation moves the visible victim', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.actors[0].braceletEquipped = true
    scene.update(snapshot, 0)
    const holder = actors(scene)[0]
    const victim = actors(scene)[1]
    expect(holder.children[13]).toMatchObject({ visible: true })
    expect(holder.children[13].children.map(({ visible }) => visible)).toEqual([false, false, true])

    snapshot.state.physicalTick = 101
    snapshot.state.actors[1].position.x -= 1024
    scene.update(snapshot, 16)
    scene.render(24)
    expect(actors(scene)[1]).toBe(victim)
    expect(victim.x).toBeGreaterThan(snapshot.state.actors[1].position.x * 48 / 1024)
    scene.render(500)
    expect(victim.x).toBe(snapshot.state.actors[1].position.x * 48 / 1024)
    expect(actors(scene)[0]).toBe(holder)
  })

  it('clears replacement gear and removes hidden actors with their root and cues', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.actors[0].braceletEquipped = true
    snapshot.state.actors[1].braceletEquipped = true
    scene.update(snapshot, 0)
    snapshot.state.actors[0].braceletEquipped = false
    scene.update(snapshot, 8)
    expect(actors(scene)[0].children[13]).toMatchObject({ visible: false })
    snapshot.state.actors = [snapshot.state.actors[0]]
    scene.update(snapshot, 16)
    expect(actors(scene)).toHaveLength(1)
  })
})
