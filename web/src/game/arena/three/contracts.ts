import type { BufferGeometry, Texture } from 'three'
import type { ArtManifest, RuntimeArtManifest } from '../artAtlas/types'
import type { SnapshotMessage } from '../types'
import type { PresentationDeathCue } from '../types'

export type FrameAnchor = 'pivot' | 'feet' | 'center' | 'paintedCenter'
export type RuntimeFrame = RuntimeArtManifest['frames'][number]

/** One renderer owns the atlas and its shared GPU geometry; entities only borrow it. */
export interface ThreeAtlas {
  texture: Texture
  sourceManifest: ArtManifest
  manifest: RuntimeArtManifest
  frame: (id: string) => RuntimeFrame
  geometry: (id: string, anchor?: FrameAnchor) => BufferGeometry
  dispose: () => void
}

export interface SceneRevision {
  snapshot: SnapshotMessage | null
  deaths: readonly PresentationDeathCue[]
  reducedMotion: boolean
  receivedAt: number
  serial: number
}

/** Snapshot changes notify React; visual interpolation runs through refs in the frame loop. */
export interface ArenaSceneStore {
  read: () => SceneRevision
  subscribe: (listener: () => void) => () => void
  update: (snapshot: SnapshotMessage, now: number) => void
  clear: () => void
  setReducedMotion: (reduced: boolean) => void
}
