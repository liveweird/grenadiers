import { effectSemanticError, expandedProfileSemanticErrorFor, itemSetSemanticError,
  type GameplayProfileSemanticError } from './profileSemantics'
import { roomLifecycleSemanticError } from './roomLifecycleSemantics'
import type { GameServerMessage as V17GameServerMessage, GameState as V17GameState,
  RoomView as V17RoomView, Seat, WireEvent as V17WireEvent } from '../v17/schema'
import type { GameServerMessage as V18GameServerMessage, GameState as V18GameState,
  RoomView as V18RoomView, WireEvent as V18WireEvent } from '../v18/schema'
import type { GameServerMessage as V19GameServerMessage, GameState as V19GameState,
  RoomView as V19RoomView, WireEvent as V19WireEvent } from '../v19/schema'

type GameServerMessage = V17GameServerMessage | V18GameServerMessage | V19GameServerMessage
type GameState = V17GameState | V18GameState | V19GameState
type RoomView = V17RoomView | V18RoomView | V19RoomView
type WireEvent = V17WireEvent | V18WireEvent | V19WireEvent

const ITEM_IDS = [
  'kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb',
  'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame',
  'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls',
  'invisibleBombs', 'oilBarrel', 'teleporter',
] as const

export type BaseSemanticError = GameplayProfileSemanticError
  | 'invalidConfiguration' | 'invalidControllerMetadata' | 'invalidFrozenState'
  | 'invalidLairEnvironment'
  | 'invalidLifecycle' | 'invalidOilBarrels' | 'invalidPersonalAdmission' | 'invalidRoster'
  | 'invalidStateBinding' | 'invalidStormEnvironment' | 'invalidTeleportMarker' | 'invalidWorldState'

const SEATS: readonly Seat[] = ['one', 'two', 'three', 'four']
const UNITS_PER_CELL = 1024
const HALF_FOOTPRINT = 320
const cellKey = ({ column, row }: { column: number; row: number }): string => `${column},${row}`

function supportedRoomProfile(room: RoomView, rulesVersion: number): boolean {
  const { width, height } = room.configuration.board
  return ((width === 11 && height === 9) || (width === 13 && height === 11)
      || (width === 15 && height === 13))
    && room.rulesVersion === rulesVersion
    && room.configuration.controllers.length >= 2 && room.configuration.controllers.length <= 4
    && room.configuration.environment.stormCloud === room.settings.environment.stormCloud
}

function lairEnvironmentMismatch(room: RoomView): boolean {
  const configured = room.configuration.environment.lairs
  const selected = room.settings.environment.lairs
  return configured.enabled !== selected.enabled
    || configured.spawnSeconds !== selected.spawnSeconds
    || configured.maxLivingMobs !== selected.maxLivingMobs
}

function cellOutsideBoard(value: unknown, width: number, height: number): boolean {
  if (Array.isArray(value)) return value.some((child) => cellOutsideBoard(child, width, height))
  if (typeof value !== 'object' || value === null) return false
  const record = value as Record<string, unknown>
  if (typeof record.column === 'number' && typeof record.row === 'number'
    && (record.column < 0 || record.column >= width || record.row < 0 || record.row >= height)) return true
  return Object.values(record).some((child) => cellOutsideBoard(child, width, height))
}

function boardRelativeWorldInvalid(state: GameState): boolean {
  const { width, height } = state.arena
  if (cellOutsideBoard(state, width, height)) return true
  return state.actors.some((actor) => actor.type === 'visible'
    && (actor.position.x - HALF_FOOTPRINT < 0
      || actor.position.x + HALF_FOOTPRINT > width * UNITS_PER_CELL
      || actor.position.y - HALF_FOOTPRINT < 0
      || actor.position.y + HALF_FOOTPRINT > height * UNITS_PER_CELL))
}

function livingPlayersOverlap(state: GameState): boolean {
  const visible = state.actors.filter((actor): actor is Extract<typeof actor, { type: 'visible' }> =>
    actor.type === 'visible' && actor.alive)
  for (const [index, actor] of visible.entries()) {
    for (const other of visible.slice(index + 1)) {
      const sameCell = Math.floor(actor.position.x / UNITS_PER_CELL)
          === Math.floor(other.position.x / UNITS_PER_CELL)
        && Math.floor(actor.position.y / UNITS_PER_CELL)
          === Math.floor(other.position.y / UNITS_PER_CELL)
      const positiveAreaOverlap = Math.abs(actor.position.x - other.position.x) < 2 * HALF_FOOTPRINT
        && Math.abs(actor.position.y - other.position.y) < 2 * HALF_FOOTPRINT
      if (sameCell || positiveAreaOverlap) return true
    }
  }
  return false
}

function configurationError(room: RoomView): BaseSemanticError | null {
  const { controllers, cpuDifficulty } = room.configuration
  const expectedSeats = SEATS.slice(0, controllers.length)
  if (controllers[0] !== 'human'
    || room.seats.length !== controllers.length || room.scores.length !== controllers.length
    || room.seats.some(({ seat }, index) => seat !== expectedSeats[index])
    || room.scores.some(({ seat }, index) => seat !== expectedSeats[index])) return 'invalidConfiguration'
  const hasCpu = controllers.some((controller) => controller === 'cpu')
  return (hasCpu && cpuDifficulty === null) || (!hasCpu && cpuDifficulty !== null)
    ? 'invalidConfiguration' : null
}

function statusError(room: RoomView, index: number): BaseSemanticError | null {
  const status = room.seats[index]!
  const configured = room.configuration.controllers[index]
  const reserved = status.connection.type !== 'vacant'
  if (status.occupied !== reserved || (!status.occupied && (status.ready || status.rematchReady))) {
    return 'invalidControllerMetadata'
  }
  if (configured === 'cpu') {
    const ready = room.phase.type === 'waiting' || room.phase.type === 'countdown'
    const rematchReady = room.phase.type === 'matchResult' && room.tick >= room.phase.rematchAt
    return status.controller.type !== 'cpu' || !status.occupied
      || status.connection.type !== 'serverControlled'
      || status.controller.difficulty !== room.configuration.cpuDifficulty
      || status.ready !== ready || status.rematchReady !== rematchReady
      ? 'invalidControllerMetadata' : null
  }
  return status.controller.type !== 'human' || status.connection.type === 'serverControlled'
    ? 'invalidControllerMetadata' : null
}

function metadataError(room: RoomView): BaseSemanticError | null {
  const configuration = configurationError(room)
  if (configuration) return configuration
  for (const index of room.seats.keys()) if (statusError(room, index)) return 'invalidControllerMetadata'
  if (room.settingsEditor === null) return null
  const eligible = room.configuration.controllers[0] === 'human'
    && ['connected', 'disconnected'].includes(room.seats[0]?.connection.type ?? '')
  return room.settingsEditor === 'one' && eligible ? null : 'invalidControllerMetadata'
}

function oilError(state: GameState, room: RoomView): BaseSemanticError | null {
  if (state.oilBarrels.length > 0 && !room.settings.enabledItems.includes('oilBarrel')) return 'invalidOilBarrels'
  const flames = new Set(state.flames.map(({ cell }) => cellKey(cell)))
  const occupied = new Set<string>()
  let previous = -1
  let burning = 0
  for (const barrel of state.oilBarrels) {
    const order = barrel.cell.row * state.arena.width + barrel.cell.column
    const key = cellKey(barrel.cell)
    if (order <= previous || occupied.has(key) || flames.has(key)
      || state.arena.terrain[order] !== 'floor'
      || barrel.cell.column === 0 || barrel.cell.column === state.arena.width - 1
      || barrel.cell.row === 0 || barrel.cell.row === state.arena.height - 1
      || state.arena.blocks.some((cell) => cellKey(cell) === key)
      || state.pickups.some(({ cell }) => cellKey(cell) === key)
      || state.bombs.some(({ cell }) => cellKey(cell) === key)) return 'invalidOilBarrels'
    if (barrel.state === 'burning') burning += 1
    occupied.add(key); previous = order
  }
  return state.pickups.some(({ kind }) => kind === 'oilBarrel')
    || flames.size + burning > state.arena.width * state.arena.height ? 'invalidOilBarrels' : null
}

function frozenActorInvalid(
  actor: Extract<GameState['actors'][number], { type: 'visible' }>, state: GameState, room: RoomView,
): boolean {
  if (actor.voluntaryActionsFrozen && !room.settings.enabledItems.includes('stopwatch')) return true
  const visibleStopwatch = state.actors.some((other) => other.id !== actor.id
    && other.type === 'visible' && other.alive
    && other.effects.some(({ kind }) => kind === 'stopwatch'))
  const hiddenOpponent = state.actors.some((other) => other.id !== actor.id && other.type === 'hidden')
  return (!actor.alive && actor.voluntaryActionsFrozen)
    || (actor.alive && visibleStopwatch && !actor.voluntaryActionsFrozen)
    || (actor.alive && !visibleStopwatch && !hiddenOpponent && actor.voluntaryActionsFrozen)
}

function actorError(state: GameState, room: RoomView): BaseSemanticError | null {
  const ids = new Set<number>()
  const roundStart = state.physicalTick - state.runningUpdates
  const incompleteBombVisibility = state.actors.some((actor) => actor.type === 'visible'
    && actor.effects.some(({ kind }) => kind === 'invisibleBombs'))
  if (roundStart < 0) return 'invalidTeleportMarker'
  for (const actor of state.actors) {
    if (ids.has(actor.id)) return 'invalidRoster'
    ids.add(actor.id)
    if (actor.type !== 'visible') continue
    const error = effectSemanticError(actor.effects)
    if (error) return error
    if (frozenActorInvalid(actor, state, room)) return 'invalidFrozenState'
    const visibleRegularBombs = state.bombs.filter((bomb) =>
      bomb.owner === actor.id && bomb.trigger.type !== 'contact').length
    if (actor.liveBombCount < visibleRegularBombs || actor.liveBombCount > actor.bombCapacity
      || (!incompleteBombVisibility && actor.liveBombCount !== visibleRegularBombs)) {
      return 'invalidWorldState'
    }
    const marker = actor.lastTeleportTick
    if (marker !== null && (!room.settings.enabledItems.includes('teleporter')
      || marker <= roundStart || marker > state.physicalTick)) return 'invalidTeleportMarker'
  }
  return state.bombs.some(({ owner }) => !ids.has(owner)) ? 'invalidWorldState' : null
}

function eventError(state: GameState, events: readonly WireEvent[]): BaseSemanticError | null {
  for (const { kind } of events) {
    if ('id' in kind && 'seat' in kind) {
      const actor = state.actors.find(({ id }) => id === kind.id)
      if (!actor || actor.seat !== kind.seat) return 'invalidWorldState'
    }
    if ('bomb' in kind && !state.actors.some(({ id }) => id === kind.bomb.owner)) return 'invalidWorldState'
  }
  return null
}

function stormError(state: GameState, room: RoomView): BaseSemanticError | null {
  const enabled = room.settings.environment.stormCloud
  const visible = state.actors.filter((actor) => actor.type === 'visible')
  if (visible.some((actor) => !actor.alive && actor.stormSlowRemainingTicks !== 0)) {
    return 'invalidStormEnvironment'
  }
  if (!enabled) return state.stormCloud === null
    && visible.every((actor) => actor.stormSlowRemainingTicks === 0) ? null : 'invalidStormEnvironment'
  const cloud = state.stormCloud
  if (cloud === null) return 'invalidStormEnvironment'
  const { x, y } = cloud.position
  const maxX = (state.arena.width - 2) * UNITS_PER_CELL + UNITS_PER_CELL / 2
  const maxY = (state.arena.height - 2) * UNITS_PER_CELL + UNITS_PER_CELL / 2
  if (x < 1_536 || x > maxX || y < 1_536 || y > maxY) return 'invalidStormEnvironment'
  const progress = (128 - cloud.legRemainingTicks) * 8
  const centered = (value: number): boolean => value % UNITS_PER_CELL === UNITS_PER_CELL / 2
  const consistent = cloud.direction === 'north'
    ? centered(x) && (y + progress) % UNITS_PER_CELL === UNITS_PER_CELL / 2
    : cloud.direction === 'east'
      ? centered(y) && (x - progress + UNITS_PER_CELL) % UNITS_PER_CELL === UNITS_PER_CELL / 2
      : cloud.direction === 'south'
        ? centered(x) && (y - progress + UNITS_PER_CELL) % UNITS_PER_CELL === UNITS_PER_CELL / 2
        : centered(y) && (x + progress) % UNITS_PER_CELL === UNITS_PER_CELL / 2
  return consistent ? null : 'invalidStormEnvironment'
}

function worldError(
  state: GameState, room: RoomView, events: readonly WireEvent[],
): BaseSemanticError | null {
  const seats = room.seats.map(({ seat }) => seat)
  if (state.arena.width !== room.configuration.board.width
    || state.arena.height !== room.configuration.board.height
    || state.arena.terrain.length !== state.arena.width * state.arena.height
    || state.actors.length !== seats.length
    || state.actors.some(({ seat }, index) => seat !== seats[index])) return 'invalidRoster'
  if (boardRelativeWorldInvalid(state) || livingPlayersOverlap(state)
    || cellOutsideBoard(events, state.arena.width, state.arena.height)) {
    return 'invalidWorldState'
  }
  return stormError(state, room) ?? actorError(state, room) ?? eventError(state, events) ?? oilError(state, room)
}

function stateBinds(room: RoomView, state: GameState): boolean {
  const { phase } = room
  if (phase.type === 'running') return state.physicalTick === room.tick
  if (phase.type === 'result') return room.lastRound !== null && state.physicalTick === room.lastRound.endedAt
  if (phase.type !== 'matchResult') return false
  const completed = room.lastRound !== null && state.physicalTick === room.lastRound.endedAt
    && room.lastRound.endedAt <= phase.endedAt
  return completed || state.physicalTick === phase.endedAt
}

function phaseAllowsNullState(room: RoomView): boolean {
  const { phase } = room
  return phase.type === 'waiting' || phase.type === 'countdown' || phase.type === 'interrupted'
    || (phase.type === 'matchResult' && phase.result.type === 'interrupted'
      && phase.result.reason === 'internalError')
}

function stateEpochIsCurrent(room: RoomView, state: GameState): boolean {
  const { phase } = room
  if (phase.type === 'running') return true
  if (phase.type === 'result') return room.lastRound?.roundId === room.roundId
  if (phase.type !== 'matchResult') return false
  if (room.lastRound?.roundId === room.roundId
    && state.physicalTick === room.lastRound.endedAt) return true
  return state.physicalTick === phase.endedAt
    && (room.lastRound === null || room.lastRound.endedAt < phase.endedAt)
}

function stateBindingError(message: Extract<GameServerMessage,
  { type: 'admitted' | 'roomState' | 'snapshot' }>): BaseSemanticError | null {
  if (message.type === 'roomState') return null
  if (message.type === 'snapshot') return stateBinds(message.room, message.state) ? null : 'invalidStateBinding'
  if (message.state === null) {
    return message.lastActionId === null && phaseAllowsNullState(message.room)
      ? null : 'invalidStateBinding'
  }
  if (!stateBinds(message.room, message.state)) return 'invalidStateBinding'
  return message.lastActionId === null || stateEpochIsCurrent(message.room, message.state)
    ? null : 'invalidStateBinding'
}

export function baseSemanticErrorForRules(
  message: GameServerMessage, rulesVersion: number,
): BaseSemanticError | null {
  if (message.type !== 'admitted' && message.type !== 'roomState' && message.type !== 'snapshot') return null
  const { room } = message
  const metadata = metadataError(room)
  if (metadata) return metadata
  if (lairEnvironmentMismatch(room)) return 'invalidLairEnvironment'
  if (!supportedRoomProfile(room, rulesVersion)) return 'unsupportedProfile'
  const lifecycle = roomLifecycleSemanticError(room)
  if (lifecycle) return lifecycle
  const binding = stateBindingError(message)
  if (binding) return binding
  if (message.type === 'admitted') {
    const own = room.seats.find(({ seat }) => seat === message.youSeat)
    if (own?.controller.type !== 'human' || own.connection.type !== 'connected') return 'invalidPersonalAdmission'
  }
  const itemError = itemSetSemanticError(room.settings.enabledItems, room.availableItems)
  if (itemError) return itemError
  const state = message.type === 'roomState' ? null : message.state
  const events = message.type === 'snapshot' ? message.events : []
  if (events.some(({ kind }) => kind.type === 'pickupCollected' && kind.kind === 'oilBarrel')) {
    return 'invalidOilBarrels'
  }
  return state && worldError(state, room, events)
    || expandedProfileSemanticErrorFor(room, state, events, ITEM_IDS, {
      allowPausedTimed: true, rulesVersions: [rulesVersion],
    })
}

export function baseSemanticError(message: GameServerMessage): BaseSemanticError | null {
  return baseSemanticErrorForRules(message, 9)
}
