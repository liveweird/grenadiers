import type { GameState, RoomView, Snapshot } from '../v20/schema'

function cellKey(cell: { column: number; row: number }): string {
  return `${cell.column},${cell.row}`
}

/** Pit cells are construction facts; only their authoritative phases and timers may change. */
export function preservesSpikePitLayout(
  previous: Snapshot | null, room: RoomView, next: GameState | null,
): boolean {
  if (!previous || !next || previous.room.roomId !== room.roomId
    || previous.room.roundId !== room.roundId) return true
  const priorCells = previous.state.spikePits.map(({ cell }) => cellKey(cell))
  const nextCells = next.spikePits.map(({ cell }) => cellKey(cell))
  return priorCells.length === nextCells.length
    && priorCells.every((cell, index) => cell === nextCells[index])
}
