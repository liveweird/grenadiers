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
    scaleValue = 1
    position = { set: (x: number, y: number) => { this.x = x; this.y = y } }
    scale = { set: (value: number) => { this.scaleValue = value } }
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

it('expires Reverse Controls without hiding simultaneous older effect cues', () => {
  const scene = new ArenaScene()
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
