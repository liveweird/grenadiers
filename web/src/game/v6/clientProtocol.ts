import Ajv from 'ajv'
import gameplaySchema from '../../../../api/gameplay-v6.schema.json'
import type { Direction, GameServerMessage, GameSettings, RoomView, Seat, V6ErrorCode } from './schema'

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
  | { type: 'placeBomb'; actionId: number }
  | { type: 'rematch'; ready: boolean }
export type ControlCommand = { sequence: number; roomId: string; roundId: number; revision: number }
export type PendingSettings = {
  roomId: string
  roundId: number
  settingsRevision: number
  settings: GameSettings
}
export type ControlFailure = ControlCommand & { code: V6ErrorCode }

const TOKEN_PATTERN = /^[0-9a-f]{64}$/
const ajv = new Ajv({ allErrors: true })
export const validateServerMessage = ajv.compile<GameServerMessage>(gameplaySchema)

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
