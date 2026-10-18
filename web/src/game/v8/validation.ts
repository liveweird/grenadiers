import Ajv, { type ErrorObject, type ValidateFunction } from 'ajv'
import gameplayV8Schema from '../../../../api/gameplay-v8.schema.json'
import { effectSemanticError, expandedProfileSemanticError, itemSetSemanticError,
  type GameplayV8SemanticError } from './semantics'
import type { GameServerMessage, GameState, RoomView } from './schema'

export type GameplayV8ValidationResult =
  | { valid: true; value: GameServerMessage }
  | { valid: false; source: 'schema'; errors: ErrorObject[] }
  | { valid: false; source: 'semantics'; error: GameplayV8SemanticError }

const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV8Schema)
const schemaReference = `${gameplayV8Schema.$id}#/$defs/GameServerMessage`
function resolveServerValidator(): ValidateFunction<GameServerMessage> {
  const resolved = ajv.getSchema<GameServerMessage>(schemaReference)
  if (!resolved) throw new Error(`AJV did not resolve ${schemaReference}`)
  return resolved
}
const validateServer = resolveServerValidator()

function stateSemanticError(state: GameState | null): GameplayV8SemanticError | null {
  if (!state) return null
  for (const actor of state.actors) {
    if (actor.type === 'visible') {
      const error = effectSemanticError(actor.effects)
      if (error) return error
    }
  }
  return null
}

function roomSemanticError(room: RoomView): GameplayV8SemanticError | null {
  return itemSetSemanticError(room.settings.enabledItems, room.availableItems)
}

function messageSemanticError(message: GameServerMessage): GameplayV8SemanticError | null {
  switch (message.type) {
    case 'admitted':
      return roomSemanticError(message.room) ?? stateSemanticError(message.state)
        ?? expandedProfileSemanticError(message.room, message.state)
    case 'roomState':
      return roomSemanticError(message.room) ?? expandedProfileSemanticError(message.room, null)
    case 'snapshot':
      return roomSemanticError(message.room) ?? stateSemanticError(message.state)
        ?? expandedProfileSemanticError(message.room, message.state, message.events)
    default:
      return null
  }
}

export function validateGameplayV8ServerMessage(value: unknown): GameplayV8ValidationResult {
  if (!validateServer(value)) {
    return { valid: false, source: 'schema', errors: [...(validateServer.errors ?? [])] }
  }
  const error = messageSemanticError(value)
  return error ? { valid: false, source: 'semantics', error } : { valid: true, value }
}

export function gameplayV8ServerSchemaValidator(): ValidateFunction<GameServerMessage> {
  return validateServer
}
