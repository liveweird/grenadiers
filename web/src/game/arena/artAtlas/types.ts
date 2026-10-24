import type { Container } from 'pixi.js'

export interface ArtRect { x: number; y: number; width: number; height: number }
interface ArtPoint { x: number; y: number }
type ArtFrameStatus = 'baseline' | 'reserved'

export interface ArtFrame {
  id: string
  status: ArtFrameStatus
  sourceRect: ArtRect
  localBounds: ArtRect
  pivot: ArtPoint
  footAnchor?: ArtPoint
  opacity?: number
}

export interface ArtAnimation {
  frameIds: readonly string[]
  frameDurationMs: number
  reducedMotionFrame: string
}

export interface ArtManifest {
  schemaVersion: 1
  tileSize: 48
  sheet: { width: number; height: number; slotSize: 112; columns: 10 }
  frames: readonly ArtFrame[]
  animations: Readonly<Record<string, ArtAnimation>>
  aliases: Readonly<Record<string, string>>
  compositions: Readonly<Record<string, readonly string[]>>
}

interface ArtDrawingLease {
  drawing: Container
  destroy(): void
}

export interface ArtCatalogEntry {
  frame: ArtFrame
  createDrawing(): ArtDrawingLease
}

export interface ArtCatalog {
  entries: readonly ArtCatalogEntry[]
  animations: Readonly<Record<string, ArtAnimation>>
  aliases: Readonly<Record<string, string>>
  compositions: Readonly<Record<string, readonly string[]>>
}

export interface PixelBuffer {
  width: number
  height: number
  data: Uint8Array | Uint8ClampedArray
}

export interface RuntimeArtManifest {
  schemaVersion: 1
  texture: { width: number; height: number; padding: 2; extrusion: 2 }
  frames: readonly Omit<ArtFrame, 'status'>[]
  animations: Readonly<Record<string, ArtAnimation>>
  aliases: Readonly<Record<string, string>>
  compositions: Readonly<Record<string, readonly string[]>>
}

export interface ValidationIssue {
  code: string
  path: string
  message: string
}

export class ArtAtlasValidationError extends Error {
  readonly issues: readonly ValidationIssue[]

  constructor(issues: readonly ValidationIssue[]) {
    super(issues.map(({ path, message }) => `${path}: ${message}`).join('\n'))
    this.issues = issues
    this.name = 'ArtAtlasValidationError'
  }
}
