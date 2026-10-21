import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = {
  children: NodeView[]
  destroyed: boolean
  visible: boolean
  x: number
  y: number
}
const flames = (scene: ArenaScene): NodeView =>
  (scene.root.children as unknown as NodeView[])[4]

describe('Oil Barrel arena presentation', () => {
  it('retains one keyed root while unlit Oil becomes a charred persistent flame', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.oilBarrels = [{ cell: { column: 3, row: 1 }, state: 'unlit' }]
    snapshot.state.flames = []
    scene.update(snapshot, 0)
    const barrel = flames(scene).children[0]
    expect(barrel).toMatchObject({ x: 144, y: 48 })
    expect(barrel.children.map(({ visible }) => visible)).toEqual([true, false, false])

    snapshot.state.physicalTick = 101
    snapshot.state.oilBarrels[0].state = 'burning'
    snapshot.state.flames = [{ cell: { column: 3, row: 1 }, expiresAt: 102 }]
    scene.update(snapshot, 16)
    expect(flames(scene).children).toEqual([barrel])
    expect(barrel.children.map(({ visible }) => visible)).toEqual([false, true, true])
    expect(barrel.children[2].children.map(({ visible }) => visible)).toEqual([true, false])

    snapshot.state.physicalTick = 102
    snapshot.state.flames = []
    scene.update(snapshot, 32)
    expect(flames(scene).children).toEqual([barrel])
    expect(barrel.children[2]).toMatchObject({ visible: true })
  })

  it('freezes decoration in terminal and reduced modes and disposes it at a new round', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.oilBarrels = [{ cell: { column: 3, row: 1 }, state: 'burning' }]
    snapshot.state.flames = []
    scene.update(snapshot, 0)
    const barrel = flames(scene).children[0]
    scene.render(80)
    expect(barrel.children[2].children.map(({ visible }) => visible)).toEqual([false, true])
    scene.setReducedMotion(true)
    scene.render(160)
    expect(barrel.children[2].children.map(({ visible }) => visible)).toEqual([true, false])
    scene.setReducedMotion(false)
    snapshot.room.phase = { type: 'result' }
    scene.update(snapshot, 240)
    scene.render(320)
    expect(barrel.children[2].children.map(({ visible }) => visible)).toEqual([true, false])

    snapshot.room.roundId += 1
    snapshot.room.phase = { type: 'running' }
    snapshot.state.oilBarrels = []
    scene.update(snapshot, 400)
    expect(barrel.destroyed).toBe(true)
    expect(flames(scene).children).toHaveLength(0)
  })

  it('rebuilds retained Oil from the current state after a reconnect baseline change', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.oilBarrels = [{ cell: { column: 3, row: 1 }, state: 'burning' }]
    snapshot.state.flames = []
    scene.update(snapshot, 0)
    const old = flames(scene).children[0]
    snapshot.room.presentationBaseline = 'resume'
    scene.update(snapshot, 10)
    expect(old.destroyed).toBe(true)
    expect(flames(scene).children).toHaveLength(1)
    expect(flames(scene).children[0]).not.toBe(old)
    expect(flames(scene).children[0].children.map(({ visible }) => visible))
      .toEqual([false, true, true])
  })
})
