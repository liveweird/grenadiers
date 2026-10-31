import sourceManifestJson from '../../../../../assets/art/arena-source.json'
import {
  CanvasTexture, NearestFilter, SRGBColorSpace,
} from 'three'
import { decodePng, packRuntimePixels, pixelCanvas } from '../artAtlas/pixels'
import { buildRuntimeManifest } from '../artAtlas/runtime'
import type { ArtFrame, ArtManifest, PixelBuffer } from '../artAtlas/types'
import { assertValidRuntimeManifest, assertValidSourceManifest } from '../artAtlas/validate'
import type { FrameAnchor, RuntimeFrame, ThreeAtlas } from './contracts'
import { createFrameGeometry } from './frameGeometry'

const arenaSourceManifest = sourceManifestJson as ArtManifest
const sourceUrl = new URL('../../../../../assets/art/arena-source.png', import.meta.url)
const PAINTED_CENTER_FRAMES = ['actor.shadow', 'bomb.shadow'] as const

export function opaquePaintedCenter(
  frame: Pick<ArtFrame, 'sourceRect' | 'pivot'>, pixels: PixelBuffer,
): { x: number; y: number } {
  const { sourceRect } = frame
  let left = sourceRect.width
  let right = -1
  let top = sourceRect.height
  let bottom = -1
  for (let y = 0; y < sourceRect.height; y += 1) {
    for (let x = 0; x < sourceRect.width; x += 1) {
      const offset = ((sourceRect.y + y) * pixels.width + sourceRect.x + x) * 4
      if (pixels.data[offset + 3] === 0) continue
      left = Math.min(left, x)
      right = Math.max(right, x)
      top = Math.min(top, y)
      bottom = Math.max(bottom, y)
    }
  }
  if (right < left || bottom < top) return { ...frame.pivot }
  return { x: (left + right + 1) / 2, y: (top + bottom + 1) / 2 }
}

function findFrame(
  manifest: ThreeAtlas['manifest'], id: string,
): RuntimeFrame {
  const resolved = manifest.aliases[id] ?? id
  const frame = manifest.frames.find((candidate) => candidate.id === resolved)
  if (!frame) throw new RangeError(`Unknown arena artwork frame: ${id}`)
  return frame
}

export async function createThreeAtlas(): Promise<ThreeAtlas> {
  assertValidSourceManifest(arenaSourceManifest)
  const response = await fetch(sourceUrl)
  if (!response.ok) throw new Error(`Arena artwork failed to load (${response.status})`)
  const sourcePixels = await decodePng(
    new Uint8Array(await response.arrayBuffer()), arenaSourceManifest,
  )
  assertValidSourceManifest(arenaSourceManifest, sourcePixels)
  const sourceFrames = new Map(arenaSourceManifest.frames.map((frame) => [frame.id, frame]))
  const paintedCenters: ReadonlyMap<string, { x: number; y: number }> = new Map(
    PAINTED_CENTER_FRAMES.map((id) => {
      const sourceFrame = sourceFrames.get(id)
      if (!sourceFrame) throw new Error(`Painted-center frame is missing: ${id}`)
      return [id, opaquePaintedCenter(sourceFrame, sourcePixels)] as const
    }),
  )
  const manifest = buildRuntimeManifest(arenaSourceManifest)
  const pixels = packRuntimePixels(sourcePixels, arenaSourceManifest, manifest)
  assertValidRuntimeManifest(manifest, pixels)
  const texture = new CanvasTexture(pixelCanvas(pixels))
  texture.name = 'arena-atlas'
  texture.colorSpace = SRGBColorSpace
  texture.magFilter = NearestFilter
  texture.minFilter = NearestFilter
  texture.generateMipmaps = false
  texture.needsUpdate = true
  const geometries = new Map<string, ReturnType<typeof createFrameGeometry>>()
  let disposed = false
  const frame = (id: string): RuntimeFrame => findFrame(manifest, id)
  return {
    texture,
    sourceManifest: arenaSourceManifest,
    manifest,
    frame,
    geometry(id: string, anchor: FrameAnchor = 'pivot') {
      if (disposed) throw new Error('Arena atlas has been disposed')
      const resolved = frame(id)
      const key = `${resolved.id}:${anchor}`
      const cached = geometries.get(key)
      if (cached) return cached
      const paintedCenter = anchor === 'paintedCenter' ? paintedCenters.get(resolved.id) : undefined
      const geometryFrame = paintedCenter ? { ...resolved, pivot: paintedCenter } : resolved
      const geometry = createFrameGeometry(geometryFrame, manifest.texture, anchor)
      geometries.set(key, geometry)
      return geometry
    },
    dispose() {
      if (disposed) return
      disposed = true
      for (const geometry of geometries.values()) geometry.dispose()
      geometries.clear()
      texture.dispose()
    },
  }
}
