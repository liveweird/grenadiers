import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { graphicsArtwork } from './graphicsArtwork'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = { children: NodeView[]; x: number; y: number; visible: boolean; destroyed: boolean }

function trackedScene() {
  const mobs: ReturnType<typeof graphicsArtwork.makeMob>[] = []
  const artwork = {
    ...graphicsArtwork,
    makeMob: vi.fn(() => {
      const drawing = graphicsArtwork.makeMob()
      mobs.push(drawing)
      return drawing
    }),
  }
  return { scene: new ArenaScene(1, artwork), artwork, mobs }
}

function populate(snapshot: ReturnType<typeof gameSnapshot>): void {
  snapshot.state.lairs = [
    { id: 0, cell: { column: 6, row: 6 }, spawnRemainingTicks: null, livingMobs: 1 },
    { id: 1, cell: { column: 8, row: 6 }, spawnRemainingTicks: 300, livingMobs: 0 },
  ]
  snapshot.state.mobs = [{
    id: 90, lairId: 0, position: { x: 6_656, y: 6_656 }, heading: 'east', frozen: false,
  }]
}

describe('lair and mob arena entities', () => {
  it('overlays lairs on the wall layer and renders seatless mobs below weather', () => {
    const { scene, artwork, mobs } = trackedScene()
    const snapshot = gameSnapshot()
    populate(snapshot)
    snapshot.state.stormCloud = {
      position: { x: 7_680, y: 6_656 }, direction: 'east', legRemainingTicks: 128,
    }
    scene.update(snapshot, 0)

    expect(artwork.makeMob).toHaveBeenCalledOnce()
    expect(scene.root.children[1].children.slice(-2).map(({ x, y }) => [x, y]))
      .toEqual([[288, 288], [384, 288]])
    expect(scene.root.children[5].children).toContain(mobs[0]!.root)
    expect(scene.root.children.at(-1)?.children).toHaveLength(1)
    expect(mobs[0]!.poses.east.visible).toBe(true)
    expect(mobs[0]!.poses.north.visible).toBe(false)
  })

  it('interpolates movement and heading, then snaps on the authoritative freeze edge', () => {
    const { scene, mobs } = trackedScene()
    const snapshot = gameSnapshot()
    populate(snapshot)
    scene.update(snapshot, 0)
    const mob = mobs[0]!

    snapshot.state.physicalTick = 103
    snapshot.state.mobs![0]!.position.x += 1_024
    snapshot.state.mobs![0]!.heading = 'south'
    scene.update(snapshot, 100)
    expect(mob.root.x).toBe(312)
    expect(mob.poses.south.visible).toBe(true)
    scene.render(125)
    expect(mob.root.x).toBe(336)

    snapshot.state.physicalTick = 106
    snapshot.state.mobs![0]!.position.x += 1_024
    snapshot.state.mobs![0]!.frozen = true
    scene.update(snapshot, 200)
    expect(mob.root.x).toBe(408)
    scene.render(225)
    expect(mob.root.x).toBe(408)
  })

  it('removes absent entities and rebuilds them on reconnect, rematch, and explicit reset', () => {
    const { scene, mobs } = trackedScene()
    const snapshot = gameSnapshot()
    populate(snapshot)
    scene.update(snapshot, 0)
    const first = mobs[0]!.root as unknown as NodeView

    snapshot.state.physicalTick = 101
    snapshot.state.mobs = []
    snapshot.state.lairs = []
    scene.update(snapshot, 20)
    expect(first.destroyed).toBe(true)

    populate(snapshot)
    snapshot.room.presentationBaseline = 'reconnected'
    scene.update(snapshot, 40)
    const second = mobs[1]!.root as unknown as NodeView
    expect(second).not.toBe(first)
    snapshot.room.roundId += 1
    scene.update(snapshot, 60)
    expect(second.destroyed).toBe(true)
    expect(mobs[2]!.root).toMatchObject({ x: 312, y: 312 })
    scene.reset()
    expect((mobs[2]!.root as unknown as NodeView).destroyed).toBe(true)
  })
})
