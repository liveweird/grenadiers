import type { SnapshotMessage } from './types'

/** Presentation only: renderer time never advances authoritative game state. */
export interface ArenaRenderer {
  update: (snapshot: SnapshotMessage) => void
  suspend: () => void
  setReducedMotion: (reduced: boolean) => void
  destroy: () => void
}
