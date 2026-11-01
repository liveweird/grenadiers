import type { ErrorObject } from 'ajv'
import validateServer from '../../validation/generated/gameplay-v19.mjs'
import { rules10SemanticError, type Rules10SemanticError } from '../validation/rules10Semantics'
import type { GameServerMessage, RoomView } from './schema'

export type GameplayV19ValidationResult =
  | { valid: true; value: GameServerMessage }
  | { valid: false; source: 'schema'; errors: ErrorObject[] }
  | { valid: false; source: 'semantics'; error: Rules10SemanticError | 'invalidCpuDifficulty' }

function cpuDifficultyInvalid(room: RoomView): boolean {
  const { controllers, cpuDifficulty } = room.configuration
  const hasCpu = controllers.some((controller) => controller === 'cpu')
  if ((hasCpu && cpuDifficulty === null) || (!hasCpu && cpuDifficulty !== null)) return true
  return controllers.some((controller, index) => {
    const published = room.seats[index]?.controller
    return controller === 'cpu'
      ? published?.type !== 'cpu' || published.difficulty !== cpuDifficulty
      : published?.type !== 'human'
  })
}

export function validateGameplayV19ServerMessage(value: unknown): GameplayV19ValidationResult {
  if (!validateServer(value)) {
    return { valid: false, source: 'schema', errors: [...(validateServer.errors ?? [])] }
  }
  if (value.type !== 'admitted' && value.type !== 'snapshot' && value.type !== 'roomState') {
    return { valid: true, value }
  }
  if (cpuDifficultyInvalid(value.room)) {
    return { valid: false, source: 'semantics', error: 'invalidCpuDifficulty' }
  }
  const error = rules10SemanticError(value)
  return error ? { valid: false, source: 'semantics', error } : { valid: true, value }
}
