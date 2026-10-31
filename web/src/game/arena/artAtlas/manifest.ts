import { createArtDescriptor } from './descriptor'
import { AUTHORING_COLUMNS, AUTHORING_SLOT_SIZE, TILE_SIZE, authoringSheetHeight } from './layout'
import type { ArtCatalog, ArtManifest } from './types'

export function createSourceManifest(catalog?: ArtCatalog): ArtManifest {
  const descriptor = catalog ? {
    frames: catalog.entries.map(({ frame }) => frame),
    animations: catalog.animations,
    aliases: catalog.aliases,
    compositions: catalog.compositions,
  } : createArtDescriptor()
  return {
    schemaVersion: 1,
    tileSize: TILE_SIZE,
    sheet: {
      width: AUTHORING_COLUMNS * AUTHORING_SLOT_SIZE,
      height: authoringSheetHeight(descriptor.frames.length),
      slotSize: AUTHORING_SLOT_SIZE,
      columns: AUTHORING_COLUMNS,
    },
    frames: descriptor.frames.map((frame) => ({
      ...frame,
      sourceRect: { ...frame.sourceRect },
      localBounds: { ...frame.localBounds },
      pivot: { ...frame.pivot },
      ...(frame.footAnchor ? { footAnchor: { ...frame.footAnchor } } : {}),
    })),
    animations: descriptor.animations,
    aliases: descriptor.aliases,
    compositions: descriptor.compositions,
  }
}
