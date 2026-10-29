import { Sprite, Texture, TextureSource } from 'pixi.js'
import { describe, expect, it } from 'vitest'
import type { SnapshotMessage } from '../types'
import { createAtlasArtwork } from './artwork'
import { createSourceManifest } from './manifest'
import { createAtlasRegions, type AtlasRegions } from './regions'
import { buildRuntimeManifest } from './runtime'

function recordingRegions(created: string[]): AtlasRegions {
  return {
    createSprite(id: string) {
      const resolved = id.startsWith('bomb.shell.contact.')
        ? 'bomb.shell.contact.standard' : id
      const sprite = new Sprite(Texture.EMPTY)
      sprite.label = resolved
      if (resolved === 'actor.shadow') sprite.alpha = 0.3
      if (resolved === 'bomb.shadow') sprite.alpha = 0.28
      created.push(resolved)
      return sprite
    },
    destroy() {},
  }
}

describe('atlas gameplay artwork', () => {
  it('assembles actor sprites in the legacy tree order and initial state', () => {
    const created: string[] = []
    const actor = createAtlasArtwork(recordingRegions(created)).makeActor('two')

    expect(actor.root.children.map(({ label }) => label)).toEqual([
      'actor.shadow', 'actor.cue.protection', 'actor.cue.shield', 'actor.cue.goldenFlame',
      'actor.cue.shortFuse', 'actor.cue.shortFlame', 'actor.cue.longFuse',
      'actor.cue.reverseControls', 'actor.cue.diarrhea', null, 'actor.cue.piercing',
      'actor.cue.power', 'actor.cue.landmine', null,
      'actor.two.north.0', 'actor.two.north.1', 'actor.two.north.2', 'actor.two.north.3',
      'actor.two.east.0', 'actor.two.east.1', 'actor.two.east.2', 'actor.two.east.3',
      'actor.two.south.0', 'actor.two.south.1', 'actor.two.south.2', 'actor.two.south.3',
      'actor.two.west.0', 'actor.two.west.1', 'actor.two.west.2', 'actor.two.west.3',
      'actor.cue.cloak', 'actor.cue.stopwatch', 'actor.cue.frozen', 'actor.cue.wallPass',
    ])
    expect(actor.slowFoot.children.map(({ label }) => label)).toEqual([
      'actor.cue.slowBoot', 'actor.cue.slowMarker.1', 'actor.cue.slowMarker.2',
      'actor.cue.slowMarker.3', 'actor.cue.slowMarker.4', 'actor.cue.slowMarker.5',
      'actor.cue.slowMarker.6',
    ])
    expect(actor.equipment.children.map(({ label }) => label)).toEqual([
      'actor.cue.kick', 'actor.cue.punch', 'actor.cue.bracelet',
    ])
    expect(actor.poses.south[0].visible).toBe(true)
    expect(Object.values(actor.poses).flat().filter(({ visible }) => visible)).toHaveLength(1)
    expect(actor.shadow.alpha).toBeCloseTo(0.3)
    actor.root.destroy({ children: true })
  })

  it('assembles three timed shell sprites after the stable bomb children', () => {
    const bomb = createAtlasArtwork(recordingRegions([])).makeBomb('piercing', 'timed')

    expect(bomb.shellFrames?.map(({ label, visible }) => ({ label, visible }))).toEqual([
      { label: 'bomb.shell.timed.piercing', visible: true },
      { label: 'bomb.shell.timed.piercing.1', visible: false },
      { label: 'bomb.shell.timed.piercing.2', visible: false },
    ])
    expect(bomb.root.children.slice(0, 4)).toEqual([
      bomb.shadow, bomb.shell, bomb.spark, bomb.pause,
    ])
    expect(bomb.root.children.slice(4)).toEqual(bomb.shellFrames?.slice(1))
    bomb.root.destroy({ children: true })
  })

  it('retains deterministic terrain variants and clears only owned sprites', () => {
    const created: string[] = []
    const artwork = createAtlasArtwork(recordingRegions(created))
    const terrain = artwork.makeTerrain()
    const arena = {
      width: 3, height: 2,
      terrain: ['floor', 'floor', 'wall', 'floor', 'wall', 'floor'],
      blocks: [],
    } as unknown as SnapshotMessage['state']['arena']
    terrain.paint(arena)

    expect(created).toEqual([
      'terrain.floor.4', 'terrain.floor.1', 'terrain.wall.0',
      'terrain.floor.3', 'terrain.wall.0', 'terrain.floor.3',
    ])
    const first = terrain.root.children[0]
    expect(terrain.root.children.map(({ x, y }) => [x, y])).toEqual([
      [0, 0], [48, 0], [96, 0], [0, 48], [48, 48], [96, 48],
    ])
    terrain.clear()
    expect(terrain.root.children).toHaveLength(0)
    expect(first.destroyed).toBe(true)
  })

  it('uses contact aliases and preserves bomb, flame, and Oil state contracts', () => {
    const created: string[] = []
    const artwork = createAtlasArtwork(recordingRegions(created))
    const bomb = artwork.makeBomb('power', 'contact')
    const flame = artwork.makeFlame()
    const oil = artwork.makeOilBarrel()

    expect(bomb.root.children.map(({ label }) => label)).toEqual([
      'bomb.shadow', 'bomb.shell.contact.standard', 'bomb.spark', 'bomb.pause',
    ])
    expect(bomb).toMatchObject({ remote: false, contact: true, paused: false })
    expect(bomb.shadow.alpha).toBeCloseTo(0.28)
    expect(bomb.spark.visible).toBe(false)
    expect(flame.frames.map(({ visible }) => visible)).toEqual([true, false])
    expect(oil).toMatchObject({ state: 'unlit' })
    expect(oil.unlit.visible).toBe(true)
    expect(oil.charred.visible).toBe(false)
    expect(oil.flame.root.visible).toBe(false)
    bomb.root.destroy({ children: true })
    flame.root.destroy({ children: true })
    oil.root.destroy({ children: true })
  })

  it('does not destroy the shared atlas source with an individual node tree', () => {
    const runtime = buildRuntimeManifest(createSourceManifest())
    const source = new TextureSource({
      width: runtime.texture.width, height: runtime.texture.height,
    })
    const regions = createAtlasRegions(source, runtime)
    const actor = createAtlasArtwork(regions).makeActor('one')
    actor.root.destroy({ children: true, context: true })
    expect(source.destroyed).toBe(false)
    regions.destroy()
    expect(source.destroyed).toBe(true)
  })

  it('creates the environment cloud from its single atlas frame', () => {
    const created: string[] = []
    const cloud = createAtlasArtwork(recordingRegions(created)).makeStormCloud()
    expect(created).toEqual(['environment.stormCloud'])
    expect(cloud.label).toBe('environment.stormCloud')
    cloud.destroy()
  })

  it('creates typed lair and four-heading mob artwork from canonical frames', () => {
    const created: string[] = []
    const artwork = createAtlasArtwork(recordingRegions(created))
    const lair = artwork.makeLair()
    const mob = artwork.makeMob()
    expect(created).toEqual([
      'terrain.lair', 'mob.north', 'mob.east', 'mob.south', 'mob.west',
    ])
    expect(lair.label).toBe('terrain.lair')
    expect(Object.values(mob.poses).map(({ label }) => label)).toEqual([
      'mob.north', 'mob.east', 'mob.south', 'mob.west',
    ])
    lair.destroy()
    mob.root.destroy({ children: true })
  })
})
