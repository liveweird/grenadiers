import type { ErrorObject } from 'ajv'
import validateServer from '../../validation/generated/gameplay-v18.mjs'
import { rules10SemanticError, type Rules10SemanticError } from '../validation/rules10Semantics'
import type { GameServerMessage } from './schema'

export type GameplayV18ValidationResult =
  | { valid: true; value: GameServerMessage }
  | { valid: false; source: 'schema'; errors: ErrorObject[] }
  | { valid: false; source: 'semantics'; error: Rules10SemanticError }

export function validateGameplayV18ServerMessage(value: unknown): GameplayV18ValidationResult {
  if (!validateServer(value)) {
    return { valid: false, source: 'schema', errors: [...(validateServer.errors ?? [])] }
  }
  const error = rules10SemanticError(value)
  return error
    ? { valid: false, source: 'semantics', error }
    : { valid: true, value }
}
