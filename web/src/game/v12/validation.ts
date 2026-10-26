import type { ErrorObject } from 'ajv'
import validateServer from '../../validation/generated/gameplay-v12.mjs'
import type { GameplayProfileSemanticError } from '../validation/profileSemantics'
import { validateGameplayV11ServerMessage } from '../v11/validation'
import type { GameServerMessage, RoomView } from './schema'

type GameplayV12SemanticError =
  | GameplayProfileSemanticError
  | 'invalidControllerMetadata'
  | 'invalidFrozenState'
  | 'invalidOilBarrels'
  | 'invalidPersonalAdmission'
  | 'invalidTeleportMarker'
export type GameplayV12ValidationResult =
  | { valid: true; value: GameServerMessage }
  | { valid: false; source: 'schema'; errors: ErrorObject[] }
  | { valid: false; source: 'semantics'; error: GameplayV12SemanticError }

function metadataError(room: RoomView): GameplayV12SemanticError | null {
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

function v11PhysicalMessage(message: GameServerMessage): unknown {
  const normalized = structuredClone(message) as unknown as Record<string, unknown>
  const room = normalized.room as Record<string, unknown>
  delete room.mode
  const seats = room.seats as Array<Record<string, unknown>>
  for (const seat of seats) {
    delete seat.controller
    const connection = seat.connection as Record<string, unknown>
    if (connection.type === 'serverControlled') connection.type = 'connected'
  }
  return normalized
}

export function validateGameplayV12ServerMessage(value: unknown): GameplayV12ValidationResult {
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
  const physical = validateGameplayV11ServerMessage(v11PhysicalMessage(value))
  return physical.valid
    ? { valid: true, value }
    : physical.source === 'schema'
      ? { valid: false, source: 'schema', errors: physical.errors }
      : { valid: false, source: 'semantics', error: physical.error }
}
