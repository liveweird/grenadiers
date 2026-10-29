import { Graphics } from 'pixi.js'
import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import type { ArenaArtwork, TerrainDrawing } from './artwork'
import { graphicsArtwork } from './graphicsArtwork'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = {
  children: NodeView[]
  destroyed: boolean
  cacheCalls: Array<false | object>
  cacheUpdates: number
}

function trackedArtwork() {
  const terrainRoot = new Graphics()
  const terrain: TerrainDrawing = {
    root: terrainRoot,
    paint: vi.fn((arena) => graphicsArtwork.makeTerrain().paint(arena)),
    clear: vi.fn(),
  }
  const artwork: ArenaArtwork = {
    makeTerrain: vi.fn(() => terrain),
    makeBlock: vi.fn(graphicsArtwork.makeBlock),
    makePickup: vi.fn(graphicsArtwork.makePickup),
    makeActor: vi.fn(graphicsArtwork.makeActor),
    makeBomb: vi.fn(graphicsArtwork.makeBomb),
    makeFlame: vi.fn(graphicsArtwork.makeFlame),
    makeOilBarrel: vi.fn(graphicsArtwork.makeOilBarrel),
  }
  return { artwork, terrain, terrainRoot: terrainRoot as unknown as NodeView }
}

describe('arena artwork injection', () => {
  it('allocates retained scene nodes exclusively through the injected artwork factory', () => {
    const { artwork, terrain } = trackedArtwork()
    const scene = new ArenaScene(2, artwork)
    const snapshot = gameSnapshot()
    snapshot.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'shield' }]
    snapshot.state.oilBarrels = [{ cell: { column: 3, row: 3 }, state: 'unlit' }]

    scene.update(snapshot, 0)
    expect(artwork.makeTerrain).toHaveBeenCalledOnce()
    expect(artwork.makeBlock).toHaveBeenCalledOnce()
    expect(artwork.makePickup).toHaveBeenCalledExactlyOnceWith('shield')
    expect(artwork.makeBomb).toHaveBeenCalledExactlyOnceWith('standard', 'timed')
    expect(artwork.makeFlame).toHaveBeenCalledOnce()
    expect(artwork.makeOilBarrel).toHaveBeenCalledOnce()
    expect(artwork.makeActor).toHaveBeenCalledTimes(2)
    expect(terrain.paint).toHaveBeenCalledExactlyOnceWith(snapshot.state.arena)
    const bomb = vi.mocked(artwork.makeBomb).mock.results[0].value
    expect(bomb.shellFrames).toHaveLength(3)
    expect(bomb.shellFrames?.[0]).toBe(bomb.shell)
    expect(bomb.root.children.slice(0, 4)).toEqual([
      bomb.shadow, bomb.shell, bomb.spark, bomb.pause,
    ])
    expect(bomb.root.children.slice(4)).toEqual(bomb.shellFrames?.slice(1))

    scene.update(snapshot, 50)
    expect(artwork.makeBlock).toHaveBeenCalledOnce()
    expect(artwork.makePickup).toHaveBeenCalledOnce()
    expect(artwork.makeActor).toHaveBeenCalledTimes(2)
    expect(terrain.paint).toHaveBeenCalledOnce()
  })

  it('rebuilds only the terrain cache on context restoration and retains artwork ownership', () => {
    const { artwork, terrain, terrainRoot } = trackedArtwork()
    const scene = new ArenaScene(2, artwork)
    const snapshot = gameSnapshot()
    scene.update(snapshot, 0)
    const factoryCalls = vi.mocked(artwork.makeActor).mock.calls.length

    scene.restoreContext()
    expect(terrainRoot.cacheUpdates).toBe(2)
    expect(artwork.makeTerrain).toHaveBeenCalledOnce()
    expect(artwork.makeActor).toHaveBeenCalledTimes(factoryCalls)

    const owned = (scene.root.children as unknown as NodeView[])
      .slice(1).flatMap((layer) => [...layer.children])
    scene.reset()
    expect(terrainRoot.cacheCalls.at(-1)).toBe(false)
    expect(terrain.clear).toHaveBeenCalledTimes(2)
    expect(owned.every(({ destroyed }) => destroyed)).toBe(true)
    expect(artwork.makeTerrain).toHaveBeenCalledOnce()
  })
})
