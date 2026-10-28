import type { ErrorObject } from 'ajv'
import validateServer from '../../validation/generated/gameplay-v14.mjs'
import { effectSemanticError, expandedProfileSemanticErrorFor, itemSetSemanticError,
  type GameplayProfileSemanticError } from '../validation/profileSemantics'
import { ITEM_IDS, type GameServerMessage, type GameState, type RoomView, type Seat,
  type WireEvent } from './schema'

type SemanticError = GameplayProfileSemanticError
  | 'invalidConfiguration' | 'invalidControllerMetadata' | 'invalidFrozenState'
  | 'invalidOilBarrels' | 'invalidPersonalAdmission' | 'invalidRoster'
  | 'invalidTeleportMarker' | 'invalidWorldState'
export type GameplayV14ValidationResult =
  | { valid: true; value: GameServerMessage }
  | { valid: false; source: 'schema'; errors: ErrorObject[] }
  | { valid: false; source: 'semantics'; error: SemanticError }

const SEATS: readonly Seat[] = ['one', 'two', 'three', 'four']
const cellKey = ({ column, row }: { column: number; row: number }): string => `${column},${row}`

function configurationError(room: RoomView): SemanticError | null {
  const { controllers, cpuDifficulty } = room.configuration
  const expectedSeats = SEATS.slice(0, controllers.length)
  if (controllers[0] !== 'human'
    || room.seats.length !== controllers.length || room.scores.length !== controllers.length
    || room.seats.some(({ seat }, index) => seat !== expectedSeats[index])
    || room.scores.some(({ seat }, index) => seat !== expectedSeats[index])) return 'invalidConfiguration'
  const hasCpu = controllers.some((controller) => controller === 'cpu')
  return (hasCpu && cpuDifficulty === null) || (!hasCpu && cpuDifficulty !== null)
    ? 'invalidConfiguration' : null
}

function statusError(room: RoomView, index: number): SemanticError | null {
  const status = room.seats[index]!
  const configured = room.configuration.controllers[index]
  if (configured === 'cpu') {
    const ready = room.phase.type === 'waiting' || room.phase.type === 'countdown'
    const rematchReady = room.phase.type === 'matchResult' && room.tick >= room.phase.rematchAt
    return status.controller.type !== 'cpu' || !status.occupied
      || status.connection.type !== 'serverControlled'
      || status.controller.difficulty !== room.configuration.cpuDifficulty
      || status.ready !== ready || status.rematchReady !== rematchReady
      ? 'invalidControllerMetadata' : null
  }
  return status.controller.type !== 'human' || status.connection.type === 'serverControlled'
    ? 'invalidControllerMetadata' : null
}

function metadataError(room: RoomView): SemanticError | null {
  const configuration = configurationError(room)
  if (configuration) return configuration
  for (const index of room.seats.keys()) {
    if (statusError(room, index)) return 'invalidControllerMetadata'
  }
  const ownerEligible = ['connected', 'disconnected'].includes(room.seats[0]?.connection.type ?? '')
  return room.settingsEditor === (ownerEligible ? 'one' : null) ? null : 'invalidControllerMetadata'
}

function oilError(state: GameState, room: RoomView): SemanticError | null {
  if (state.oilBarrels.length > 0 && !room.settings.enabledItems.includes('oilBarrel')) return 'invalidOilBarrels'
  const flames = new Set(state.flames.map(({ cell }) => cellKey(cell)))
  const occupied = new Set<string>()
  let previous = -1
  let burning = 0
  for (const barrel of state.oilBarrels) {
    const order = barrel.cell.row * state.arena.width + barrel.cell.column
    const key = cellKey(barrel.cell)
    if (order <= previous || occupied.has(key) || flames.has(key)
      || state.arena.terrain[order] !== 'floor'
      || barrel.cell.column === 0 || barrel.cell.column === state.arena.width - 1
      || barrel.cell.row === 0 || barrel.cell.row === state.arena.height - 1
      || state.arena.blocks.some((cell) => cellKey(cell) === key)
      || state.pickups.some(({ cell }) => cellKey(cell) === key)
      || state.bombs.some(({ cell }) => cellKey(cell) === key)) return 'invalidOilBarrels'
    if (barrel.state === 'burning') burning += 1
    occupied.add(key); previous = order
  }
  return state.pickups.some(({ kind }) => kind === 'oilBarrel')
    || flames.size + burning > state.arena.width * state.arena.height ? 'invalidOilBarrels' : null
}

function frozenStateInvalid(
  actor: Extract<GameState['actors'][number], { type: 'visible' }>, state: GameState, room: RoomView,
): boolean {
  if (actor.voluntaryActionsFrozen && !room.settings.enabledItems.includes('stopwatch')) return true
  const visibleStopwatch = state.actors.some((other) => other.id !== actor.id
    && other.type === 'visible' && other.alive
    && other.effects.some(({ kind }) => kind === 'stopwatch'))
  const hiddenOpponent = state.actors.some((other) => other.id !== actor.id && other.type === 'hidden')
  return (!actor.alive && actor.voluntaryActionsFrozen)
    || (actor.alive && visibleStopwatch && !actor.voluntaryActionsFrozen)
    || (actor.alive && !visibleStopwatch && !hiddenOpponent && actor.voluntaryActionsFrozen)
}

function actorError(state: GameState, room: RoomView): SemanticError | null {
  const ids = new Set<number>()
  const roundStart = state.physicalTick - state.runningUpdates
  if (roundStart < 0) return 'invalidTeleportMarker'
  for (const actor of state.actors) {
    if (ids.has(actor.id)) return 'invalidRoster'
    ids.add(actor.id)
    if (actor.type !== 'visible') continue
    const error = effectSemanticError(actor.effects)
    if (error) return error
    if (frozenStateInvalid(actor, state, room)) return 'invalidFrozenState'
    const marker = actor.lastTeleportTick
    if (marker !== null && (!room.settings.enabledItems.includes('teleporter')
      || marker <= roundStart || marker > state.physicalTick)) return 'invalidTeleportMarker'
  }
  return state.bombs.some(({ owner }) => !ids.has(owner)) ? 'invalidWorldState' : null
}

function eventError(state: GameState, events: WireEvent[]): SemanticError | null {
  for (const { kind } of events) {
    if ('id' in kind && 'seat' in kind) {
      const actor = state.actors.find(({ id }) => id === kind.id)
      if (!actor || actor.seat !== kind.seat) return 'invalidWorldState'
    }
    if ('bomb' in kind && !state.actors.some(({ id }) => id === kind.bomb.owner)) return 'invalidWorldState'
  }
  return null
}

function worldError(state: GameState, room: RoomView, events: WireEvent[]): SemanticError | null {
  const expectedSeats = room.seats.map(({ seat }) => seat)
  if (state.arena.width !== room.configuration.board.width
    || state.arena.height !== room.configuration.board.height
    || state.arena.terrain.length !== state.arena.width * state.arena.height
    || state.actors.length !== expectedSeats.length
    || state.actors.some(({ seat }, index) => seat !== expectedSeats[index])) return 'invalidRoster'
  return actorError(state, room) ?? eventError(state, events) ?? oilError(state, room)
}

function semanticError(message: GameServerMessage): SemanticError | null {
  if (message.type !== 'admitted' && message.type !== 'roomState' && message.type !== 'snapshot') return null
  const room = message.room
  const metadata = metadataError(room)
  if (metadata) return metadata
  if (room.rulesVersion !== 6
    && (room.rulesVersion !== 5 || room.configuration.controllers.length !== 2)) {
    return 'unsupportedProfile'
  }
  if (message.type === 'admitted') {
    const own = room.seats.find(({ seat }) => seat === message.youSeat)
    if (own?.controller.type !== 'human' || own.connection.type !== 'connected') return 'invalidPersonalAdmission'
  }
  const itemError = itemSetSemanticError(room.settings.enabledItems, room.availableItems)
  if (itemError) return itemError
  const state = message.type === 'roomState' ? null : message.state
  const events = message.type === 'snapshot' ? message.events : []
  if (events.some(({ kind }) => kind.type === 'pickupCollected' && kind.kind === 'oilBarrel')) {
    return 'invalidOilBarrels'
  }
  return state && worldError(state, room, events)
    || expandedProfileSemanticErrorFor(room, state, events, ITEM_IDS, {
      allowPausedTimed: true, rulesVersions: [5, 6],
    })
}

export function validateGameplayV14ServerMessage(value: unknown): GameplayV14ValidationResult {
  if (!validateServer(value)) {
    return { valid: false, source: 'schema', errors: [...(validateServer.errors ?? [])] }
  }
  const error = semanticError(value)
  return error ? { valid: false, source: 'semantics', error } : { valid: true, value }
}
