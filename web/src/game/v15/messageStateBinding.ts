import type { GameServerMessage, GameState, RoomView } from './schema'

type RoomMessage = Extract<GameServerMessage, { type: 'admitted' | 'roomState' | 'snapshot' }>

function stateBinds(room: RoomView, state: GameState): boolean {
  const { phase } = room
  if (phase.type === 'running') return state.physicalTick === room.tick
  if (phase.type === 'result') {
    return room.lastRound !== null && state.physicalTick === room.lastRound.endedAt
  }
  if (phase.type !== 'matchResult') return false
  const completed = room.lastRound !== null
    && state.physicalTick === room.lastRound.endedAt
    && room.lastRound.endedAt <= phase.endedAt
  return completed || state.physicalTick === phase.endedAt
}

function phaseAllowsNullAdmission(room: RoomView): boolean {
  const { phase } = room
  return phase.type === 'waiting' || phase.type === 'countdown' || phase.type === 'interrupted'
    || (phase.type === 'matchResult' && phase.result.type === 'interrupted'
      && phase.result.reason === 'internalError')
}

function stateEpochIsCurrent(room: RoomView, state: GameState): boolean {
  const { phase } = room
  if (phase.type === 'running') return true
  if (phase.type === 'result') return room.lastRound?.roundId === room.roundId
  if (phase.type !== 'matchResult') return false
  if (room.lastRound?.roundId === room.roundId
    && state.physicalTick === room.lastRound.endedAt) return true
  return state.physicalTick === phase.endedAt
    && (room.lastRound === null || room.lastRound.endedAt < phase.endedAt)
}

export function messageStateBindingError(message: RoomMessage): 'invalidStateBinding' | null {
  if (message.type === 'roomState') return null
  if (message.type === 'snapshot') {
    return stateBinds(message.room, message.state) ? null : 'invalidStateBinding'
  }
  if (message.state === null) {
    return message.lastActionId === null && phaseAllowsNullAdmission(message.room)
      ? null : 'invalidStateBinding'
  }
  if (!stateBinds(message.room, message.state)) return 'invalidStateBinding'
  return message.lastActionId === null || stateEpochIsCurrent(message.room, message.state)
    ? null : 'invalidStateBinding'
}
