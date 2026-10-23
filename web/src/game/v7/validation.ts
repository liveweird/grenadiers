import Ajv, { type ErrorObject, type ValidateFunction } from 'ajv'
import gameplayV7Schema from '../../../../api/gameplay-v7.schema.json'
import { effectSemanticError, itemSetSemanticError, type GameplayV7SemanticError } from './semantics'
import type { GameServerMessage, GameState, RoomView, WireEvent } from './schema'

export type GameplayV7ValidationResult =
  | { valid: true; value: GameServerMessage }
  | { valid: false; source: 'schema'; errors: ErrorObject[] }
  | { valid: false; source: 'semantics'; error: GameplayV7SemanticError }

const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV7Schema)
const schemaReference = `${gameplayV7Schema.$id}#/$defs/GameServerMessage`
function resolveServerValidator(): ValidateFunction<GameServerMessage> {
  const resolved = ajv.getSchema<GameServerMessage>(schemaReference)
  if (!resolved) throw new Error(`AJV did not resolve ${schemaReference}`)
  return resolved
}

const validateServer = resolveServerValidator()

function semanticErrorForState(
  state: GameState | null, events: readonly WireEvent[] = [],
): GameplayV7SemanticError | null {
  if (!state) return null
  if (state.pickups.some(({ kind }) => kind === 'wallPass')
    || state.actors.some((actor) => actor.type === 'visible'
      && actor.inventory.equipment === 'wallPass')
    || events.some(({ kind }) => kind.type === 'pickupCollected' && kind.kind === 'wallPass')) {
    return 'unsupportedProfile'
  }
  for (const actor of state.actors) {
    if (actor.type === 'visible') {
      const error = effectSemanticError(actor.effects)
      if (error) return error
    }
  }
  return null
}

function semanticErrorForRoom(room: RoomView): GameplayV7SemanticError | null {
  if (room.availableItems.includes('wallPass') || room.settings.enabledItems.includes('wallPass')) {
    return 'unsupportedProfile'
  }
  return itemSetSemanticError(room.settings.enabledItems, room.availableItems)
}

function semanticErrorForMessage(message: GameServerMessage): GameplayV7SemanticError | null {
  switch (message.type) {
    case 'admitted':
      return semanticErrorForRoom(message.room) ?? semanticErrorForState(message.state)
    case 'roomState':
      return semanticErrorForRoom(message.room)
    case 'snapshot':
      return semanticErrorForRoom(message.room) ?? semanticErrorForState(message.state, message.events)
    default:
      return null
  }
}

export function validateGameplayV7ServerMessage(value: unknown): GameplayV7ValidationResult {
  if (!validateServer(value)) {
    return { valid: false, source: 'schema', errors: [...(validateServer.errors ?? [])] }
  }
  const error = semanticErrorForMessage(value)
  return error ? { valid: false, source: 'semantics', error } : { valid: true, value }
}

export function gameplayV7ServerSchemaValidator(): ValidateFunction<GameServerMessage> {
  return validateServer
}
