import { Container } from 'pixi.js'
import { describe, expect, it } from 'vitest'
import { makeActor } from '../actorArt'
import { makeFlame } from '../entityArt'
import { RENDERABLE_PICKUP_KINDS } from '../types'
import { createArtCatalog } from './catalog'
import { createSourceManifest } from './manifest'

describe('art atlas catalogue', () => {
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
    expect(manifest.frames.filter(({ id }) => id.startsWith('terrain.floor.'))).toHaveLength(8)
    expect(manifest.frames.filter(({ id }) => /^terrain\.wall\.\d$/.test(id))).toHaveLength(2)
    expect(manifest.frames.filter(({ id }) => /^actor\.(one|two)\.(north|east|south|west)\.[01]$/.test(id)))
      .toHaveLength(16)
    expect(manifest.frames.filter(({ id }) => id.startsWith('actor.cue.slowMarker.'))).toHaveLength(6)
    expect(manifest.frames.filter(({ status }) => status === 'reserved').map(({ id }) => id)).toEqual([
      'reserved.actor.one.death', 'reserved.actor.two.death',
      'reserved.terrain.block.destruction', 'reserved.pickup.collection',
    ])
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
