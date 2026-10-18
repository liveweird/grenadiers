import { expect, type Page } from '@playwright/test'
import { performance } from 'node:perf_hooks'
import type { Direction, GameServerMessage, V6ErrorCode } from '../../../web/src/game/v6/schema'

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
const CONTROL_LIMIT = 256
const SOCKET_LIMIT = 64
const CLIENT_TYPES = new Set([
  'authenticate', 'createRoom', 'joinRoom', 'resumeRoom', 'ready', 'leaveRoom', 'rematch',
  'updateSettings',
])
const COMMAND_TYPES = new Set([...CLIENT_TYPES, 'movement', 'placeBomb'])
const SERVER_TYPES = new Set([
  'authenticated', 'incompatibleProtocol', 'admitted', 'roomState', 'leftRoom', 'seatReplaced', 'error',
])
const ERROR_CODES = new Set<V6ErrorCode>([
  'invalidMessage', 'authenticationFailed', 'connectionInUse', 'notAuthenticated',
  'alreadyAuthenticated', 'alreadyInRoom', 'notInRoom', 'roomNotFound', 'roomFull', 'staleRoom',
  'invalidPhase', 'staleSequence', 'duplicateAction', 'rateLimited', 'capacityReached',
  'resumeRejected', 'staleSettings', 'settingsLocked',
])

export type GameControlEvent = {
  socket: number
  at: number
  direction: 'sent' | 'received' | 'transport'
  type: string
  sequence?: number | null
  errorCode?: V6ErrorCode
  admitted?: true
  seatReplaced?: true
}

type GameSocketControl = {
  socket: number
  createdAt: number
  closedAt?: number
  lastSnapshot?: { at: number; tick: number; revision: number }
}

export type GameControlDiagnostics = {
  timeOrigin: number
  events: GameControlEvent[]
  socketStates: GameSocketControl[]
}

export type ObservedGame = {
  frames: GameServerMessage[]
  commands: { type: string; actionId?: number; direction?: Direction | null }[]
  sockets: number
  control: GameControlDiagnostics
}

function objectFrame(payload: string | Buffer): Record<string, unknown> | undefined {
  try {
    const parsed: unknown = JSON.parse(payload.toString())
    return typeof parsed === 'object' && parsed !== null ? parsed as Record<string, unknown> : undefined
  } catch { return undefined }
}

function sequence(frame: Record<string, unknown>): number | null | undefined {
  if (typeof frame.sequence === 'number') return frame.sequence
  if (frame.sequence === null) return null
  return undefined
}

export class GameControlRecorder {
  readonly diagnostics: GameControlDiagnostics = {
    timeOrigin: performance.timeOrigin, events: [], socketStates: [],
  }
  private nextSocket = 1

  addSocket(): number {
    const socket = this.nextSocket
    this.nextSocket += 1
    this.diagnostics.socketStates.push({ socket, createdAt: performance.now() })
    if (this.diagnostics.socketStates.length > SOCKET_LIMIT) this.diagnostics.socketStates.shift()
    return socket
  }

  recordSent(socket: number, payload: string | Buffer): void {
    const frame = objectFrame(payload)
    if (!frame || typeof frame.type !== 'string' || !CLIENT_TYPES.has(frame.type)) return
    this.push({ socket, at: performance.now(), direction: 'sent', type: frame.type,
      sequence: sequence(frame) })
  }

  recordReceived(socket: number, payload: string | Buffer): void {
    const frame = objectFrame(payload)
    if (!frame || typeof frame.type !== 'string') return
    if (frame.type === 'snapshot') {
      const room = typeof frame.room === 'object' && frame.room !== null
        ? frame.room as Record<string, unknown> : undefined
      if (typeof room?.tick === 'number' && typeof room.revision === 'number') {
        const state = this.diagnostics.socketStates.find((value) => value.socket === socket)
        if (state) state.lastSnapshot = {
          at: performance.now(), tick: room.tick, revision: room.revision,
        }
      }
      return
    }
    if (!SERVER_TYPES.has(frame.type)) return
    const event: GameControlEvent = {
      socket, at: performance.now(), direction: 'received', type: frame.type,
      sequence: sequence(frame),
    }
    if (frame.type === 'admitted') event.admitted = true
    if (frame.type === 'seatReplaced') event.seatReplaced = true
    if (frame.type === 'error' && typeof frame.code === 'string'
      && ERROR_CODES.has(frame.code as V6ErrorCode)) event.errorCode = frame.code as V6ErrorCode
    this.push(event)
  }

  recordClose(socket: number): void {
    const at = performance.now()
    const state = this.diagnostics.socketStates.find((value) => value.socket === socket)
    if (state) state.closedAt = at
    this.push({ socket, at, direction: 'transport', type: 'close' })
  }

  private push(event: GameControlEvent): void {
    this.diagnostics.events.push(event)
    if (this.diagnostics.events.length > CONTROL_LIMIT) this.diagnostics.events.shift()
  }
}

export function observeGame(page: Page): ObservedGame {
  const recorder = new GameControlRecorder()
  const observed: ObservedGame = {
    frames: [], commands: [], sockets: 0, control: recorder.diagnostics,
  }
  page.on('websocket', (socket) => {
    if (!new URL(socket.url()).pathname.endsWith('/game/ws')) return
    const socketIndex = recorder.addSocket()
    observed.sockets += 1
    socket.on('framesent', ({ payload }) => {
      recorder.recordSent(socketIndex, payload)
      const command = objectFrame(payload)
      if (command && typeof command.type === 'string' && command.type !== 'authenticate'
        && COMMAND_TYPES.has(command.type)) {
        let direction: Direction | null | undefined
        if (['north', 'east', 'south', 'west'].includes(String(command.direction))) {
          direction = command.direction as Direction
        } else if (command.direction === null) {
          direction = null
        }
        const summary: ObservedGame['commands'][number] = {
          type: command.type,
          actionId: typeof command.actionId === 'number' ? command.actionId : undefined,
        }
        if (command.type === 'movement') summary.direction = direction
        observed.commands.push(summary)
      }
    })
    socket.on('framereceived', ({ payload }) => {
      recorder.recordReceived(socketIndex, payload)
      const frame = JSON.parse(payload.toString()) as GameServerMessage
      if (frame.type === 'admitted') frame.resumeToken = '[redacted]'
      observed.frames.push(frame)
      if (observed.frames.length > 512) observed.frames.shift()
    })
    socket.on('close', () => recorder.recordClose(socketIndex))
  })
  return observed
}

export function latestSnapshot(game: ObservedGame): Snapshot | undefined {
  return game.frames.filter((frame) => frame.type === 'snapshot').at(-1)
}

export async function expectPaintedCanvas(page: Page): Promise<void> {
  const canvas = page.locator('.game-arena canvas')
  await expect(canvas).toBeVisible()
  const screenshot = await canvas.screenshot()
  // Decode a compositor screenshot: reading the WebGL drawing buffer after presentation
  // can legitimately return cleared pixels when preserveDrawingBuffer is false.
  const colors = await page.evaluate(async (png) => {
    const image = new Image()
    image.src = `data:image/png;base64,${png}`
    await image.decode()
    const sample = document.createElement('canvas')
    sample.width = image.width
    sample.height = image.height
    const context = sample.getContext('2d')
    if (!context) throw new Error('Screenshot decoder unavailable')
    context.drawImage(image, 0, 0)
    const pixels = context.getImageData(0, 0, sample.width, sample.height).data
    const distinct = new Set<string>()
    for (let index = 0; index < pixels.length; index += 16) {
      distinct.add(`${pixels[index]},${pixels[index + 1]},${pixels[index + 2]}`)
    }
    return distinct.size
  }, screenshot.toString('base64'))
  expect(colors).toBeGreaterThan(10)
}
