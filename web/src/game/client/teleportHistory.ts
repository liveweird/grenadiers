import type { GameState, RoomView, Snapshot } from '../v11/schema'

export function preservesTeleportHistory(
  previous: Snapshot | null, room: RoomView, next: GameState | null,
): boolean {
  if (!previous || !next || previous.room.roomId !== room.roomId
    || previous.room.roundId !== room.roundId) return true
  for (const actor of next.actors) {
    if (actor.type !== 'visible') continue
    const prior = previous.state.actors.find((candidate) => candidate.id === actor.id
      && candidate.seat === actor.seat)
    if (!prior || prior.type !== 'visible' || prior.lastTeleportTick === null) continue
    if (actor.lastTeleportTick === null || actor.lastTeleportTick < prior.lastTeleportTick) return false
  }
  return true
}
