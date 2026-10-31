import { describe, expect, it } from 'vitest'
import { createArtCatalog } from './catalog'
import { makeFireArtwork, type FireArtworkId } from './fireArtwork'

const FLAMES: readonly FireArtworkId[] = [
  'effect.flame.2', 'effect.flame.3', 'effect.flame.4', 'effect.flame.5',
]

describe('artist fire template drawings', () => {
  it.each(FLAMES)('%s reaches all tile edges for continuous rays', (id) => {
    const drawing = makeFireArtwork(id)
    expect(drawing.getLocalBounds().rectangle).toMatchObject({ x: 0, y: 0, width: 48, height: 48 })
    drawing.destroy()
  })

  it('keeps the ember tiny and centered while giving smoke and ignition readable silhouettes', () => {
    const ember = makeFireArtwork('effect.ember')
    const smoke = makeFireArtwork('effect.smoke')
    const ignition = makeFireArtwork('effect.ignition')
    expect(ember.getLocalBounds().rectangle).toMatchObject({ x: 22, y: 22, width: 4, height: 4 })
    expect(smoke.getLocalBounds().rectangle).toMatchObject({ x: 10, y: 9, width: 28, height: 29 })
    expect(ignition.getLocalBounds().rectangle).toMatchObject({ x: 5, y: 5, width: 38, height: 38 })
    for (const drawing of [ember, smoke, ignition]) drawing.destroy()
  })

  it('supplies every appended fire frame through the canonical catalogue', () => {
    const ids = createArtCatalog().entries.slice(189).map(({ frame }) => frame.id)
    expect(ids).toEqual([...FLAMES, 'effect.ember', 'effect.smoke', 'effect.ignition'])
  })
})
