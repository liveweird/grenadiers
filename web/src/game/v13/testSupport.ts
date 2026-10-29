import type { Admitted, RoomPhase, RoomView, Snapshot } from './schema'
import { admitted as currentAdmission, room as currentRoom, snapshot as currentSnapshot } from '../v14/testSupport'

function legacyRoom(value: ReturnType<typeof currentRoom>): RoomView {
  const { configuration, scores, ...rest } = value
  if (configuration.controllers.length !== 2) throw new Error('v13 fixtures require two controllers')
  const difficulty = configuration.cpuDifficulty
  return {
    ...rest, rulesVersion: 5,
    mode: difficulty === null ? { type: 'humanDuel' } : { type: 'solo', difficulty },
    scores: { one: scores.find(({ seat }) => seat === 'one')?.wins ?? 0,
      two: scores.find(({ seat }) => seat === 'two')?.wins ?? 0 },
    seats: rest.seats.map((seat) => ({ ...seat, seat: seat.seat as 'one' | 'two' })),
  } as RoomView
}

export function room(revision: number, phase: RoomPhase = { type: 'waiting' }, roundId = 1): RoomView {
  return legacyRoom(currentRoom(revision, phase, roundId))
}

export function snapshot(revision: number, roundId = 1): Snapshot {
  const value = currentSnapshot(revision, roundId)
  return { ...value, room: legacyRoom(value.room) } as unknown as Snapshot
}

export function admitted(revision = 1, lastActionId: number | null = null): Admitted {
  const value = currentAdmission(revision, lastActionId)
  return { ...value, room: legacyRoom(value.room) } as unknown as Admitted
}
