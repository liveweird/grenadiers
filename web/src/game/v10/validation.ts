import Ajv, { type ErrorObject, type ValidateFunction } from 'ajv'
import gameplayV10Schema from '../../../../api/gameplay-v10.schema.json'
import { effectSemanticError, expandedProfileSemanticErrorFor, itemSetSemanticError,
  type GameplayV8SemanticError } from '../v8/semantics'
import { freezeSemanticError } from '../v9/freezeValidation'
import type { GameServerMessage, GameState, RoomView } from './schema'

type GameplayV10SemanticError = GameplayV8SemanticError
  | 'invalidFrozenState' | 'invalidTeleportMarker'
export type GameplayV10ValidationResult =
  | { valid: true; value: GameServerMessage }
  | { valid: false; source: 'schema'; errors: ErrorObject[] }
  | { valid: false; source: 'semantics'; error: GameplayV10SemanticError }

const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV10Schema)
const schemaReference = `${gameplayV10Schema.$id}#/$defs/GameServerMessage`
const resolved = ajv.getSchema<GameServerMessage>(schemaReference)
if (!resolved) throw new Error(`AJV did not resolve ${schemaReference}`)
const validateServer: ValidateFunction<GameServerMessage> = resolved

const supportedItems = [
  'kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch',
  'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame',
  'longFuse', 'reverseControls', 'invisibleBombs', 'teleporter',
] as const

function stateSemanticError(
  state: GameState | null, room: RoomView,
): GameplayV10SemanticError | null {
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
  return null
}

function messageSemanticError(message: GameServerMessage): GameplayV10SemanticError | null {
  if (message.type !== 'admitted' && message.type !== 'roomState' && message.type !== 'snapshot') return null
  const roomError = itemSetSemanticError(message.room.settings.enabledItems, message.room.availableItems)
  if (roomError) return roomError
  const state = message.type === 'roomState' ? null : message.state
  const events = message.type === 'snapshot' ? message.events : []
  return stateSemanticError(state, message.room)
    ?? expandedProfileSemanticErrorFor(message.room, state, events, supportedItems, true)
}

export function validateGameplayV10ServerMessage(value: unknown): GameplayV10ValidationResult {
  if (!validateServer(value)) {
    return { valid: false, source: 'schema', errors: [...(validateServer.errors ?? [])] }
  }
  const error = messageSemanticError(value)
  return error ? { valid: false, source: 'semantics', error } : { valid: true, value }
}
