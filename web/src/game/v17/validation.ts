import type { ErrorObject } from 'ajv'
import validateServer from '../../validation/generated/gameplay-v17.mjs'
import { baseSemanticError, type BaseSemanticError } from './baseSemantics'
import type { GameServerMessage, GameState, RoomView } from './schema'

type SemanticError = BaseSemanticError
  | 'invalidLairEnvironment' | 'invalidLairState' | 'invalidWorldState'

export type GameplayV17ValidationResult =
  | { valid: true; value: GameServerMessage }
  | { valid: false; source: 'schema'; errors: ErrorObject[] }
  | { valid: false; source: 'semantics'; error: SemanticError }

function lairLayout(room: RoomView): readonly [number, number][] {
  const { width, height } = room.configuration.board
  if (width === 15 && height === 13) return [[6, 6], [8, 6]]
  if (width === 13 && height === 11) return [[6, 4], [6, 6]]
  return [[4, 4], [6, 4]]
}

function entityError(state: GameState): SemanticError | null {
  const ids = new Set(state.actors.map(({ id }) => id))
  const bombCells = new Set<string>()
  for (const bomb of state.bombs) {
    const key = `${bomb.cell.column},${bomb.cell.row}`
    if (ids.has(bomb.id) || bombCells.has(key)) return 'invalidWorldState'
    ids.add(bomb.id)
    bombCells.add(key)
  }
  for (const mob of state.mobs) {
    if (ids.has(mob.id) || mob.lairId > 1
      || mob.position.x < 320 || mob.position.x > state.arena.width * 1024 - 320
      || mob.position.y < 320 || mob.position.y > state.arena.height * 1024 - 320) return 'invalidWorldState'
    ids.add(mob.id)
  }
  return null
}

function lairFactsInvalid(state: GameState, room: RoomView): boolean {
  const configured = room.configuration.environment.lairs
  const layout = lairLayout(room)
  const counts = [0, 1].map((id) => state.mobs.filter((mob) => mob.lairId === id).length)
  if (state.lairs.length !== 2) return true
  for (const [index, lair] of state.lairs.entries()) {
    const [column, row] = layout[index]!
    const terrain = state.arena.terrain[row * state.arena.width + column]
    if (lair.id !== index || lair.cell.column !== column || lair.cell.row !== row
      || terrain !== 'wall'
      || (lair.livingMobs === 0 && (lair.spawnRemainingTicks === null
        || lair.spawnRemainingTicks > configured.spawnSeconds * 60))
      || (lair.livingMobs === 1 && lair.spawnRemainingTicks !== null)
      || lair.livingMobs !== counts[index]
      || lair.livingMobs > 1) return true
  }
  return false
}

function frozenStateInvalid(state: GameState): boolean {
  if (state.mobs.some((mob) => mob.frozen !== state.mobs[0]?.frozen)) return true
  const visibleStopwatch = state.actors.some((actor) => actor.type === 'visible' && actor.alive
    && actor.effects.some(({ kind }) => kind === 'stopwatch'))
  const hiddenActor = state.actors.some((actor) => actor.type === 'hidden')
  return (visibleStopwatch && state.mobs.some((mob) => !mob.frozen))
    || (!visibleStopwatch && !hiddenActor && state.mobs.some((mob) => mob.frozen))
}

function lairError(state: GameState, room: RoomView): SemanticError | null {
  const configured = room.configuration.environment.lairs
  if (!configured.enabled) {
    return state.lairs.length === 0 && state.mobs.length === 0 ? null : 'invalidLairState'
  }
  const entities = entityError(state)
  if (entities) return entities
  if (lairFactsInvalid(state, room) || frozenStateInvalid(state)) return 'invalidLairState'
  return null
}

export function validateGameplayV17ServerMessage(value: unknown): GameplayV17ValidationResult {
  if (!validateServer(value)) {
    return { valid: false, source: 'schema', errors: [...(validateServer.errors ?? [])] }
  }
  const baseError = baseSemanticError(value)
  if (baseError) return { valid: false, source: 'semantics', error: baseError }
  if (value.type !== 'admitted' && value.type !== 'snapshot') return { valid: true, value }
  const error = value.state && lairError(value.state, value.room)
  return error ? { valid: false, source: 'semantics', error } : { valid: true, value }
}
