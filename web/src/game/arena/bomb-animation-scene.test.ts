import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = { children: NodeView[]; scaleValue: number; visible: boolean }
const bombNodes = (scene: ArenaScene): NodeView[] =>
  (scene.root.children[3] as unknown as NodeView).children[0].children

describe('fuse-driven bomb animation', () => {
  it('drives shell and spark from authoritative remaining ticks, not render time', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks: 180, paused: false }
    scene.update(snapshot, 0)
    const [, shell, spark] = bombNodes(scene)
    expect(shell.scaleValue).toBe(1.125)
    expect(spark.visible).toBe(true)

    scene.render(10_000)
    expect(shell.scaleValue).toBe(1.125)
    expect(spark.visible).toBe(true)
    snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks: 195, paused: false }
    scene.update(snapshot, 10_001)
    expect(shell.scaleValue).toBe(1)
    expect(spark.visible).toBe(false)
  })

  it.each(['remote', 'contact'] as const)('keeps %s bombs static and unlit', (type) => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.bombs[0].trigger = { type }
    scene.update(snapshot, 0)
    const [, shell, spark] = bombNodes(scene)
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
    expect(bombNodes(scene)[1].scaleValue).toBe(1)

    scene.setReducedMotion(false)
    snapshot.room.phase = { type: 'result' }
    scene.update(snapshot, 1)
    expect(bombNodes(scene)[1].scaleValue).toBe(1)
  })

  it.each(['piercing', 'power'] as const)('animates a %s shell without replacing it', (blast) => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.bombs[0].blast = blast
    snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks: 60, paused: false }
    scene.update(snapshot, 0)
    const shell = bombNodes(scene)[1]
    scene.render(5_000)
    expect(bombNodes(scene)[1]).toBe(shell)
    expect(shell.scaleValue).toBe(1.125)
  })
})
