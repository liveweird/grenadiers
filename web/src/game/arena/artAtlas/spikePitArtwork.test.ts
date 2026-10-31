import { describe, expect, it } from 'vitest'
import { createArtCatalog } from './catalog'
import { TILE_BOUNDS } from './descriptor'
import { makeSpikePit, type SpikePitArtworkPhase } from './spikePitArtwork'

const PHASES: readonly SpikePitArtworkPhase[] = ['idle', 'warning', 'raised']

describe('spike-pit artist template drawings', () => {
  it.each(PHASES)('fills one logical tile for %s', (phase) => {
    const drawing = makeSpikePit(phase)
    expect(drawing.getLocalBounds().rectangle).toMatchObject(TILE_BOUNDS)
    drawing.destroy()
  })

  it('registers three distinct shared-sheet catalogue entries', () => {
    const entries = createArtCatalog().entries.filter(({ frame }) =>
      frame.id.startsWith('terrain.spikePit.'))
    expect(entries.map(({ frame }) => frame.id)).toEqual(PHASES.map((phase) =>
      `terrain.spikePit.${phase}`))
    expect(entries.every(({ frame }) => frame.localBounds.x === 0 && frame.localBounds.y === 0
      && frame.localBounds.width === 48 && frame.localBounds.height === 48
      && frame.pivot.x === 0 && frame.pivot.y === 0)).toBe(true)
    for (const entry of entries) entry.createDrawing().destroy()
  })
})
