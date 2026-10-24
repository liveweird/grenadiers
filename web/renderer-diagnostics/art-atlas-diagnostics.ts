import {
  createSourceManifest, type ArtManifest, type PixelBuffer, type RuntimeArtManifest,
} from '../src/game/arena/artAtlas'
import { bytesToBase64, pngBytes, slicePixels } from './art-atlas-pixels'
import { inspectImportedFrame, renderBaselineSource } from './art-atlas-render'

export type PixelEdit = { x: number; y: number; rgba: readonly [number, number, number, number] }
export type ImportState = {
  source?: PixelBuffer
  sourceManifest?: ArtManifest
  runtime?: PixelBuffer
  runtimeManifest?: RuntimeArtManifest
}

export async function createEditedTemplate(
  edit: PixelEdit,
  serializeManifest: (manifest: ArtManifest) => string,
): Promise<{ pngBase64: string; manifestJson: string }> {
  const manifest = createSourceManifest()
  const pixels = await renderBaselineSource(manifest)
  if (!Number.isInteger(edit.x) || !Number.isInteger(edit.y)
    || edit.x < 0 || edit.y < 0 || edit.x >= pixels.width || edit.y >= pixels.height
    || edit.rgba.length !== 4 || edit.rgba.some((value) => !Number.isInteger(value) || value < 0 || value > 255)) {
    throw new RangeError('Edited pixel must use an in-bounds integer coordinate and four byte values')
  }
  const changed = { ...pixels, data: new Uint8ClampedArray(pixels.data) }
  changed.data.set(edit.rgba, (edit.y * pixels.width + edit.x) * 4)
  return {
    pngBase64: bytesToBase64(await pngBytes(changed)),
    manifestJson: serializeManifest(manifest),
  }
}

export async function compareImportedFrame(id: string, state: ImportState): Promise<{
  differentBytes: number
  bounds: { x: number; y: number; width: number; height: number }
}> {
  const { source, sourceManifest, runtime, runtimeManifest } = state
  if (!source || !sourceManifest || !runtime || !runtimeManifest) {
    throw new Error('Art atlas is not initialized')
  }
  const sourceFrame = sourceManifest.frames.find((frame) => frame.id === id)
  if (!sourceFrame) throw new RangeError(`Unknown frame ${id}`)
  if (sourceFrame.opacity !== undefined) throw new RangeError('Opaque frame comparison does not accept manifest opacity')
  const expected = slicePixels(
    source, sourceFrame.sourceRect.x, sourceFrame.sourceRect.y,
    sourceFrame.sourceRect.width, sourceFrame.sourceRect.height,
  )
  const actual = await inspectImportedFrame(runtime, runtimeManifest, id)
  let differentBytes = 0
  for (let index = 0; index < expected.data.length; index += 1) {
    if (expected.data[index] !== actual.pixels.data[index]) differentBytes += 1
  }
  return { differentBytes, bounds: actual.bounds }
}
