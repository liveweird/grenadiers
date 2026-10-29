import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = {
  children: NodeView[]; scaleValue: number; visible: boolean; x: number; y: number
  destroyed: boolean; releasedContext: boolean
}
const bombNodes = (scene: ArenaScene): NodeView[] =>
  (scene.root.children[3] as unknown as NodeView).children[0].children
const shellNodes = (scene: ArenaScene): NodeView[] => {
  const nodes = bombNodes(scene)
  return [nodes[1], nodes[4], nodes[5]]
}
function expectShellState(scene: ArenaScene, pose: 0 | 1 | 2, scale: number): void {
  expect(shellNodes(scene).map(({ visible }) => visible))
    .toEqual([pose === 0, pose === 1, pose === 2])
  expect(shellNodes(scene).map(({ scaleValue }) => scaleValue)).toEqual([scale, scale, scale])
}

describe('fuse-driven bomb animation', () => {
  it('drives shell and spark from authoritative remaining ticks, not render time', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks: 180, paused: false }
    scene.update(snapshot, 0)
    const spark = bombNodes(scene)[2]
    expectShellState(scene, 2, 1.125)
    expect(spark.visible).toBe(true)

    scene.render(10_000)
    expectShellState(scene, 2, 1.125)
    expect(spark.visible).toBe(true)
    snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks: 195, paused: false }
    scene.update(snapshot, 10_001)
    expectShellState(scene, 1, 1)
    expect(spark.visible).toBe(false)
  })

  it('selects exactly one of all three drawings from each authoritative phase', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    for (const [ticks, pose, scale] of [[48, 2, 1.125], [51, 1, 1], [54, 0, 1]] as const) {
      snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks: ticks, paused: false }
      scene.update(snapshot, ticks)
      expectShellState(scene, pose, scale)
      expect(shellNodes(scene).filter(({ visible }) => visible)).toHaveLength(1)
    }
  })

  it.each(['remote', 'contact'] as const)('keeps %s bombs static and unlit', (type) => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.bombs[0].trigger = { type }
    scene.update(snapshot, 0)
    const [, shell, spark] = bombNodes(scene)
    expect(bombNodes(scene)).toHaveLength(4)
    expect(shell.scaleValue).toBe(1)
    expect(spark.visible).toBe(false)
    scene.render(10_000)
    expect(shell.scaleValue).toBe(1)
  })

  it('keeps a legacy bomb without trigger metadata neutral', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    delete snapshot.state.bombs[0].trigger
    scene.update(snapshot, 0)
    const [, shell] = bombNodes(scene)
    expectShellState(scene, 0, 1)
    expect(shell.scaleValue).toBe(1)
    scene.render(10_000)
    expect(shell.scaleValue).toBe(1)
  })

  it('freezes the shell while paused and keeps shadow and pause cue independent', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks: 60, paused: true }
    scene.update(snapshot, 0)
    const [shadow, shell, spark, pause] = bombNodes(scene)
    expect(shadow.scaleValue).toBe(1)
    expectShellState(scene, 0, 1)
    expect(shell.scaleValue).toBe(1)
    expect(spark.visible).toBe(false)
    expect(pause.visible).toBe(true)
  })

  it('freezes shell motion outside a running round and for reduced motion', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks: 60, paused: false }
    scene.setReducedMotion(true)
    scene.update(snapshot, 0)
    expectShellState(scene, 0, 1)

    scene.setReducedMotion(false)
    snapshot.room.phase = { type: 'result' }
    scene.update(snapshot, 1)
    expectShellState(scene, 0, 1)
  })

  it.each(['piercing', 'power'] as const)('animates a %s shell without replacing it', (blast) => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.bombs[0].blast = blast
    snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks: 60, paused: false }
    scene.update(snapshot, 0)
    const shells = shellNodes(scene)
    scene.render(5_000)
    expect(shellNodes(scene)).toEqual(shells)
    expectShellState(scene, 2, 1.125)
  })

  it('retains a moving bomb by id and destroys every shell pose on cleanup', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    scene.update(snapshot, 0)
    const bomb = (scene.root.children[3] as unknown as NodeView).children[0]
    const owned = [...bomb.children]
    snapshot.state.bombs[0].cell = { column: 4, row: 5 }
    scene.update(snapshot, 20)
    expect((scene.root.children[3] as unknown as NodeView).children[0]).toBe(bomb)
    expect(bomb).toMatchObject({ x: 216, y: 264 })

    snapshot.state.bombs = []
    scene.update(snapshot, 40)
    expect(bomb.destroyed).toBe(true)
    expect(owned.every(({ destroyed }) => destroyed)).toBe(true)
    expect(owned.every(({ releasedContext }) => releasedContext)).toBe(true)
  })
})
