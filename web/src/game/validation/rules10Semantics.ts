import { baseSemanticErrorForRules, type BaseSemanticError } from './baseSemantics'
import type { Cell as V18Cell, GameServerMessage as V18GameServerMessage,
  GameState as V18GameState, RoomView as V18RoomView } from '../v18/schema'
import type { Cell as V19Cell, GameServerMessage as V19GameServerMessage,
  GameState as V19GameState, RoomView as V19RoomView } from '../v19/schema'
import type { Cell as V20Cell, GameServerMessage as V20GameServerMessage,
  GameState as V20GameState, RoomView as V20RoomView } from '../v20/schema'

type Cell = V18Cell | V19Cell | V20Cell
type GameServerMessage = V18GameServerMessage | V19GameServerMessage | V20GameServerMessage
type GameState = V18GameState | V19GameState | V20GameState
type RoomView = V18RoomView | V19RoomView | V20RoomView

export type Rules10SemanticError = BaseSemanticError
  | 'invalidLairState' | 'invalidSpikePitState' | 'invalidWorldState'

function lairLayout(room: RoomView): readonly [number, number][] {
  const { width, height } = room.configuration.board
  if (width === 15 && height === 13) return [[6, 6], [8, 6]]
  if (width === 13 && height === 11) return [[6, 4], [6, 6]]
  return [[4, 4], [6, 4]]
}

function entityError(state: GameState): Rules10SemanticError | null {
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
  return state.lairs.some((lair, index) => {
    const [column, row] = layout[index]!
    const terrain = state.arena.terrain[row * state.arena.width + column]
    return lair.id !== index || lair.cell.column !== column || lair.cell.row !== row
      || terrain !== 'wall'
      || (lair.livingMobs === 0 && (lair.spawnRemainingTicks === null
        || lair.spawnRemainingTicks > configured.spawnSeconds * 60))
      || (lair.livingMobs === 1 && lair.spawnRemainingTicks !== null)
      || lair.livingMobs !== counts[index] || lair.livingMobs > 1
  })
}

function frozenStateInvalid(state: GameState): boolean {
  if (state.mobs.some((mob) => mob.frozen !== state.mobs[0]?.frozen)) return true
  const visibleStopwatch = state.actors.some((actor) => actor.type === 'visible' && actor.alive
    && actor.effects.some(({ kind }) => kind === 'stopwatch'))
  const hiddenActor = state.actors.some((actor) => actor.type === 'hidden')
  return (visibleStopwatch && state.mobs.some((mob) => !mob.frozen))
    || (!visibleStopwatch && !hiddenActor && state.mobs.some((mob) => mob.frozen))
}

function lairError(state: GameState, room: RoomView): Rules10SemanticError | null {
  if (!room.configuration.environment.lairs.enabled) {
    return state.lairs.length === 0 && state.mobs.length === 0 ? null : 'invalidLairState'
  }
  return lairFactsInvalid(state, room) || frozenStateInvalid(state) ? 'invalidLairState' : null
}

function spikeEnvironmentError(room: RoomView): Rules10SemanticError | null {
  const configured = room.configuration.environment.spikePits
  const selected = room.settings.environment.spikePits
  return configured.enabled === selected.enabled
    && configured.warningTicks === selected.warningTicks
    && configured.raisedTicks === selected.raisedTicks
    ? null : 'invalidSpikePitState'
}

function cellKey(cell: Cell): string { return `${cell.column},${cell.row}` }

function validPitCell(state: GameState, room: RoomView, cell: Cell): boolean {
  const { width, height } = room.configuration.board
  const index = cell.row * width + cell.column
  const spawnReserved = [[1, 1], [width - 2, 1], [1, height - 2], [width - 2, height - 2]]
    .some(([column, row]) => Math.abs(column! - cell.column) + Math.abs(row! - cell.row) <= 1)
  return cell.column > 0 && cell.row > 0 && cell.column + 1 < width && cell.row + 1 < height
    && state.arena.terrain[index] === 'floor'
    && !state.arena.blocks.some((block) => cellKey(block) === cellKey(cell))
    && !state.lairs.some((lair) => cellKey(lair.cell) === cellKey(cell))
    && !state.oilBarrels.some((barrel) => cellKey(barrel.cell) === cellKey(cell))
    && !spawnReserved
}

function pitError(state: GameState, room: RoomView): Rules10SemanticError | null {
  const configured = room.configuration.environment.spikePits
  if (!configured.enabled) return state.spikePits.length === 0 ? null : 'invalidSpikePitState'
  if (![0, 2, 4].includes(state.spikePits.length)) return 'invalidSpikePitState'
  const keys = new Set(state.spikePits.map(({ cell }) => cellKey(cell)))
  for (const [index, pit] of state.spikePits.entries()) {
    const previous = state.spikePits[index - 1]
    if (previous && (previous.cell.row > pit.cell.row
      || (previous.cell.row === pit.cell.row && previous.cell.column >= pit.cell.column))) {
      return 'invalidSpikePitState'
    }
    const reflected = [
      `${room.configuration.board.width - 1 - pit.cell.column},${pit.cell.row}`,
      `${pit.cell.column},${room.configuration.board.height - 1 - pit.cell.row}`,
      `${room.configuration.board.width - 1 - pit.cell.column},${room.configuration.board.height - 1 - pit.cell.row}`,
    ]
    const timerValid = pit.phase === 'idle' ? pit.remainingTicks === 0
      : pit.phase === 'warning' ? pit.remainingTicks <= configured.warningTicks
        : pit.remainingTicks <= configured.raisedTicks
    if (!validPitCell(state, room, pit.cell) || !reflected.every((key) => keys.has(key)) || !timerValid) {
      return 'invalidSpikePitState'
    }
  }
  return null
}

export function rules10SemanticError(message: GameServerMessage): Rules10SemanticError | null {
  const baseError = baseSemanticErrorForRules(message, 10)
  if (baseError) return baseError
  if (message.type !== 'admitted' && message.type !== 'snapshot' && message.type !== 'roomState') {
    return null
  }
  const environmentError = spikeEnvironmentError(message.room)
  if (environmentError) return environmentError
  if (message.type === 'roomState' || !message.state) return null
  return entityError(message.state) ?? lairError(message.state, message.room)
    ?? pitError(message.state, message.room)
}
