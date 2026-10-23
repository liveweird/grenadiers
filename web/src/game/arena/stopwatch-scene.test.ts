import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = { children: NodeView[]; x: number; visible: boolean; alpha: number; shapes: string[] }
const layers = (scene: ArenaScene): NodeView[] => scene.root.children as unknown as NodeView[]

describe('Stopwatch arena presentation', () => {
  it('draws holder, frozen victim, and snaps the victim on the authoritative freeze edge', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'stopwatch' }]
    snapshot.state.actors[0].stopwatchRemainingTicks = 180
    scene.update(snapshot, 0)
    const [, , pickups, , , actors] = layers(scene)
    expect(pickups.children[0].shapes.length).toBeGreaterThan(12)
    expect(actors.children[0].children[22]).toMatchObject({ visible: true, alpha: 0.7 })

    snapshot.state.physicalTick = (snapshot.state.physicalTick ?? 0) + 1
    snapshot.state.actors[1].position.x -= 1024
    snapshot.state.actors[1].voluntaryActionsFrozen = true
    scene.update(snapshot, 20)
    const victim = actors.children[1]
    expect(victim.x).toBe(600)
    expect(victim.children[23]).toMatchObject({ visible: true })
    scene.render(21)
    expect(victim.x).toBe(600)
  })

  it('shows a stable pause mark and never re-enables a paused timed-bomb spark', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks: 179, paused: true }
    scene.update(snapshot, 0)
    const bomb = layers(scene)[3].children[0]
    expect(bomb.children[2]).toMatchObject({ visible: false })
    expect(bomb.children[3]).toMatchObject({ visible: true })
    scene.render(90)
    expect(bomb.children[2]).toMatchObject({ visible: false })
    snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks: 178, paused: false }
    scene.update(snapshot, 100)
    expect(bomb.children[3]).toMatchObject({ visible: false })
    scene.render(190)
    const firstFrame = bomb.children[2].visible
    scene.render(280)
    expect(bomb.children[2].visible).not.toBe(firstFrame)
  })
})
