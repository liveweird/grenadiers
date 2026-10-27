import type { RoomView } from '../v13/schema'
import type { GameCopy } from './locale'

export function controllersAvailable(room: RoomView): boolean {
  return room.seats.every((seat) => seat.controller.type === 'cpu'
    ? seat.connection.type === 'serverControlled' : seat.connection.type === 'connected')
}

export function roomCopy(copy: GameCopy, room: RoomView | null): GameCopy {
  if (room?.mode.type !== 'solo') return copy
  return { ...copy, label: copy.solo.label, player: { ...copy.player, two: copy.solo.opponent } }
}
