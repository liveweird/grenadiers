import type { ErrorObject } from 'ajv'
import validateServer from '../../validation/generated/gameplay-v21.mjs'
import { cpuConfigurationInvalid } from '../validation/cpuConfigurationSemantics'
import { rules10SemanticError, type Rules10SemanticError } from '../validation/rules10Semantics'
import type { GameServerMessage } from './schema'

export type GameplayV21ValidationResult =
  | { valid: true; value: GameServerMessage }
  | { valid: false; source: 'schema'; errors: ErrorObject[] }
  | { valid: false; source: 'semantics'; error: Rules10SemanticError | 'invalidCpuDifficulty' }

export function validateGameplayV21ServerMessage(value: unknown): GameplayV21ValidationResult {
  if (!validateServer(value)) {
    return { valid: false, source: 'schema', errors: [...(validateServer.errors ?? [])] }
  }
  if (value.type !== 'admitted' && value.type !== 'snapshot' && value.type !== 'roomState') {
    return { valid: true, value }
  }
  if (cpuConfigurationInvalid(value.room)) {
    return { valid: false, source: 'semantics', error: 'invalidCpuDifficulty' }
  }
  const error = rules10SemanticError(value)
  return error ? { valid: false, source: 'semantics', error } : { valid: true, value }
}
