import { beforeEach, describe, expect, it, vi } from 'vitest'
import sourceManifestJson from '../../../../../assets/art/arena-source.json'
import type { ArtManifest, PixelBuffer } from '../artAtlas/types'
import { createThreeAtlas, opaquePaintedCenter } from './atlas'

const hooks = vi.hoisted(() => ({
  decode: vi.fn(),
  canvas: vi.fn(),
}))

vi.mock('../artAtlas/pixels', async (importOriginal) => {
  const original = await importOriginal<typeof import('../artAtlas/pixels')>()
  return { ...original, decodePng: hooks.decode, pixelCanvas: hooks.canvas }
})

const manifest = sourceManifestJson as ArtManifest

function validSourcePixels(): PixelBuffer {
  const data = new Uint8ClampedArray(manifest.sheet.width * manifest.sheet.height * 4)
  for (const frame of manifest.frames) {
    if (frame.status !== 'baseline') continue
    if (frame.id === 'actor.shadow' || frame.id === 'bomb.shadow') continue
    data[(frame.sourceRect.y * manifest.sheet.width + frame.sourceRect.x) * 4 + 3] = 255
  }
  const paintBox = (id: string, left: number, top: number, right: number, bottom: number): void => {
    const frame = manifest.frames.find((candidate) => candidate.id === id)
    if (!frame) throw new Error(`Missing fixture frame ${id}`)
    for (let y = top; y <= bottom; y += 1) {
      for (let x = left; x <= right; x += 1) {
        data[((frame.sourceRect.y + y) * manifest.sheet.width + frame.sourceRect.x + x) * 4 + 3] = 255
      }
    }
  }
  paintBox('actor.shadow', 28, 60, 51, 65)
  paintBox('bomb.shadow', 30, 53, 49, 57)
  return { width: manifest.sheet.width, height: manifest.sheet.height, data }
}

beforeEach(() => {
  hooks.decode.mockReset().mockResolvedValue(validSourcePixels())
  hooks.canvas.mockReset().mockImplementation((pixels: PixelBuffer) => {
    const canvas = document.createElement('canvas')
    canvas.width = pixels.width
    canvas.height = pixels.height
    return canvas
  })
  vi.stubGlobal('fetch', vi.fn(async () => ({
    ok: true,
    arrayBuffer: async () => new ArrayBuffer(24),
  })))
})

describe('Three atlas loader', () => {
  it('rejects transport and decode failures before allocating a GPU texture source', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({ ok: false, status: 503 })))
    await expect(createThreeAtlas()).rejects.toThrow('Arena artwork failed to load (503)')
    expect(hooks.decode).not.toHaveBeenCalled()
    expect(hooks.canvas).not.toHaveBeenCalled()

    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: true, arrayBuffer: async () => new ArrayBuffer(4),
    })))
    hooks.decode.mockRejectedValueOnce(new Error('Source file is not a PNG image'))
    await expect(createThreeAtlas()).rejects.toThrow('Source file is not a PNG image')
    expect(hooks.canvas).not.toHaveBeenCalled()
  })

  it('uses canonical aliases, caches each anchor, and guards disposed geometry', async () => {
    const atlas = await createThreeAtlas()
    const [alias, target] = Object.entries(atlas.manifest.aliases)[0]
    expect(alias).toBeDefined()
    expect(atlas.frame(alias)).toBe(atlas.frame(target))
    expect(() => atlas.frame('missing.frame')).toThrow('Unknown arena artwork frame')

    const pivot = atlas.geometry(target)
    expect(atlas.geometry(alias)).toBe(pivot)
    const feet = atlas.geometry(target, 'feet')
    const center = atlas.geometry(target, 'center')
    expect(atlas.geometry(alias, 'paintedCenter')).toBe(atlas.geometry(target, 'paintedCenter'))
    expect(feet).not.toBe(pivot)
    expect(center).not.toBe(pivot)
    const disposals = [pivot, feet, center].map((geometry) => vi.spyOn(geometry, 'dispose'))
    const textureDispose = vi.spyOn(atlas.texture, 'dispose')

    atlas.dispose()
    atlas.dispose()
    disposals.forEach((dispose) => expect(dispose).toHaveBeenCalledOnce())
    expect(textureDispose).toHaveBeenCalledOnce()
    expect(() => atlas.geometry(target)).toThrow('Arena atlas has been disposed')
  })

  it('anchors each shadow to its painted bounds without mutating canonical frames', async () => {
    const atlas = await createThreeAtlas()
    const actor = atlas.frame('actor.shadow')
    const bomb = atlas.frame('bomb.shadow')
    const actorPivot = { ...actor.pivot }
    const bombPivot = { ...bomb.pivot }

    const actorPositions = Array.from(
      atlas.geometry('actor.shadow', 'paintedCenter').getAttribute('position').array,
    )
    const bombPositions = Array.from(
      atlas.geometry('bomb.shadow', 'paintedCenter').getAttribute('position').array,
    )
    expect(actorPositions.slice(0, 6)).toEqual([-40, -17, 0, 40, -17, 0])
    expect(bombPositions.slice(0, 6)).toEqual([-40, -24.5, 0, 40, -24.5, 0])
    expect(actor.pivot).toEqual(actorPivot)
    expect(bomb.pivot).toEqual(bombPivot)
    atlas.dispose()
  })

  it('falls back to the declared pivot when painted artwork is blank', () => {
    const pixels = { width: 4, height: 4, data: new Uint8Array(4 * 4 * 4) }
    expect(opaquePaintedCenter({
      sourceRect: { x: 0, y: 0, width: 4, height: 4 }, pivot: { x: 1, y: 3 },
    }, pixels)).toEqual({ x: 1, y: 3 })
  })
})
