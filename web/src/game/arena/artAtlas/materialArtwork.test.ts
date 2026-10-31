import { describe, expect, it } from 'vitest'
import { createArtCatalog } from './catalog'
import { MATERIAL_FRAME_IDS } from './descriptor'

describe('artist material template drawings', () => {
  it.each(MATERIAL_FRAME_IDS)('fills exactly one logical tile for %s', (id) => {
    const entry = createArtCatalog().entries.find(({ frame }) => frame.id === id)
    expect(entry?.frame.sourceRect).toMatchObject({ width: 48, height: 48 })
    const lease = entry!.createDrawing()
    expect(lease.drawing.getLocalBounds().rectangle).toMatchObject({
      x: 0, y: 0, width: 48, height: 48,
    })
    lease.destroy()
    lease.destroy()
    expect(lease.drawing.destroyed).toBe(true)
  })
})
