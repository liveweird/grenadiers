import { expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', () => {
  class Node {
    children: Node[] = []
    parent: Node | null = null
    x = 0
    y = 0
    visible = true
    alpha = 1
    rotation = 0
    cached = false
    scaleValue = 1
    position = { set: (x: number, y: number) => { this.x = x; this.y = y } }
    scale = { set: (value: number) => { this.scaleValue = value } }
    get isCachedAsTexture(): boolean { return this.cached }
    cacheAsTexture(value: false | object): void { this.cached = value !== false }
    updateCacheTexture(): void {}
    addChild(...nodes: Node[]): Node {
      for (const node of nodes) { node.parent = this; this.children.push(node) }
      return nodes[0]
    }
    destroy(options?: { children?: boolean }): void {
      if (options?.children) for (const child of [...this.children]) child.destroy(options)
      if (this.parent) this.parent.children = this.parent.children.filter((child) => child !== this)
    }
  }
  class Graphics extends Node {
    shapes: string[] = []
    clear(): this { this.shapes = []; return this }
    rect(): this { this.shapes.push('rect'); return this }
    roundRect(): this { return this }
    ellipse(): this { return this }
    circle(): this { return this }
    poly(): this { return this }
    fill(): this { return this }
    stroke(): this { return this }
    moveTo(): this { return this }
    lineTo(): this { return this }
  }
  return { Container: Node, Graphics }
})

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
