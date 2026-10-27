import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  decodePng: vi.fn(),
  packRuntimePixels: vi.fn(),
  pixelCanvas: vi.fn(),
  createAtlasRegions: vi.fn(),
  assertValidRuntimeManifest: vi.fn(),
  assertValidSourceManifest: vi.fn(),
  sources: [] as Array<{ destroyed: boolean; destroy(): void }>,
}))

vi.mock('pixi.js', () => ({
  CanvasSource: class {
    destroyed = false

    constructor() { mocks.sources.push(this) }

    destroy(): void { this.destroyed = true }
  },
}))
vi.mock('./pixels', () => ({
  decodePng: mocks.decodePng,
  packRuntimePixels: mocks.packRuntimePixels,
  pixelCanvas: mocks.pixelCanvas,
}))
vi.mock('./regions', () => ({ createAtlasRegions: mocks.createAtlasRegions }))
vi.mock('./validate', () => ({
  assertValidRuntimeManifest: mocks.assertValidRuntimeManifest,
  assertValidSourceManifest: mocks.assertValidSourceManifest,
}))

import { createArenaAtlas, loadArenaAtlas } from './load'

const sourcePixels = { width: 1120, height: 1344, data: new Uint8Array() }
const runtimePixels = { width: 2040, height: 440, data: new Uint8Array() }

afterEach(() => vi.unstubAllGlobals())

beforeEach(() => {
  vi.clearAllMocks()
  mocks.sources.length = 0
  mocks.decodePng.mockResolvedValue(sourcePixels)
  mocks.packRuntimePixels.mockReturnValue(runtimePixels)
  mocks.pixelCanvas.mockReturnValue({})
  mocks.createAtlasRegions.mockImplementation((source) => ({ source }))
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
    ok: true, status: 200, arrayBuffer: async () => new ArrayBuffer(24),
  }))
})

describe('arena atlas loading', () => {
  it('validates metadata before decoding and gives each renderer independent ownership', async () => {
    const first = await loadArenaAtlas()
    const second = await loadArenaAtlas()

    expect(first).not.toBe(second)
    expect(mocks.sources).toHaveLength(2)
    expect(mocks.createAtlasRegions.mock.calls[0]?.[0]).not.toBe(
      mocks.createAtlasRegions.mock.calls[1]?.[0],
    )
    expect(mocks.assertValidSourceManifest).toHaveBeenCalledTimes(6)
    expect(mocks.assertValidSourceManifest.mock.invocationCallOrder[0]).toBeLessThan(
      mocks.decodePng.mock.invocationCallOrder[0] ?? 0,
    )
  })

  it('cleans a failed source and allows a fresh later load', async () => {
    mocks.createAtlasRegions.mockImplementationOnce(() => { throw new Error('region failure') })
      .mockImplementationOnce((source) => ({ source }))

    await expect(loadArenaAtlas()).rejects.toThrow('region failure')
    expect(mocks.sources[0]?.destroyed).toBe(true)
    await expect(loadArenaAtlas()).resolves.toBeDefined()
    expect(mocks.sources).toHaveLength(2)
    expect(mocks.sources[1]?.destroyed).toBe(false)
  })

  it('packs supplied source pixels without fetching the bundled artwork', () => {
    const regions = createArenaAtlas(sourcePixels)

    expect(regions).toEqual({ source: mocks.sources[0] })
    expect(fetch).not.toHaveBeenCalled()
    expect(mocks.decodePng).not.toHaveBeenCalled()
    expect(mocks.assertValidSourceManifest).toHaveBeenCalledWith(expect.anything(), sourcePixels)
    expect(mocks.packRuntimePixels.mock.calls[0]?.[0]).toBe(sourcePixels)
  })
})
