import {
  MAX_RGBA_BYTES, MAX_TEXTURE_SIZE, RUNTIME_EXTRUSION, RUNTIME_PADDING,
} from './layout'
import type { ArtFrame, ArtManifest, RuntimeArtManifest } from './types'
import { assertValidSourceManifest } from './validate'

const MARGIN = RUNTIME_PADDING + RUNTIME_EXTRUSION

export function buildRuntimeManifest(source: ArtManifest): RuntimeArtManifest {
  assertValidSourceManifest(source)
  const sourceFrames = source.frames.filter(({ status }) => status === 'baseline')
  const frames: Array<Omit<ArtFrame, 'status'>> = []
  let x = MARGIN
  let y = MARGIN
  let rowHeight = 0
  let usedWidth = 0
  let usedHeight = 0
  for (const sourceFrame of sourceFrames) {
    const frame: Omit<ArtFrame, 'status'> = {
      id: sourceFrame.id,
      sourceRect: sourceFrame.sourceRect,
      localBounds: sourceFrame.localBounds,
      pivot: sourceFrame.pivot,
      ...(sourceFrame.footAnchor ? { footAnchor: sourceFrame.footAnchor } : {}),
      ...(sourceFrame.opacity === undefined ? {} : { opacity: sourceFrame.opacity }),
    }
    const footprintWidth = frame.sourceRect.width + MARGIN * 2
    const footprintHeight = frame.sourceRect.height + MARGIN * 2
    if (footprintWidth > MAX_TEXTURE_SIZE || footprintHeight > MAX_TEXTURE_SIZE) {
      throw new RangeError(`frame ${frame.id} exceeds the runtime texture size`)
    }
    if (x + frame.sourceRect.width + MARGIN > MAX_TEXTURE_SIZE) {
      x = MARGIN
      y += rowHeight
      rowHeight = 0
    }
    frames.push({
      ...frame,
      sourceRect: { ...frame.sourceRect, x, y },
      localBounds: { ...frame.localBounds },
      pivot: { ...frame.pivot },
      ...(frame.footAnchor ? { footAnchor: { ...frame.footAnchor } } : {}),
    })
    usedWidth = Math.max(usedWidth, x + frame.sourceRect.width + MARGIN)
    usedHeight = Math.max(usedHeight, y + frame.sourceRect.height + MARGIN)
    x += footprintWidth
    rowHeight = Math.max(rowHeight, footprintHeight)
  }
  const height = usedHeight
  if (height > MAX_TEXTURE_SIZE || usedWidth * height * 4 > MAX_RGBA_BYTES) {
    throw new RangeError('runtime atlas exceeds the 2048px or 16MiB RGBA texture budget')
  }
  return {
    schemaVersion: 1,
    texture: {
      width: usedWidth,
      height,
      padding: RUNTIME_PADDING,
      extrusion: RUNTIME_EXTRUSION,
    },
    frames,
    animations: source.animations,
    aliases: source.aliases,
    compositions: source.compositions,
  }
}
