import Ajv, { type ErrorObject, type ValidateFunction } from 'ajv'
import gameplayV9Schema from '../../../../api/gameplay-v9.schema.json'
import { effectSemanticError, expandedProfileSemanticErrorFor, itemSetSemanticError,
  type GameplayProfileSemanticError } from '../validation/profileSemantics'
import type { GameServerMessage, GameState, RoomView } from './schema'
import { freezeSemanticError } from './freezeValidation'

type GameplayV9SemanticError = GameplayProfileSemanticError | 'invalidFrozenState'
export type GameplayV9ValidationResult =
  | { valid: true; value: GameServerMessage }
  | { valid: false; source: 'schema'; errors: ErrorObject[] }
  | { valid: false; source: 'semantics'; error: GameplayV9SemanticError }

const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV9Schema)
const schemaReference = `${gameplayV9Schema.$id}#/$defs/GameServerMessage`
function resolveServerValidator(): ValidateFunction<GameServerMessage> {
  const resolved = ajv.getSchema<GameServerMessage>(schemaReference)
  if (!resolved) throw new Error(`AJV did not resolve ${schemaReference}`)
  return resolved
}
const validateServer = resolveServerValidator()

const supportedItems = [
  'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch',
  'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame',
  'longFuse', 'reverseControls', 'invisibleBombs',
] as const

function stateSemanticError(
  state: GameState | null, stopwatchEnabled: boolean,
): GameplayV9SemanticError | null {
  if (!state) return null
  for (const actor of state.actors) {
    if (actor.type === 'visible') {
      const error = effectSemanticError(actor.effects)
      if (error) return error
    }
  }
  return freezeSemanticError(state.actors, stopwatchEnabled) ? 'invalidFrozenState' : null
}

function roomSemanticError(room: RoomView): GameplayV9SemanticError | null {
  return itemSetSemanticError(room.settings.enabledItems, room.availableItems)
}

function messageSemanticError(message: GameServerMessage): GameplayV9SemanticError | null {
  switch (message.type) {
    case 'admitted': {
      const enabled = message.room.settings.enabledItems.includes('stopwatch')
      return roomSemanticError(message.room) ?? stateSemanticError(message.state, enabled)
        ?? expandedProfileSemanticErrorFor(message.room, message.state, [], supportedItems, true)
    }
    case 'roomState':
      return roomSemanticError(message.room)
        ?? expandedProfileSemanticErrorFor(message.room, null, [], supportedItems, true)
    case 'snapshot': {
      const enabled = message.room.settings.enabledItems.includes('stopwatch')
      return roomSemanticError(message.room) ?? stateSemanticError(message.state, enabled)
        ?? expandedProfileSemanticErrorFor(
          message.room, message.state, message.events, supportedItems, true,
        )
    }
    default:
      return null
  }
}

export function validateGameplayV9ServerMessage(value: unknown): GameplayV9ValidationResult {
  if (!validateServer(value)) {
    return { valid: false, source: 'schema', errors: [...(validateServer.errors ?? [])] }
  }
  const error = messageSemanticError(value)
  return error ? { valid: false, source: 'semantics', error } : { valid: true, value }
}

