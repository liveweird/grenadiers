import { expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = { children: NodeView[]; visible: boolean; alpha: number; shapes: string[] }

it('draws six Slow-foot stack markers and preserves older cues through expiry', () => {
  const scene = new ArenaScene(1)
  const snapshot = gameSnapshot()
  snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'slowFoot' }]
  Object.assign(snapshot.state.actors[0], {
    damageGraceRemainingTicks: 60,
    shieldRemainingTicks: 300,
    goldenFlameRemainingTicks: 480,
    shortFuseRemainingTicks: 240,
    shortFlameRemainingTicks: 360,
    longFuseRemainingTicks: 540,
    reverseControlsRemainingTicks: 420,
    diarrheaRemainingTicks: 300,
    slowFootRemainingTicks: [600, 520, 440, 360, 280, 200],
  })
  scene.update(snapshot, 0)
  const layers = scene.root.children as unknown as NodeView[]
  expect(layers[2].children[0].shapes).toHaveLength(18)
  const actor = layers[5].children[0]
  expect(actor.children.slice(1, 9).every(({ visible }) => visible)).toBe(true)
  expect(actor.children[9]).toMatchObject({ visible: true })
  expect(actor.children[9].children.slice(1).every(({ visible }) => visible)).toBe(true)
  scene.setReducedMotion(true); scene.render(500)
  expect(actor.children[9].children.slice(1).every(({ visible }) => visible)).toBe(true)
  snapshot.room.presentationBaseline = 'slow-foot-resumed'
  snapshot.state.actors[0].slowFootRemainingTicks = []
  scene.update(snapshot, 600)
  const resumedActor = layers[5].children[0]
  expect(resumedActor.children.slice(1, 9).every(({ visible }) => visible)).toBe(true)
  expect(resumedActor.children[9]).toMatchObject({ visible: false })
})
