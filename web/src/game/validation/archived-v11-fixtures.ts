// Archived wire tests retain v11 metadata while sharing unchanged physical fixtures.
import type { Admitted, RoomView, Snapshot } from '../v11/schema'
import {
  admitted as liveAdmission, room as liveRoom, snapshot as liveSnapshot,
} from '../client/testSupport'

function archivedRoom(value: ReturnType<typeof liveRoom>): RoomView {
  const { mode, seats, ...room } = value
  if (mode.type !== 'humanDuel') throw new Error('A solo room cannot become an archived fixture')
  return { ...room, seats: seats.map(({ controller, connection, ...seat }) => {
    if (controller.type !== 'human' || connection.type === 'serverControlled') {
      throw new Error('Archived fixtures require human controllers')
    }
    return { ...seat, connection }
  }) }
}

export function room(...args: Parameters<typeof liveRoom>): RoomView {
  return archivedRoom(liveRoom(...args))
}

export function snapshot(...args: Parameters<typeof liveSnapshot>): Snapshot {
  const value = liveSnapshot(...args)
  return { ...value, room: archivedRoom(value.room) }
}

export function admitted(...args: Parameters<typeof liveAdmission>): Admitted {
  const value = liveAdmission(...args)
  return { ...value, room: archivedRoom(value.room) }
}
