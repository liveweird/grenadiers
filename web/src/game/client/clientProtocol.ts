import type { Direction, GameServerMessage, GameSettings, RoomView, Seat, V11ErrorCode } from '../v11/schema'
import { validateGameplayV11ServerMessage } from '../v11/validation'

export const MAX_COUNTER = 2_147_483_647
export const MAX_RECONNECT_ATTEMPTS = 3

export interface Dependencies {
  fetch: typeof fetch
  socket: (url: string) => WebSocket
  location: Location
  schedule: (callback: () => void, delay: number) => ReturnType<typeof setTimeout>
  cancel: (timer: ReturnType<typeof setTimeout>) => void
}

export type Capability = { roomId: string; seat: Seat; token: string }
export type RoomCommand =
  | { type: 'ready'; ready: boolean; settingsRevision: number }
  | { type: 'movement'; direction: Direction | null }
  | { type: 'manualAction'; actionId: number; action: 'bomb' | 'equipment' }
  | { type: 'rematch'; ready: boolean }
export type ControlCommand = { sequence: number; roomId: string; roundId: number; revision: number }
export type PendingSettings = {
  roomId: string
  roundId: number
  settingsRevision: number
  settings: GameSettings
}
export type ControlFailure = ControlCommand & { code: V11ErrorCode }

const TOKEN_PATTERN = /^[0-9a-f]{64}$/
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
  const result = validateGameplayV11ServerMessage(value)
  return result.valid && roomRecipientValid(result.value) && stateRecipientValid(result.value, expectedSeat)
}

export function dependenciesWithDefaults(overrides?: Partial<Dependencies>): Dependencies {
  return {
    fetch: overrides?.fetch ?? globalThis.fetch.bind(globalThis),
    socket: overrides?.socket ?? ((url) => new WebSocket(url)),
    location: overrides?.location ?? window.location,
    schedule: overrides?.schedule ?? ((callback, delay) => setTimeout(callback, delay)),
    cancel: overrides?.cancel ?? ((timer) => clearTimeout(timer)),
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
    && next.roundId >= current.roundId)
}

export function newerRoom(next: RoomView, current: RoomView): boolean {
  return next.roomId === current.roomId && next.revision > current.revision && next.roundId >= current.roundId
}

export function controlIsObsolete(control: ControlCommand, room: RoomView | null): boolean {
  return Boolean(room && room.roomId === control.roomId && room.revision > control.revision
    && (room.roundId > control.roundId || room.phase.type !== 'running'))
}

export function isCurrentConnection(
  generation: number, socket: WebSocket, currentGeneration: number, currentSocket: WebSocket | null,
): boolean {
  return generation === currentGeneration && socket === currentSocket
}
