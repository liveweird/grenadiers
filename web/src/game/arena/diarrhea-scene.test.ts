import { expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = { children: NodeView[]; visible: boolean; alpha: number; shapes: string[] }

it('draws a bomb-drop Diarrhea cue and preserves older cues through expiry', () => {
  const scene = new ArenaScene(1)
  const snapshot = gameSnapshot()
  snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'diarrhea' }]
  Object.assign(snapshot.state.actors[0], {
    damageGraceRemainingTicks: 60,
    shieldRemainingTicks: 300,
    goldenFlameRemainingTicks: 480,
    shortFuseRemainingTicks: 240,
    shortFlameRemainingTicks: 360,
    longFuseRemainingTicks: 540,
    reverseControlsRemainingTicks: 420,
    diarrheaRemainingTicks: 600,
  })
  scene.update(snapshot, 0)
  const layers = scene.root.children as unknown as NodeView[]
  expect(layers[2].children[0].shapes).toHaveLength(20)
  expect(layers[5].children[0].children.slice(1, 9)).toMatchObject([
    { visible: true }, { visible: true }, { visible: true }, { visible: true },
    { visible: true }, { visible: true }, { visible: true },
    { visible: true, alpha: 0.65 },
  ])
  scene.setReducedMotion(true); scene.render(500)
  expect(layers[5].children[0].children[8]).toMatchObject({ visible: true, alpha: 1 })
  snapshot.room.presentationBaseline = 'diarrhea-resumed'
  snapshot.state.actors[0].diarrheaRemainingTicks = 0
  scene.update(snapshot, 600)
  expect(layers[5].children[0].children.slice(1, 8).every(({ visible }) => visible)).toBe(true)
  expect(layers[5].children[0].children[8]).toMatchObject({ visible: false })
})
