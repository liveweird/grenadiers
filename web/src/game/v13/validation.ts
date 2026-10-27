import type { ErrorObject } from 'ajv'
import validateServer from '../../validation/generated/gameplay-v13.mjs'
import type { GameplayProfileSemanticError } from '../validation/profileSemantics'
import { validateGameplayV12ServerMessage } from '../v12/validation'
import type { GameServerMessage, RoomView } from './schema'

type RoomMessage = Extract<GameServerMessage, { type: 'admitted' | 'roomState' | 'snapshot' }>

type GameplayV13SemanticError =
  | GameplayProfileSemanticError
  | 'invalidControllerMetadata'
  | 'invalidFrozenState'
  | 'invalidOilBarrels'
  | 'invalidPersonalAdmission'
  | 'invalidTeleportMarker'
export type GameplayV13ValidationResult =
  | { valid: true; value: GameServerMessage }
  | { valid: false; source: 'schema'; errors: ErrorObject[] }
  | { valid: false; source: 'semantics'; error: GameplayV13SemanticError }

function metadataError(room: RoomView): GameplayV13SemanticError | null {
  const [one, two] = room.seats
  if (!one || !two || one.seat !== 'one' || two.seat !== 'two') {
    return 'invalidControllerMetadata'
  }
  if (room.mode.type === 'humanDuel') {
    return room.seats.every(({ controller, connection }) =>
      controller.type === 'human' && connection.type !== 'serverControlled')
      ? null : 'invalidControllerMetadata'
  }
  const humanEligible = one.connection.type === 'connected'
    || one.connection.type === 'disconnected'
  const validEditor = room.settingsEditor === (humanEligible ? 'one' : null)
  const valid = validEditor
    && one.controller.type === 'human'
    && one.connection.type !== 'serverControlled'
    && two.controller.type === 'cpu'
    && two.controller.difficulty === room.mode.difficulty
    && two.occupied
    && two.connection.type === 'serverControlled'
  return valid ? null : 'invalidControllerMetadata'
}

function v12PhysicalMessage(message: RoomMessage): unknown {
  const normalized = structuredClone(message)
  if (normalized.room.mode.type === 'solo') normalized.room.mode.difficulty = 'standard'
  for (const seat of normalized.room.seats) {
    if (seat.controller.type === 'cpu') seat.controller.difficulty = 'standard'
  }
  return normalized
}

export function validateGameplayV13ServerMessage(value: unknown): GameplayV13ValidationResult {
  if (!validateServer(value)) {
    return { valid: false, source: 'schema', errors: [...(validateServer.errors ?? [])] }
  }
  if (value.type !== 'admitted' && value.type !== 'roomState' && value.type !== 'snapshot') {
    return { valid: true, value }
  }
  const metadata = metadataError(value.room)
  if (metadata) return { valid: false, source: 'semantics', error: metadata }
  if (value.type === 'admitted' && value.room.mode.type === 'solo'
    && (value.youSeat !== 'one' || value.room.seats[0]?.connection.type !== 'connected')) {
    return { valid: false, source: 'semantics', error: 'invalidPersonalAdmission' }
  }
  const physical = validateGameplayV12ServerMessage(v12PhysicalMessage(value))
  return physical.valid
    ? { valid: true, value }
    : physical.source === 'schema'
      ? { valid: false, source: 'schema', errors: physical.errors }
      : { valid: false, source: 'semantics', error: physical.error }
}
