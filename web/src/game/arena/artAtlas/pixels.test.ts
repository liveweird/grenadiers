import { afterEach, describe, expect, it, vi } from 'vitest'
import type { ArtManifest } from './types'
import { decodePng } from './pixels'

afterEach(() => vi.unstubAllGlobals())

describe('atlas PNG input budgets', () => {
  it('rejects oversized compressed input before browser decoding', async () => {
    const createBitmap = vi.fn()
    vi.stubGlobal('createImageBitmap', createBitmap)
    const bytes = new Uint8Array(16 * 1024 * 1024 + 1)
    const manifest = { sheet: { width: 1120, height: 1120 } } as ArtManifest

    await expect(decodePng(bytes, manifest)).rejects.toThrow('Compressed PNG')
    expect(createBitmap).not.toHaveBeenCalled()
  })
})
