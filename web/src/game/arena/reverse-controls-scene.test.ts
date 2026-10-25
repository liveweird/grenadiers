import { expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = { children: NodeView[]; visible: boolean; alpha: number; shapes: string[] }

it('expires Reverse Controls without hiding simultaneous older effect cues', () => {
  const scene = new ArenaScene(1)
  const snapshot = gameSnapshot()
  snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'reverseControls' }]
  Object.assign(snapshot.state.actors[0], {
    damageGraceRemainingTicks: 60,
    shieldRemainingTicks: 300,
    goldenFlameRemainingTicks: 480,
    shortFuseRemainingTicks: 240,
    shortFlameRemainingTicks: 360,
    longFuseRemainingTicks: 540,
    reverseControlsRemainingTicks: 600,
  })
  scene.update(snapshot, 0)
  const layers = scene.root.children as unknown as NodeView[]
  expect(layers[2].children[0].shapes).toHaveLength(21)
  expect(layers[5].children[0].children.slice(1, 8)).toMatchObject([
    { visible: true }, { visible: true }, { visible: true }, { visible: true },
    { visible: true }, { visible: true }, { visible: true, alpha: 0.7 },
  ])
  scene.setReducedMotion(true); scene.render(500)
  expect(layers[5].children[0].children[7]).toMatchObject({ visible: true, alpha: 1 })
  snapshot.room.presentationBaseline = 'reverse-resumed'
  snapshot.state.actors[0].reverseControlsRemainingTicks = 0
  scene.update(snapshot, 600)
  expect(layers[5].children[0].children.slice(1, 7).every(({ visible }) => visible)).toBe(true)
  expect(layers[5].children[0].children[7]).toMatchObject({ visible: false })
})
