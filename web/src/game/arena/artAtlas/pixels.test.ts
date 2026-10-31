import { afterEach, describe, expect, it, vi } from 'vitest'
import type { ArtManifest } from './types'
import { decodePng } from './pixels'

afterEach(() => vi.unstubAllGlobals())

function pngHeader(width: number, height: number): Uint8Array {
  const bytes = new Uint8Array(24)
  bytes.set([137, 80, 78, 71, 13, 10, 26, 10])
  bytes.set([73, 72, 68, 82], 12)
  const view = new DataView(bytes.buffer)
  view.setUint32(16, width)
  view.setUint32(20, height)
  return bytes
}

const manifest = (width: number, height: number): ArtManifest => ({
  sheet: { width, height },
} as ArtManifest)

describe('atlas PNG input budgets', () => {
  it('rejects oversized compressed input before browser decoding', async () => {
    const createBitmap = vi.fn()
    vi.stubGlobal('createImageBitmap', createBitmap)
    const bytes = new Uint8Array(16 * 1024 * 1024 + 1)
    await expect(decodePng(bytes, manifest(1120, 2128))).rejects.toThrow('Compressed PNG')
    expect(createBitmap).not.toHaveBeenCalled()
  })

  it('accepts the canonical 2128px authoring extent before browser decoding', async () => {
    const createBitmap = vi.fn().mockRejectedValue(new Error('stub decoder reached'))
    vi.stubGlobal('createImageBitmap', createBitmap)

    await expect(decodePng(pngHeader(1120, 2128), manifest(1120, 2128)))
      .rejects.toThrow('stub decoder reached')
    expect(createBitmap).toHaveBeenCalledOnce()
  })

  it.each([
    { name: 'zero width', png: [0, 2128], sheet: [0, 2128], message: '2128px authoring' },
    { name: 'authoring extent overflow', png: [1120, 2129], sheet: [1120, 2129], message: '2128px authoring' },
    { name: 'decoded RGBA overflow', png: [2128, 2128], sheet: [2128, 2128], message: '16MiB RGBA' },
    { name: 'declared sheet mismatch', png: [1120, 2128], sheet: [1120, 1904], message: '1120 × 1904' },
  ])('rejects $name before browser decoding', async ({ png, sheet, message }) => {
    const createBitmap = vi.fn()
    vi.stubGlobal('createImageBitmap', createBitmap)

    await expect(decodePng(
      pngHeader(png[0], png[1]), manifest(sheet[0], sheet[1]),
    )).rejects.toThrow(message)
    expect(createBitmap).not.toHaveBeenCalled()
  })
})
