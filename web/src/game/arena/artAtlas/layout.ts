import type { ArtFrame, ArtRect } from './types'

export const TILE_SIZE = 48 as const
export const AUTHORING_SLOT_SIZE = 112 as const
export const AUTHORING_COLUMNS = 10 as const
export const RUNTIME_PADDING = 2 as const
export const RUNTIME_EXTRUSION = 2 as const
export const MAX_TEXTURE_SIZE = 2048
export const MAX_RGBA_BYTES = 16 * 1024 * 1024

export function sourceFrame(
  id: string,
  slot: number,
  dimensions: { width: number; height: number },
  localBounds: ArtRect,
  options: Pick<ArtFrame, 'status' | 'pivot' | 'footAnchor' | 'opacity'>,
): ArtFrame {
  return {
    id,
    status: options.status,
    sourceRect: {
      x: (slot % AUTHORING_COLUMNS) * AUTHORING_SLOT_SIZE,
      y: Math.floor(slot / AUTHORING_COLUMNS) * AUTHORING_SLOT_SIZE,
      ...dimensions,
    },
    localBounds,
    pivot: options.pivot,
    ...(options.footAnchor ? { footAnchor: options.footAnchor } : {}),
    ...(options.opacity === undefined ? {} : { opacity: options.opacity }),
  }
}

export function authoringSheetHeight(frameCount: number): number {
  return Math.ceil(frameCount / AUTHORING_COLUMNS) * AUTHORING_SLOT_SIZE
}
