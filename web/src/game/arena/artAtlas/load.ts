import { CanvasSource } from 'pixi.js'
import sourceManifestJson from '../../../../../assets/art/arena-source.json'
import { pixelCanvas, decodePng, packRuntimePixels } from './pixels'
import { createAtlasRegions, type AtlasRegions } from './regions'
import { buildRuntimeManifest } from './runtime'
import type { ArtManifest } from './types'
import { assertValidRuntimeManifest, assertValidSourceManifest } from './validate'

const sourceManifest = sourceManifestJson as ArtManifest
const sourceUrl = new URL('../../../../../assets/art/arena-source.png', import.meta.url)

export async function loadArenaAtlas(): Promise<AtlasRegions> {
  assertValidSourceManifest(sourceManifest)
  const response = await fetch(sourceUrl)
  if (!response.ok) throw new Error(`Arena artwork failed to load (${response.status})`)
  const sourcePixels = await decodePng(new Uint8Array(await response.arrayBuffer()), sourceManifest)
  assertValidSourceManifest(sourceManifest, sourcePixels)
  const runtimeManifest = buildRuntimeManifest(sourceManifest)
  const runtimePixels = packRuntimePixels(sourcePixels, sourceManifest, runtimeManifest)
  assertValidRuntimeManifest(runtimeManifest, runtimePixels)
  const source = new CanvasSource({
    resource: pixelCanvas(runtimePixels),
    resolution: 1,
    scaleMode: 'nearest',
    autoGenerateMipmaps: false,
    antialias: false,
    autoGarbageCollect: false,
    label: 'arena-atlas',
  })
  try {
    return createAtlasRegions(source, runtimeManifest)
  } catch (error) {
    source.destroy()
    throw error
  }
}
