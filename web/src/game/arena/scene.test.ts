import { describe, expect, it, vi } from 'vitest'
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
    destroyed = false
    releasedContext = false
    scaleValue = 1
    position = { set: (x: number, y: number) => { this.x = x; this.y = y } }
    scale = { set: (value: number) => { this.scaleValue = value } }
    addChild(...nodes: Node[]): Node {
      for (const node of nodes) { node.parent = this; this.children.push(node) }
      return nodes[0]
    }
    destroy(options?: { children?: boolean; context?: boolean }): void {
      this.destroyed = true
      if (options?.children) for (const child of [...this.children]) child.destroy(options)
      if (this.parent) this.parent.children = this.parent.children.filter((child) => child !== this)
    }
  }
  class Graphics extends Node {
    shapes: string[] = []
    override destroy(options?: { children?: boolean; context?: boolean }): void {
      this.releasedContext = !options || options.context === true
      super.destroy(options)
    }
    clear(): this { this.shapes = []; return this }
    rect(): this { this.shapes.push('rect'); return this }
    roundRect(): this { this.shapes.push('roundRect'); return this }
    ellipse(): this { this.shapes.push('ellipse'); return this }
    circle(): this { this.shapes.push('circle'); return this }
    poly(): this { this.shapes.push('poly'); return this }
    fill(): this { return this }
    stroke(): this { return this }
    moveTo(): this { return this }
    lineTo(): this { return this }
  }
  return { Container: Node, Graphics }
})

type NodeView = {
  children: NodeView[]; x: number; y: number; visible: boolean; alpha: number
  rotation: number; destroyed: boolean; releasedContext: boolean; scaleValue: number; shapes: string[]
}
function layers(scene: ArenaScene): NodeView[] {
  return scene.root.children as unknown as NodeView[]
}

describe('retained public arena scene', () => {
  it('draws every public layer and retains unchanged objects between snapshots', () => {
    const scene = new ArenaScene()
    const snapshot = gameSnapshot()
    snapshot.state.pickups = [
      { cell: { column: 2, row: 2 }, kind: 'bombCapacity' },
      { cell: { column: 3, row: 2 }, kind: 'blastRange' },
      { cell: { column: 4, row: 2 }, kind: 'movementSpeed' },
    ]
    scene.update(snapshot, 0)
    const [terrain, blocks, pickups, bombs, flames, actors] = layers(scene)
    expect(terrain.shapes.filter((shape) => shape === 'rect').length).toBeGreaterThan(195)
    expect(blocks.children).toHaveLength(1)
    expect(pickups.children).toHaveLength(3)
    expect(bombs.children).toHaveLength(1)
    expect(flames.children).toHaveLength(1)
    expect(actors.children).toHaveLength(2)
    expect(actors.children.map(({ x, y }) => [x, y])).toEqual([[72, 72], [648, 552]])
    expect(bombs.children[0]).toMatchObject({ x: 72, y: 72 })
    expect(pickups.children.map(({ x, y }) => [x, y])).toEqual([[96, 96], [144, 96], [192, 96]])
    expect(pickups.children.map(({ shapes }) => shapes)).toEqual([
      ['ellipse', 'roundRect', 'circle', 'circle'],
      ['ellipse', 'roundRect', 'roundRect', 'roundRect', 'circle'],
      ['ellipse', 'roundRect', 'poly', 'poly'],
    ])
    const retained = [
      blocks.children[0], ...pickups.children, bombs.children[0], flames.children[0], ...actors.children,
    ]
    scene.update(snapshot, 50)
    expect([
      blocks.children[0], ...pickups.children, bombs.children[0], flames.children[0], ...actors.children,
    ]).toEqual(retained)
    const oneHelmet = actors.children[0].children[1]
    const twoHelmet = actors.children[1].children[1]
    expect(oneHelmet.shapes).not.toContain('poly')
    expect(twoHelmet.shapes).toContain('poly')
    expect(oneHelmet.rotation).toBe(-Math.PI / 2)
    expect(twoHelmet.rotation).toBe(Math.PI / 2)
  })

  it('interpolates only between public positions and never extrapolates past the target', () => {
    const scene = new ArenaScene()
    const snapshot = gameSnapshot()
    scene.update(snapshot, 0)
    snapshot.room.tick += 3
    snapshot.room.revision += 1
    snapshot.state.actors[0].position.x += 1024
    scene.update(snapshot, 100)
    const actor = layers(scene)[5].children[0]
    expect(actor.x).toBe(72)
    scene.render(125)
    expect(actor.x).toBe(96)
    scene.render(150)
    expect(actor.x).toBe(120)
    scene.render(100_000)
    expect(actor.x).toBe(120)
    scene.render(90)
    expect(actor.x).toBe(72)
  })

  it('removes absent public entities and replaces a changed pickup kind at the same cell', () => {
    const scene = new ArenaScene()
    const snapshot = gameSnapshot()
    snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'bombCapacity' }]
    scene.update(snapshot, 0)
    const [, blocks, pickups, bombs, flames, actors] = layers(scene)
    const firstPickup = pickups.children[0]
    snapshot.state.pickups[0].kind = 'movementSpeed'
    scene.update(snapshot, 25)
    expect(firstPickup.destroyed).toBe(true)
    expect(pickups.children[0].shapes).toEqual(['ellipse', 'roundRect', 'poly', 'poly'])
    const removed = [blocks.children[0], pickups.children[0], bombs.children[0], flames.children[0], actors.children[0]]
    const ownedGraphics = [...bombs.children[0].children, ...actors.children[0].children]
    snapshot.state.arena.blocks = []
    snapshot.state.pickups = []
    snapshot.state.bombs = []
    snapshot.state.flames = []
    snapshot.state.actors[0].id = 11
    scene.update(snapshot, 50)
    expect(removed.every((drawing) => drawing.destroyed)).toBe(true)
    expect(ownedGraphics.every((drawing) => drawing.releasedContext)).toBe(true)
    expect([blocks.children.length, pickups.children.length, bombs.children.length, flames.children.length, actors.children.length])
      .toEqual([0, 0, 0, 0, 2])
  })

  it('snaps terminal states, hides eliminated actors, and freezes decorative effects', () => {
    const scene = new ArenaScene()
    const snapshot = gameSnapshot()
    scene.update(snapshot, 0)
    scene.render(25)
    const [, , , bombs, flames, actors] = layers(scene)
    expect(bombs.children[0].scaleValue).not.toBe(1)
    snapshot.room.phase = { type: 'result', endedAt: 101, result: { type: 'winner', seat: 'one' } }
    snapshot.state.actors[0].position.x += 1024
    snapshot.state.actors[1].alive = false
    scene.update(snapshot, 50)
    expect(actors.children[0].x).toBe(120)
    expect(actors.children[1].visible).toBe(false)
    expect(bombs.children[0].scaleValue).toBe(1)
    expect(flames.children[0].alpha).toBe(1)
  })

  it('respects reduced effects and resets interpolation across room/round identities', () => {
    const scene = new ArenaScene()
    const snapshot = gameSnapshot()
    snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'blastRange' }]
    scene.update(snapshot, 0)
    scene.setReducedMotion(true)
    snapshot.state.actors[0].position.y += 1024
    scene.update(snapshot, 50)
    const oldActors = [...layers(scene)[5].children]
    const oldPickup = layers(scene)[2].children[0]
    expect(oldActors[0].y).toBe(120)
    expect(layers(scene)[3].children[0].scaleValue).toBe(1)
    scene.setReducedMotion(false)
    snapshot.room.roundId += 1
    snapshot.state.actors[0].position.x = 1536
    snapshot.state.actors[0].heading = 'north'
    scene.update(snapshot, 100)
    expect(oldActors.every((actor) => actor.destroyed)).toBe(true)
    expect(oldPickup.destroyed).toBe(true)
    expect(layers(scene)[2].children[0]).not.toBe(oldPickup)
    expect(layers(scene)[5].children[0]).toMatchObject({ x: 72, y: 120 })
    expect(layers(scene)[5].children[0].children[1].rotation).toBe(Math.PI)
  })
})
