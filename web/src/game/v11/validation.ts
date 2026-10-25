import type { ErrorObject } from 'ajv'
import validateServer from '../../validation/generated/gameplay-v11.mjs'
import { freezeSemanticError } from '../validation/freezeSemantics'
import { effectSemanticError, expandedProfileSemanticErrorFor, itemSetSemanticError,
  type GameplayProfileSemanticError } from '../validation/profileSemantics'
import { ITEM_IDS, type GameServerMessage, type GameState, type RoomView } from './schema'

type GameplayV11SemanticError = GameplayProfileSemanticError
  | 'invalidFrozenState' | 'invalidTeleportMarker' | 'invalidOilBarrels'
export type GameplayV11ValidationResult =
  | { valid: true; value: GameServerMessage }
  | { valid: false; source: 'schema'; errors: ErrorObject[] }
  | { valid: false; source: 'semantics'; error: GameplayV11SemanticError }

const cellKey = ({ column, row }: { column: number; row: number }): string => `${column},${row}`
function oilSemanticError(state: GameState, room: RoomView): GameplayV11SemanticError | null {
  if (state.oilBarrels.length > 0 && !room.settings.enabledItems.includes('oilBarrel')) {
    return 'invalidOilBarrels'
  }
  const finite = new Set<string>()
  for (const flame of state.flames) {
    const key = cellKey(flame.cell)
    if (finite.has(key)) return 'invalidOilBarrels'
    finite.add(key)
  }
  const occupied = new Set<string>()
  let burning = 0
  let previous = -1
  for (const barrel of state.oilBarrels) {
    const order = barrel.cell.row * state.arena.width + barrel.cell.column
    const key = cellKey(barrel.cell)
    const interior = barrel.cell.column > 0 && barrel.cell.column < state.arena.width - 1
      && barrel.cell.row > 0 && barrel.cell.row < state.arena.height - 1
    if (!interior || state.arena.terrain[order] !== 'floor' || order <= previous || occupied.has(key)
      || finite.has(key) || state.pickups.some((pickup) => cellKey(pickup.cell) === key)
      || state.arena.blocks.some((block) => cellKey(block) === key)
      || state.bombs.some((bomb) => cellKey(bomb.cell) === key)) {
      return 'invalidOilBarrels'
    }
    if (barrel.state === 'burning') burning += 1
    occupied.add(key)
    previous = order
  }
  return state.pickups.some(({ kind }) => kind === 'oilBarrel') || finite.size + burning > 195
    ? 'invalidOilBarrels' : null
}

function stateSemanticError(state: GameState | null, room: RoomView): GameplayV11SemanticError | null {
  if (!state) return null
  for (const actor of state.actors) {
    if (actor.type === 'visible') {
      const error = effectSemanticError(actor.effects)
      if (error) return error
    }
  }
  if (freezeSemanticError(state.actors, room.settings.enabledItems.includes('stopwatch'))) {
    return 'invalidFrozenState'
  }
  const visible = state.actors.filter((actor) => actor.type === 'visible')
  const markers = visible.map((actor) => actor.lastTeleportTick)
  const roundStart = state.physicalTick - state.runningUpdates
  if (roundStart < 0 || markers.some((marker) => marker !== null
    && (!room.settings.enabledItems.includes('teleporter')
      || marker <= roundStart || marker > state.physicalTick))) return 'invalidTeleportMarker'
  if (markers.length === 2 && markers[0] !== markers[1]) return 'invalidTeleportMarker'
  return oilSemanticError(state, room)
}

function messageSemanticError(message: GameServerMessage): GameplayV11SemanticError | null {
  if (message.type !== 'admitted' && message.type !== 'roomState' && message.type !== 'snapshot') return null
  const roomError = itemSetSemanticError(message.room.settings.enabledItems, message.room.availableItems)
  if (roomError) return roomError
  const state = message.type === 'roomState' ? null : message.state
  const events = message.type === 'snapshot' ? message.events : []
  if (events.some(({ kind }) => kind.type === 'pickupCollected' && kind.kind === 'oilBarrel')) {
    return 'invalidOilBarrels'
  }
  return stateSemanticError(state, message.room)
    ?? expandedProfileSemanticErrorFor(message.room, state, events, ITEM_IDS, true)
}

export function validateGameplayV11ServerMessage(value: unknown): GameplayV11ValidationResult {
  if (!validateServer(value)) {
    return { valid: false, source: 'schema', errors: [...(validateServer.errors ?? [])] }
  }
  const error = messageSemanticError(value)
  return error ? { valid: false, source: 'semantics', error } : { valid: true, value }
}
