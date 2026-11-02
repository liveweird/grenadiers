import type { GameState, RoomView, Snapshot } from '../v20/schema'

type Visible = Extract<GameState['actors'][number], { type: 'visible' }>
function changedMarker(prior: Visible, actor: Visible): number | null | false {
  if (prior.lastTeleportTick !== null
    && (actor.lastTeleportTick === null || actor.lastTeleportTick < prior.lastTeleportTick)) {
    return false
  }
  return actor.lastTeleportTick !== null && actor.lastTeleportTick !== prior.lastTeleportTick
    ? actor.lastTeleportTick : null
}

export function preservesTeleportHistory(
  previous: Snapshot | null, room: RoomView, next: GameState | null,
): boolean {
  if (!previous || !next || previous.room.roomId !== room.roomId
    || previous.room.roundId !== room.roundId) return true
  const privacyLimited = previous.state.actors.some(({ type }) => type === 'hidden')
    || next.actors.some(({ type }) => type === 'hidden')
  if (previous.state.actors.length !== next.actors.length
    || next.actors.some((actor) => !previous.state.actors.some((prior) =>
      prior.id === actor.id && prior.seat === actor.seat))) return false
  const changed: number[] = []
  for (const actor of next.actors) {
    if (actor.type !== 'visible') continue
    const prior = previous.state.actors.find((candidate) => candidate.id === actor.id
      && candidate.seat === actor.seat)
    if (!prior || prior.type !== 'visible') continue
    const change = changedMarker(prior, actor)
    if (change === false) return false
    if (change !== null) changed.push(change)
  }
  if (changed.length === 0) return true
  const latest = Math.max(...changed)
  const latestParticipants = changed.filter((tick) => tick === latest).length
  return latestParticipants <= 2 && (privacyLimited || latestParticipants === 2)
}
