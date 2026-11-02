import type { GameState, RoomView, Snapshot } from '../v20/schema'
import { preservesTeleportHistory } from './teleportHistory'
import { preservesSpikePitLayout } from './spikePitHistory'

/** Validate immutable construction facts and monotonic actor history at one acceptance boundary. */
export function preservesSnapshotHistory(
  previous: Snapshot | null, room: RoomView, next: GameState | null,
): boolean {
  return preservesTeleportHistory(previous, room, next) && preservesSpikePitLayout(previous, room, next)
}
