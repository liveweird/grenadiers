import type { RoomView } from '../v20/schema'
import { format, type GameCopy } from './locale'

export function controllersAvailable(room: RoomView): boolean {
  const active = room.seats.filter(({ connection }) => connection.type !== 'expired')
  return active.length >= 2 && active.some(({ controller, connection }) =>
    controller.type === 'human' && connection.type === 'connected')
    && active.every((seat) => seat.controller.type === 'cpu'
      ? seat.connection.type === 'serverControlled' : seat.connection.type === 'connected')
}

export function isLegacySolo(room: RoomView): boolean {
  const controllers = room.configuration.controllers
  return controllers.length === 2 && controllers[0] === 'human' && controllers[1] === 'cpu'
}

export function roomCopy(copy: GameCopy, room: RoomView | null): GameCopy {
  if (!room) return copy
  const controllers = room.configuration.controllers
  if (isLegacySolo(room)) {
    return { ...copy, label: copy.solo.label,
      player: { ...copy.player, two: copy.solo.opponent } }
  }
  if (!controllers.some((controller) => controller === 'cpu')) return copy
  const player = { ...copy.player }
  controllers.forEach((controller, index) => {
    const seat = (['one', 'two', 'three', 'four'] as const)[index]
    if (controller === 'cpu') player[seat] = format(copy.multiplayer.cpuPlayer, { player: player[seat] })
  })
  return { ...copy, player }
}
