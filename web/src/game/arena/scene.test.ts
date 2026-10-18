import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'
vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())
type NodeView = {
  children: NodeView[]; x: number; y: number; visible: boolean; alpha: number
  visibleChanges: number; rotation: number; destroyed: boolean
  releasedContext: boolean; scaleValue: number; shapes: string[]
  cacheCalls: Array<false | object>; cacheUpdates: number; cached: boolean
}
function layers(scene: ArenaScene): NodeView[] {
  return scene.root.children as unknown as NodeView[]
}
describe('retained public arena scene', () => {
  it('draws every public layer and retains unchanged objects between snapshots', () => {
    const scene = new ArenaScene(2)
    const snapshot = gameSnapshot()
    snapshot.state.actors[0].piercingArmed = true
    snapshot.state.bombs[0].blast = 'piercing'
    snapshot.state.pickups = [
      { cell: { column: 2, row: 2 }, kind: 'bombCapacity' },
      { cell: { column: 3, row: 2 }, kind: 'blastRange' },
      { cell: { column: 4, row: 2 }, kind: 'movementSpeed' },
    ]
    scene.update(snapshot, 0)
    const [terrain, blocks, pickups, bombs, flames, actors] = layers(scene)
    expect(terrain.cacheCalls).toEqual([{ resolution: 2, antialias: false, scaleMode: 'nearest' }])
    expect(terrain.cacheUpdates).toBe(1)
    expect(terrain.shapes.filter((shape) => shape === 'rect').length).toBeGreaterThan(195)
    expect(blocks.children).toHaveLength(1)
    expect(pickups.children).toHaveLength(3)
    expect(bombs.children).toHaveLength(1)
    expect(flames.children).toHaveLength(1)
    expect(actors.children).toHaveLength(2)
    expect(actors.children.map(({ x, y }) => [x, y])).toEqual([[72, 72], [648, 552]])
    expect(bombs.children[0]).toMatchObject({ x: 72, y: 72 })
    expect(bombs.children[0].children[1].shapes).toHaveLength(14)
    expect(pickups.children.map(({ x, y }) => [x, y])).toEqual([[96, 96], [144, 96], [192, 96]])
    expect(pickups.children.map(({ shapes }) => shapes.every((shape) => shape === 'rect')))
      .toEqual([true, true, true])
    expect(pickups.children.map(({ shapes }) => shapes.length)).toEqual([16, 14, 18])
    const retained = [
      blocks.children[0], ...pickups.children, bombs.children[0], flames.children[0], ...actors.children,
    ]
    scene.update(snapshot, 50)
    expect(terrain.cacheUpdates).toBe(1)
    expect([
      blocks.children[0], ...pickups.children, bombs.children[0], flames.children[0], ...actors.children,
    ]).toEqual(retained)
    expect(actors.children.map((actor) => actor.children.length)).toEqual([19, 19])
    expect(actors.children[0].children.map(({ visible }) => visible))
      .toEqual([true, false, false, false, false, false, false, false, false, false, true, false, false, true, false, false, false, false, false])
    expect(actors.children[1].children.map(({ visible }) => visible))
      .toEqual([true, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, true, false])
  })
  it('interpolates only between public positions and never extrapolates past the target', () => {
    const scene = new ArenaScene(1); const snapshot = gameSnapshot()
    scene.update(snapshot, 0)
    snapshot.state.physicalTick = (snapshot.state.physicalTick ?? 0) + 3
    snapshot.state.actors[0].position.x += 1024
    scene.update(snapshot, 100)
    const actor = layers(scene)[5].children[0]
    expect(actor.x).toBe(72)
    scene.render(125)
    expect(actor.x).toBe(96)
    expect(actor.children.map(({ visible }) => visible))
      .toEqual([true, false, false, false, false, false, false, false, false, false, false, false, false, false, true, false, false, false, false])
    scene.render(150)
    expect(actor.x).toBe(120)
    scene.render(100_000)
    expect(actor.x).toBe(120)
    scene.render(90)
    expect(actor.x).toBe(72)
    scene.update(snapshot, 200)
    scene.render(325)
    expect(actor.children.map(({ visible }) => visible))
      .toEqual([true, false, false, false, false, false, false, false, false, false, false, false, false, true, false, false, false, false, false])
  })
  it('keeps a selected frame stable when another moving snapshot arrives in that frame', () => {
    const scene = new ArenaScene(1); const snapshot = gameSnapshot()
    Object.assign(snapshot.state.actors[0], {
      damageGraceRemainingTicks: 60,
      shieldRemainingTicks: 300,
      goldenFlameRemainingTicks: 600,
      shortFuseRemainingTicks: 600,
      shortFlameRemainingTicks: 600,
    })
    scene.update(snapshot, 0)
    snapshot.state.physicalTick = (snapshot.state.physicalTick ?? 0) + 3
    snapshot.state.actors[0].position.x += 1024
    scene.update(snapshot, 100)
    scene.render(120)
    const actor = layers(scene)[5].children[0]
    const cues = actor.children.slice(1, 6)
    const poses = actor.children.slice(6)
    for (const cue of cues) cue.visibleChanges = 0
    for (const pose of poses) pose.visibleChanges = 0
    snapshot.state.physicalTick = (snapshot.state.physicalTick ?? 0) + 1
    snapshot.state.actors[0].position.x += 1024
    scene.update(snapshot, 120)
    expect(poses.reduce((total, pose) => total + pose.visibleChanges, 0)).toBe(0)
    expect(poses.filter(({ visible }) => visible)).toHaveLength(1)
    expect(cues.reduce((total, cue) => total + cue.visibleChanges, 0)).toBe(0)
  })
  it('removes absent public entities and replaces a changed pickup kind at the same cell', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'bombCapacity' }]
    scene.update(snapshot, 0)
    const [, blocks, pickups, bombs, flames, actors] = layers(scene)
    const firstPickup = pickups.children[0]
    snapshot.state.pickups[0].kind = 'movementSpeed'
    scene.update(snapshot, 25)
    expect(firstPickup.destroyed).toBe(true)
    expect(pickups.children[0].shapes).toHaveLength(18)
    expect(pickups.children[0].shapes.every((shape) => shape === 'rect')).toBe(true)
    const removed = [blocks.children[0], pickups.children[0], bombs.children[0], flames.children[0], actors.children[0]]
    const ownedGraphics = [...bombs.children[0].children, ...actors.children[0].children]
      .flatMap((drawing) => [drawing, ...drawing.children])
      .filter((drawing) => Array.isArray(drawing.shapes))
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
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    scene.update(snapshot, 0)
    scene.render(25)
    const [, , , bombs, flames, actors] = layers(scene)
    const spark = bombs.children[0].children[2]
    const firstSparkFrame = spark.visible
    scene.render(115)
    expect(spark.visible).not.toBe(firstSparkFrame)
    snapshot.room.phase = { type: 'result' }
    snapshot.state.actors[0].position.x += 1024
    snapshot.state.actors[1].alive = false
    scene.update(snapshot, 50)
    expect(actors.children[0].x).toBe(120)
    expect(actors.children[1].visible).toBe(false)
    expect(spark.visible).toBe(true)
    expect(flames.children[0].children.map(({ visible }) => visible)).toEqual([true, false])
  })
})
describe('arena scene transitions and optional-item presentation', () => {
  it('respects reduced effects and resets interpolation across room/round identities', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'blastRange' }]
    scene.update(snapshot, 0)
    scene.setReducedMotion(true)
    snapshot.state.actors[0].position.y += 1024
    scene.update(snapshot, 50)
    const oldActors = [...layers(scene)[5].children]
    const oldPickup = layers(scene)[2].children[0]
    expect(oldActors[0].y).toBe(120)
    expect(layers(scene)[3].children[0].children[2].visible).toBe(true)
    scene.setReducedMotion(false)
    snapshot.room.roundId += 1
    snapshot.state.actors[0].position.x = 1536
    snapshot.state.actors[0].heading = 'north'
    scene.update(snapshot, 100)
    expect(layers(scene)[0].cacheCalls.slice(-2)).toEqual([
      false, { resolution: 1, antialias: false, scaleMode: 'nearest' },
    ])
    expect(layers(scene)[0].cacheUpdates).toBe(2)
    expect(oldActors.every((actor) => actor.destroyed)).toBe(true)
    expect(oldPickup.destroyed).toBe(true)
    expect(layers(scene)[2].children[0]).not.toBe(oldPickup)
    expect(layers(scene)[5].children[0]).toMatchObject({ x: 72, y: 120 })
    expect(layers(scene)[5].children[0].children.map(({ visible }) => visible))
      .toEqual([true, false, false, false, false, false, false, false, false, false, false, true, false, false, false, false, false, false, false])
  })
  it('deletes hidden actor motion and snaps a later reveal to its current position', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    scene.update(snapshot, 0)
    const actors = layers(scene)[5]
    const removed = actors.children[1]
    snapshot.state.physicalTick = 2
    snapshot.state.actors = [snapshot.state.actors[0]]
    scene.update(snapshot, 20)
    expect(removed.destroyed).toBe(true)
    expect(actors.children).toHaveLength(1)
    snapshot.state.physicalTick = 3
    snapshot.state.actors.push({
      id: 20, seat: 'two', position: { x: 4096, y: 3072 }, heading: 'north', alive: true,
    })
    scene.update(snapshot, 40)
    expect(actors.children).toHaveLength(2)
    expect(actors.children[1]).toMatchObject({ x: 192, y: 144 })
    scene.render(41)
    expect(actors.children[1]).toMatchObject({ x: 192, y: 144 })
  })
})
describe('optional-item presentation', () => {
  it('draws Heart pickups and current damage grace without replaying a reconnect transition', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'heart' }]
    snapshot.state.actors[0].damageGraceRemainingTicks = 60
    scene.update(snapshot, 0)
    const [, , pickups, , , actors] = layers(scene)
    expect(pickups.children[0].shapes).toHaveLength(23)
    expect(actors.children[0].children[1]).toMatchObject({ visible: true, alpha: 0.65 })
    scene.setReducedMotion(true)
    scene.render(500)
    expect(actors.children[0].children[1]).toMatchObject({ visible: true, alpha: 1 })
    snapshot.room.presentationBaseline = 'resumed'
    scene.update(snapshot, 600)
    expect(actors.children[0].children[1]).toMatchObject({ visible: true, alpha: 1 })
  })
  it('draws a Shield glyph and a distinct current aura that stays static with reduced motion', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'shield' }]
    snapshot.state.actors[0].shieldRemainingTicks = 300
    scene.update(snapshot, 0); const [, , pickups, , , actors] = layers(scene)
    expect(pickups.children[0].shapes).toHaveLength(21)
    expect(actors.children[0].children[1]).toMatchObject({ visible: false })
    expect(actors.children[0].children[2]).toMatchObject({ visible: true, alpha: 0.75 })
    scene.setReducedMotion(true); scene.render(500)
    expect(actors.children[0].children[2]).toMatchObject({ visible: true, alpha: 1 })
    snapshot.room.presentationBaseline = 'resumed-with-shield'; scene.update(snapshot, 600)
    expect(actors.children[0].children[2]).toMatchObject({ visible: true, alpha: 1 })
  })
  it('draws Golden Flame as a distinct crest that coexists with Shield and Heart grace', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'goldenFlame' }]
    snapshot.state.actors[0].damageGraceRemainingTicks = 60
    snapshot.state.actors[0].shieldRemainingTicks = 300
    snapshot.state.actors[0].goldenFlameRemainingTicks = 600
    scene.update(snapshot, 0); const [, , pickups, , , actors] = layers(scene)
    expect(pickups.children[0].shapes).toHaveLength(22)
    expect(actors.children[0].children.slice(1, 4))
      .toMatchObject([{ visible: true }, { visible: true }, { visible: true, alpha: 0.7 }])
    scene.setReducedMotion(true); scene.render(500)
    expect(actors.children[0].children.slice(1, 4))
      .toMatchObject([{ alpha: 1 }, { alpha: 1 }, { alpha: 1 }])
    snapshot.room.presentationBaseline = 'resumed-with-golden-flame'; scene.update(snapshot, 600)
    expect(actors.children[0].children[3]).toMatchObject({ visible: true, alpha: 1 })
  })
  it('draws a distinct Short Fuse spark that coexists and stays static after reconnect', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'shortFuse' }]
    snapshot.state.actors[0].damageGraceRemainingTicks = 60
    snapshot.state.actors[0].shieldRemainingTicks = 300
    snapshot.state.actors[0].goldenFlameRemainingTicks = 600
    snapshot.state.actors[0].shortFuseRemainingTicks = 600
    scene.update(snapshot, 0); const [, , pickups, , , actors] = layers(scene)
    expect(pickups.children[0].shapes).toHaveLength(21)
    expect(actors.children[0].children.slice(1, 5)).toMatchObject([
      { visible: true }, { visible: true }, { visible: true }, { visible: true, alpha: 0.65 },
    ])
    scene.setReducedMotion(true); scene.render(500)
    expect(actors.children[0].children.slice(1, 5))
      .toMatchObject([{ alpha: 1 }, { alpha: 1 }, { alpha: 1 }, { alpha: 1 }])
    snapshot.room.presentationBaseline = 'resumed-with-short-fuse'; scene.update(snapshot, 600)
    expect(actors.children[0].children[4]).toMatchObject({ visible: true, alpha: 1 })
  })
  it('draws a distinct Short Flame curse cue with all simultaneous effects', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'shortFlame' }]
    Object.assign(snapshot.state.actors[0], {
      damageGraceRemainingTicks: 60,
      shieldRemainingTicks: 300,
      goldenFlameRemainingTicks: 480,
      shortFuseRemainingTicks: 240,
      shortFlameRemainingTicks: 600,
    })
    scene.update(snapshot, 0)
    const [, , pickups, , , actors] = layers(scene)
    expect(pickups.children[0].shapes).toHaveLength(22)
    expect(actors.children[0].children.slice(1, 6)).toMatchObject([
      { visible: true }, { visible: true }, { visible: true }, { visible: true },
      { visible: true, alpha: 0.7 },
    ])
    scene.setReducedMotion(true)
    scene.render(500)
    expect(actors.children[0].children.slice(1, 6).map(({ alpha }) => alpha))
      .toEqual([1, 1, 1, 1, 1])
    snapshot.room.presentationBaseline = 'resumed-with-short-flame'
    scene.update(snapshot, 600)
    expect(actors.children[0].children[5]).toMatchObject({ visible: true, alpha: 1 })
  })
  it('draws a distinct Long Fuse cue and freezes it with reduced motion', () => {
    const scene = new ArenaScene(1); const snapshot = gameSnapshot()
    snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'longFuse' }]
    Object.assign(snapshot.state.actors[0], { shortFuseRemainingTicks: 240, longFuseRemainingTicks: 600 })
    scene.update(snapshot, 0); const [, , pickups, , , actors] = layers(scene)
    expect(pickups.children[0].shapes).toHaveLength(20); expect(actors.children[0].children.slice(4, 7)).toMatchObject([
      { visible: true }, { visible: false }, { visible: true, alpha: 0.65 }])
    scene.setReducedMotion(true); scene.render(500); expect(actors.children[0].children[6]).toMatchObject({ visible: true, alpha: 1 })
  })
  it('resets retained state idempotently and snaps the same identity to a fresh baseline', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    scene.update(snapshot, 0)
    const originalLayers = [...layers(scene)]
    const oldDrawings = originalLayers.slice(1).flatMap((layer) => layer.children)
    scene.setReducedMotion(true)
    scene.reset()
    scene.reset()
    expect(layers(scene)).toEqual(originalLayers)
    expect(originalLayers[0].shapes).toEqual([])
    expect(originalLayers[0].cached).toBe(false)
    expect(originalLayers.slice(1).every((layer) => layer.children.length === 0)).toBe(true)
    expect(oldDrawings.every((drawing) => drawing.destroyed)).toBe(true)
    snapshot.state.actors[0].position.x += 1024
    scene.update(snapshot, 100)
    expect(originalLayers[0].cached).toBe(true)
    expect(originalLayers[0].cacheUpdates).toBe(2)
    const actor = layers(scene)[5].children[0]
    expect(actor.x).toBe(120)
    snapshot.state.actors[0].position.x += 1024
    scene.update(snapshot, 150)
    expect(actor.x).toBe(168)
  })
  it('refreshes changed terrain and regenerates only a populated cache after context restoration', () => {
    const scene = new ArenaScene(2)
    const snapshot = gameSnapshot()
    const terrain = layers(scene)[0]
    scene.restoreContext()
    expect(terrain.cacheUpdates).toBe(0)
    scene.update(snapshot, 0)
    scene.restoreContext()
    expect(terrain.cacheUpdates).toBe(2)
    snapshot.state.arena.terrain[1] = snapshot.state.arena.terrain[1] === 'wall' ? 'floor' : 'wall'
    scene.update(snapshot, 50)
    expect(terrain.cacheUpdates).toBe(3)
    snapshot.state.arena.width -= 1
    scene.update(snapshot, 100)
    expect(terrain.cacheUpdates).toBe(4)
  })
})
