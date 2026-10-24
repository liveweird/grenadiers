import { createArtCatalog } from './catalog'
import { AUTHORING_COLUMNS, AUTHORING_SLOT_SIZE, TILE_SIZE, authoringSheetHeight } from './layout'
import type { ArtCatalog, ArtManifest } from './types'

export function createSourceManifest(catalog: ArtCatalog = createArtCatalog()): ArtManifest {
  return {
    schemaVersion: 1,
    tileSize: TILE_SIZE,
    sheet: {
      width: AUTHORING_COLUMNS * AUTHORING_SLOT_SIZE,
      height: authoringSheetHeight(catalog.entries.length),
      slotSize: AUTHORING_SLOT_SIZE,
      columns: AUTHORING_COLUMNS,
    },
    frames: catalog.entries.map(({ frame }) => ({
      ...frame,
      sourceRect: { ...frame.sourceRect },
      localBounds: { ...frame.localBounds },
      pivot: { ...frame.pivot },
      ...(frame.footAnchor ? { footAnchor: { ...frame.footAnchor } } : {}),
    })),
    animations: catalog.animations,
    aliases: catalog.aliases,
    compositions: catalog.compositions,
  }
}
