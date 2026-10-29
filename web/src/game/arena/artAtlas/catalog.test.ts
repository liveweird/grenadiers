import { Container } from 'pixi.js'
import { describe, expect, it } from 'vitest'
import adoptedSource from '../../../../../assets/art/arena-source.json'
import { makeActor } from '../actorArt'
import { makeFlame } from '../entityArt'
import { RENDERABLE_PICKUP_KINDS } from '../types'
import { createArtCatalog } from './catalog'
import { createSourceManifest } from './manifest'

describe('art atlas catalogue', () => {
  it('keeps the adopted artist manifest synchronized with the generated catalogue', () => {
    expect(adoptedSource).toEqual(createSourceManifest())
  })

  it('covers every renderable pickup while keeping Oil separate', () => {
    const manifest = createSourceManifest()
    const pickupIds = manifest.frames
      .filter(({ id }) => id.startsWith('pickup.'))
      .map(({ id }) => id.slice('pickup.'.length))
    expect(pickupIds).toEqual(RENDERABLE_PICKUP_KINDS)
    expect(pickupIds).not.toContain('oil')
    expect(manifest.compositions['oil.burning.0']).toEqual(['oil.charred', 'effect.flame.0'])
  })

  it('provides the complete baseline and only the planned transparent reservations', () => {
    const manifest = createSourceManifest()
    const actorFrames = manifest.frames.filter(({ id }) =>
      /^actor\.(one|two|three|four)\.(north|east|south|west)\.[0-3]$/.test(id))
    expect(manifest.frames.filter(({ id }) => id.startsWith('terrain.floor.'))).toHaveLength(8)
    expect(manifest.frames.filter(({ id }) => /^terrain\.wall\.\d$/.test(id))).toHaveLength(2)
    expect(actorFrames).toHaveLength(64)
    expect(manifest.frames.filter(({ id }) => id.startsWith('actor.cue.slowMarker.'))).toHaveLength(6)
    expect(manifest.frames.filter(({ status }) => status === 'reserved').map(({ id }) => id)).toEqual([
      'reserved.actor.one.death', 'reserved.actor.two.death',
      'reserved.terrain.block.destruction', 'reserved.pickup.collection',
    ])
    expect(manifest.frames).toHaveLength(155)
    expect(manifest.frames.filter(({ status }) => status === 'baseline')).toHaveLength(151)
    expect(manifest.sheet).toEqual({ width: 1120, height: 1792, slotSize: 112, columns: 10 })
    expect(Object.keys(manifest.animations).filter((id) => id.endsWith('.walk'))).toHaveLength(16)
  })

  it('appends four-frame walking and timed bomb poses after the frozen v111 slots', () => {
    const manifest = createSourceManifest()
    const walking = ['one', 'two', 'three', 'four'].flatMap((appearance) =>
      ['north', 'east', 'south', 'west'].flatMap((direction) =>
        [2, 3].map((frame) => `actor.${appearance}.${direction}.${frame}`)))
    const bombs = ['standard', 'piercing', 'power'].flatMap((blast) =>
      [1, 2].map((frame) => `bomb.shell.timed.${blast}.${frame}`))

    expect(manifest.frames.slice(0, 111)).toEqual(adoptedSource.frames.slice(0, 111))
    expect(manifest.frames.slice(111, 150).map(({ id }) => id)).toEqual([
      ...walking, ...bombs, 'environment.stormCloud',
    ])
    expect(manifest.animations['actor.one.north.walk']).toEqual({
      frameIds: [
        'actor.one.north.0', 'actor.one.north.2',
        'actor.one.north.1', 'actor.one.north.3',
      ],
      frameDurationMs: 110,
      reducedMotionFrame: 'actor.one.north.0',
    })
    expect(manifest.animations['bomb.timed.power.pulse'].frameIds).toEqual([
      'bomb.shell.timed.power', 'bomb.shell.timed.power.1', 'bomb.shell.timed.power.2',
    ])
  })

  it('appends the storm cloud in the last physical slot without moving earlier frames', () => {
    const frames = createSourceManifest().frames
    expect(frames.slice(0, 149)).toEqual(adoptedSource.frames.slice(0, 149))
    expect(frames[149]).toEqual({
      id: 'environment.stormCloud', status: 'baseline',
      sourceRect: { x: 1008, y: 1568, width: 80, height: 80 },
      localBounds: { x: -40, y: -40, width: 80, height: 80 },
      pivot: { x: 40, y: 40 },
    })
  })

  it('appends lairs and four mob directions in a new row with shared foot anchors', () => {
    const frames = createSourceManifest().frames
    expect(frames.slice(0, 150)).toEqual(adoptedSource.frames.slice(0, 150))
    expect(frames.slice(150).map(({ id }) => id)).toEqual([
      'terrain.lair', 'mob.north', 'mob.east', 'mob.south', 'mob.west',
    ])
    expect(frames[150].sourceRect).toEqual({ x: 0, y: 1680, width: 48, height: 48 })
    for (const [index, frame] of frames.slice(151).entries()) {
      expect(frame.sourceRect).toEqual({ x: (index + 1) * 112, y: 1680, width: 80, height: 80 })
      expect(frame.pivot).toEqual({ x: 40, y: 44 })
      expect(frame.footAnchor).toEqual({ x: 40, y: 65 })
    }
  })

  it('appends appearances three and four without moving any legacy atlas slot', () => {
    const frames = createSourceManifest().frames
    expect(frames.slice(0, 95).map(({ sourceRect }) => sourceRect)).toEqual(
      frames.slice(0, 95).map(({ sourceRect }, slot) => ({
        ...sourceRect, x: (slot % 10) * 112, y: Math.floor(slot / 10) * 112,
      })),
    )
    expect(frames[94].id).toBe('reserved.pickup.collection')
    expect(frames.slice(95, 111).map(({ id }) => id)).toEqual(
      ['three', 'four'].flatMap((appearance) =>
        ['north', 'east', 'south', 'west'].flatMap((direction) =>
          [0, 1].map((frame) => `actor.${appearance}.${direction}.${frame}`))),
    )
    expect(frames.slice(95, 111).every(({ sourceRect, localBounds, pivot, footAnchor }) =>
      sourceRect.width === 80 && sourceRect.height === 80
      && localBounds.x === -40 && localBounds.y === -44
      && localBounds.width === 80 && localBounds.height === 80
      && pivot.x === 40 && pivot.y === 44
      && footAnchor?.x === 40 && footAnchor.y === 65)).toBe(true)
  })

  it('leases named parts without depending on child indexes or changing gameplay alpha', () => {
    const actor = makeActor('one')
    expect(actor.shadow.alpha).toBeCloseTo(0.3)
    expect(actor.slowMarkers).toHaveLength(6)
    actor.root.destroy({ children: true })

    const shadowEntry = createArtCatalog().entries.find(({ frame }) => frame.id === 'actor.shadow')!
    const lease = shadowEntry.createDrawing()
    const exportStage = new Container()
    exportStage.addChild(lease.drawing)
    expect(lease.drawing.alpha).toBe(1)
    expect(shadowEntry.frame.opacity).toBeCloseTo(0.3)
    lease.destroy()
    lease.destroy()
    expect(lease.drawing.destroyed).toBe(true)
    exportStage.destroy({ children: true })
  })

  it('retains the baseline two-pixel top gap in flame frame one', () => {
    const flame = makeFlame()
    const bounds = flame.frames[1].getLocalBounds()
    expect(bounds.y).toBe(2)
    flame.root.destroy({ children: true })
  })
})
