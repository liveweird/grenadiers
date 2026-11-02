import type { Difficulty, Direction, GameServerMessage, GameSettings, RoomConfiguration,
  RoomView, Seat, V14ErrorCode } from '../v20/schema'
import { validateGameplayV20ServerMessage } from '../v20/validation'
import type { GameClientState } from './clientTypes'

export const MAX_COUNTER = 2_147_483_647
export const MAX_RECONNECT_ATTEMPTS = 3
export const MAX_SERVER_FRAME_BYTES = 196_608

export function parseServerFrame(raw: unknown): unknown {
  if (typeof raw !== 'string' || new TextEncoder().encode(raw).byteLength > MAX_SERVER_FRAME_BYTES) {
    throw new Error('invalid server frame')
  }
  return JSON.parse(raw) as unknown
}

export interface Dependencies {
  fetch: typeof fetch
  socket: (url: string) => WebSocket
  location: Location
  schedule: (callback: () => void, delay: number) => ReturnType<typeof setTimeout>
  cancel: (timer: ReturnType<typeof setTimeout>) => void
  now: () => number
}

export type Capability = { roomId: string; seat: Seat; token: string }
export type RoomCommand =
  | { type: 'ready'; ready: boolean; settingsRevision: number }
  | { type: 'movement'; direction: Direction | null }
  | { type: 'manualAction'; actionId: number; action: 'bomb' }
  | { type: 'rematch'; ready: boolean }
export type ControlCommand = { sequence: number; roomId: string; roundId: number; revision: number }
export type PendingSettings = {
  roomId: string
  roundId: number
  settingsRevision: number
  settings: GameSettings
}
export type ControlFailure = ControlCommand & { code: V14ErrorCode }
type ConfigurationIntent = { configuration?: RoomConfiguration; mode?: {
  type: 'humanDuel'
} | { type: 'solo'; difficulty: Difficulty } }

const TOKEN_PATTERN = /^[0-9a-f]{64}$/

export function createConfiguration(intent: ConfigurationIntent): RoomConfiguration {
  if (intent.configuration) return intent.configuration
  const difficulty = intent.mode?.type === 'solo' ? intent.mode.difficulty : null
  return difficulty === null
    ? { controllers: ['human', 'human'], cpuDifficulty: null, board: { width: 15, height: 13 },
      environment: { stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 },
        spikePits: { enabled: true, warningTicks: 36, raisedTicks: 180 } } }
    : { controllers: ['human', 'cpu'], cpuDifficulty: difficulty, board: { width: 15, height: 13 },
      environment: { stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 },
        spikePits: { enabled: true, warningTicks: 36, raisedTicks: 180 } } }
}
function unique<T>(values: readonly T[]): boolean {
  return new Set(values).size === values.length
}

function stateRecipientValid(message: GameServerMessage, expectedSeat: Seat | null): boolean {
  if (message.type !== 'snapshot' && message.type !== 'admitted') return true
  const state = message.state
  if (!state) return true
  const actorIds = state.actors.map((actor) => actor.id)
  const actorSeats = state.actors.map((actor) => actor.seat)
  const entityIds = [...actorIds, ...state.bombs.map((bomb) => bomb.id)]
  if (!unique(entityIds) || !unique(actorSeats)) return false
  const ownSeat = message.type === 'admitted' ? message.youSeat : expectedSeat
  if (ownSeat === null) return true
  const cloakEnabled = message.room.settings.enabledItems.includes('invisibilityCloak')
  const own = state.actors.find((actor) => actor.seat === ownSeat)
  return own?.type === 'visible'
    && (!own.effects.some((effect) => effect.kind === 'invisibleBombs') || state.bombs.length === 0)
    && state.actors.every((actor) => {
    if (actor.seat === ownSeat) return actor.type === 'visible'
    if (actor.type === 'hidden') return cloakEnabled && actor.alive
    return !actor.effects.some((effect) => effect.kind === 'invisibilityCloak')
  })
}

function roomRecipientValid(message: GameServerMessage): boolean {
  if (message.type !== 'snapshot' && message.type !== 'admitted' && message.type !== 'roomState') return true
  return unique(message.room.seats.map((seat) => seat.seat))
}

export function validateServerMessage(value: unknown, expectedSeat: Seat | null): value is GameServerMessage {
  const result = validateGameplayV20ServerMessage(value)
  return result.valid && roomRecipientValid(result.value) && stateRecipientValid(result.value, expectedSeat)
}

export function dependenciesWithDefaults(overrides?: Partial<Dependencies>): Dependencies {
  return {
    fetch: overrides?.fetch ?? globalThis.fetch.bind(globalThis),
    socket: overrides?.socket ?? ((url) => new WebSocket(url)),
    location: overrides?.location ?? window.location,
    schedule: overrides?.schedule ?? ((callback, delay) => setTimeout(callback, delay)),
    cancel: overrides?.cancel ?? ((timer) => clearTimeout(timer)),
    now: overrides?.now ?? (() => performance.now()),
  }
}

export function gameplayUrl(location: Location = window.location): string {
  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${protocol}//${location.host}/api/v1/game/ws`
}

export function sessionToken(value: unknown): string | null {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) return null
  if (Object.keys(value).length !== 1 || !Object.hasOwn(value, 'token')) return null
  const token = Reflect.get(value, 'token')
  return typeof token === 'string' && TOKEN_PATTERN.test(token) ? token : null
}

export function sameGameSettings(left: GameSettings, right: GameSettings): boolean {
  return left.startingSpeedLevel === right.startingSpeedLevel
    && left.turnAssistUnits === right.turnAssistUnits
    && left.winsToFinish === right.winsToFinish
    && left.bombFuseTicks === right.bombFuseTicks
    && left.enabledItems.length === right.enabledItems.length
    && left.enabledItems.every((item, index) => item === right.enabledItems[index])
    && left.environment.stormCloud === right.environment.stormCloud
    && left.environment.lairs.enabled === right.environment.lairs.enabled
    && left.environment.lairs.spawnSeconds === right.environment.lairs.spawnSeconds
    && left.environment.lairs.maxLivingMobs === right.environment.lairs.maxLivingMobs
    && left.environment.spikePits.enabled === right.environment.spikePits.enabled
    && left.environment.spikePits.warningTicks === right.environment.spikePits.warningTicks
    && left.environment.spikePits.raisedTicks === right.environment.spikePits.raisedTicks
}

function validBaseSettings(settings: GameSettings): boolean {
  return Number.isInteger(settings.startingSpeedLevel)
    && settings.startingSpeedLevel >= 0 && settings.startingSpeedLevel <= 4
    && [64, 128, 192, 256].includes(settings.turnAssistUnits)
    && Number.isInteger(settings.winsToFinish)
    && settings.winsToFinish >= 1 && settings.winsToFinish <= 9
    && [60, 90, 120, 150, 180, 210, 240, 270, 300].includes(settings.bombFuseTicks)
}

function validLairSettings(lairs: GameSettings['environment']['lairs']): boolean {
  return Object.keys(lairs ?? {}).length === 3 && typeof lairs?.enabled === 'boolean'
    && Number.isInteger(lairs.spawnSeconds) && lairs.spawnSeconds >= 5 && lairs.spawnSeconds <= 60
    && lairs.maxLivingMobs === 1
}

function validSpikePitSettings(pits: GameSettings['environment']['spikePits']): boolean {
  return Object.keys(pits ?? {}).length === 3 && typeof pits?.enabled === 'boolean'
    && Number.isInteger(pits.warningTicks) && pits.warningTicks >= 6 && pits.warningTicks <= 120
    && Number.isInteger(pits.raisedTicks) && pits.raisedTicks >= 60 && pits.raisedTicks <= 600
}

export function validGameSettings(settings: GameSettings): boolean {
  const environment = settings.environment
  const lairs = environment?.lairs
  const spikePits = environment?.spikePits
  return Object.keys(settings).length === 6
    && Object.keys(environment ?? {}).length === 3
    && typeof environment?.stormCloud === 'boolean'
    && lairs !== undefined && spikePits !== undefined && validBaseSettings(settings)
    && validLairSettings(lairs) && validSpikePitSettings(spikePits)
}

export function settlesPendingSettings(
  pending: PendingSettings | null,
  next: RoomView,
  current: RoomView,
): boolean {
  return Boolean(pending && next.roomId === pending.roomId && next.roomId === current.roomId
    && next.roundId === pending.roundId && next.roundId === current.roundId
    && next.revision === current.revision && next.settingsRevision === pending.settingsRevision
    && sameGameSettings(current.settings, pending.settings)
    && sameGameSettings(next.settings, pending.settings))
}

export function acceptableBaseline(next: RoomView, current: RoomView | null): boolean {
  return !current || (next.roomId === current.roomId && next.revision >= current.revision
    && next.roundId >= current.roundId && settingsRevisionValid(next, current)
    && sameConfiguration(next, current))
}

export function newerRoom(next: RoomView, current: RoomView): boolean {
  return next.roomId === current.roomId && next.revision > current.revision && next.roundId >= current.roundId
    && settingsRevisionValid(next, current) && sameConfiguration(next, current)
}

function settingsRevisionValid(next: RoomView, current: RoomView): boolean {
  return next.settingsRevision > current.settingsRevision
    || (next.settingsRevision === current.settingsRevision
      && sameGameSettings(next.settings, current.settings))
}

function sameConfiguration(left: RoomView, right: RoomView): boolean {
  return left.configuration.cpuDifficulty === right.configuration.cpuDifficulty
    && left.configuration.board.width === right.configuration.board.width
    && left.configuration.board.height === right.configuration.board.height
    && left.configuration.controllers.length === right.configuration.controllers.length
    && left.configuration.controllers.every((controller, index) =>
      controller === right.configuration.controllers[index])
}

export function controlIsObsolete(control: ControlCommand, room: RoomView | null): boolean {
  return Boolean(room && room.roomId === control.roomId && room.revision > control.revision
    && (room.roundId > control.roundId || room.phase.type !== 'running'))
}

export function ownSeatConnected(state: GameClientState): boolean {
  return state.room?.seats.find((seat) => seat.seat === state.youSeat)?.connection.type === 'connected'
}

export function canControl(state: GameClientState): boolean {
  if (state.connection !== 'authenticated' || state.admissionPending
    || state.leavePending || state.room?.phase.type !== 'running') return false
  const actor = state.snapshot?.state.actors.find((candidate) => candidate.seat === state.youSeat)
  return Boolean(ownSeatConnected(state) && state.youSeat && actor?.type === 'visible' && actor.alive)
}

export function isCurrentConnection(
  generation: number, socket: WebSocket, currentGeneration: number, currentSocket: WebSocket | null,
): boolean {
  return generation === currentGeneration && socket === currentSocket
}
