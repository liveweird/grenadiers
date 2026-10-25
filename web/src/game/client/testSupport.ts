import { vi } from 'vitest'
import { GameClient, type AdmissionIntent, type GameClientState } from './client'
import type { Admitted, GameServerMessage, GameState, RoomPhase, RoomView, Snapshot } from '../v11/schema'

export const sessionToken = 'a'.repeat(64)
export const resumeToken = 'c'.repeat(64)
export const roomId = 'b'.repeat(32)

export class SocketStub {
  static readonly OPEN = 1
  readonly listeners = new Map<string, Array<(event: Event | MessageEvent | CloseEvent) => void>>()
  readonly sent: string[] = []
  readonly close = vi.fn()
  readonly url: string
  readyState = 0

  constructor(url: string) { this.url = url }

  addEventListener(name: string, listener: (event: Event | MessageEvent | CloseEvent) => void): void {
    this.listeners.set(name, [...(this.listeners.get(name) ?? []), listener])
  }

  send(value: string): void { this.sent.push(value) }

  emit(name: string, data?: unknown): void {
    if (name === 'open') this.readyState = SocketStub.OPEN
    const event = name === 'close' ? { code: data ?? 1006 } : { data }
    for (const listener of this.listeners.get(name) ?? []) listener(event as MessageEvent & CloseEvent)
  }

  message(message: GameServerMessage | object): void { this.emit('message', JSON.stringify(message)) }
}

export interface Harness {
  client: GameClient
  fetcher: ReturnType<typeof vi.fn>
  sockets: SocketStub[]
  states: GameClientState[]
  scheduled: Array<() => void>
  clock: FakeClock
}

interface ClockTask {
  due: number
  canceled: boolean
  completed: boolean
  callback: () => void
  run: () => void
}

export class FakeClock {
  now = 0
  readonly scheduled: Array<() => void> = []
  private readonly tasks = new Map<ReturnType<typeof setTimeout>, ClockTask>()

  schedule = (callback: () => void, delay: number): ReturnType<typeof setTimeout> => {
    const handle = {} as ReturnType<typeof setTimeout>
    const task: ClockTask = {
      due: this.now + delay, canceled: false, completed: false, callback, run: () => undefined,
    }
    task.run = () => {
      if (task.canceled || task.completed) return
      task.completed = true
      this.remove(task.run)
      this.tasks.delete(handle)
      this.now = Math.max(this.now, task.due)
      callback()
    }
    this.tasks.set(handle, task)
    const index = this.scheduled.findIndex((run) => this.taskFor(run)!.due > task.due)
    this.scheduled.splice(index < 0 ? this.scheduled.length : index, 0, task.run)
    return handle
  }

  cancel = (handle: ReturnType<typeof setTimeout>): void => {
    const task = this.tasks.get(handle)
    if (!task) return
    task.canceled = true
    this.remove(task.run)
    this.tasks.delete(handle)
  }

  advanceBy(milliseconds: number): void {
    const target = this.now + milliseconds
    while (this.scheduled.length > 0 && this.taskFor(this.scheduled[0])!.due <= target) {
      this.scheduled[0]()
    }
    this.now = target
  }

  rawCallback(run: () => void): () => void {
    const task = this.taskFor(run)
    if (!task) throw new Error('scheduled callback not found')
    return task.callback
  }

  private remove(run: () => void): void {
    const index = this.scheduled.indexOf(run)
    if (index >= 0) this.scheduled.splice(index, 1)
  }

  private taskFor(run: () => void): ClockTask | undefined {
    return [...this.tasks.values()].find((task) => task.run === run)
  }
}

export function harness(payload: unknown = { token: sessionToken }): Harness {
  const sockets: SocketStub[] = []
  const states: GameClientState[] = []
  const clock = new FakeClock()
  const fetcher = vi.fn().mockResolvedValue({ ok: true, json: async () => payload })
  const client = new GameClient((state) => states.push(state), {
    fetch: fetcher as typeof fetch,
    socket: (url) => {
      const socket = new SocketStub(url)
      sockets.push(socket)
      return socket as unknown as WebSocket
    },
    location: { protocol: 'https:', host: 'play.test' } as Location,
    schedule: clock.schedule,
    cancel: clock.cancel,
    now: () => clock.now,
  })
  return { client, fetcher, sockets, states, scheduled: clock.scheduled, clock }
}

export async function authenticate(value: Harness, intent: AdmissionIntent = { type: 'create' }): Promise<SocketStub> {
  value.client.admit(intent)
  await vi.waitFor(() => expect(value.sockets).toHaveLength(1))
  const socket = value.sockets.at(-1)!
  socket.emit('open')
  socket.message({ type: 'authenticated', protocolVersion: 11 })
  return socket
}

export function room(revision: number, phase: RoomPhase = { type: 'waiting' }, roundId = 1): RoomView {
  return {
    roomId, roomCode: 'ABCDEFGH', roundId, revision, tick: revision,
    rulesVersion: 5, mapId: 'generated_v2', roundNumber: phase.type === 'running' ? 1 : 0,
    scores: { one: 0, two: 0 }, lastRound: null,
    settings: {
      startingSpeedLevel: 0, turnAssistUnits: 256, winsToFinish: 3, bombFuseTicks: 180,
      enabledItems: ['heart'],
    },
    availableItems: ['kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'oilBarrel', 'teleporter'],
    settingsRevision: 1, settingsEditor: 'one',
    seats: [
      { seat: 'one', occupied: true, ready: false, rematchReady: false, connection: { type: 'connected' } },
      { seat: 'two', occupied: true, ready: false, rematchReady: false, connection: { type: 'connected' } },
    ],
    phase,
  }
}

function gameState(alive = true): GameState {
  return {
    runningUpdates: 1, remainingTicks: 10_799, physicalTick: 1,
    arena: { width: 15 as const, height: 13 as const, terrain: Array.from({ length: 195 }, () => 'floor' as const), blocks: [] },
    actors: [
      {
        type: 'visible',
        id: 1, seat: 'one' as const, position: { x: 512, y: 512 }, heading: 'east' as const,
        speedLevel: 0, alive, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
        inventory: { equipment: null, weapon: null }, effects: [], damageGraceRemainingTicks: 0,
        voluntaryActionsFrozen: false, lastTeleportTick: null,
      },
      {
        type: 'visible',
        id: 2, seat: 'two' as const, position: { x: 14_848, y: 12_800 }, heading: 'west' as const,
        speedLevel: 0, alive: true, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
        inventory: { equipment: null, weapon: null }, effects: [], damageGraceRemainingTicks: 0,
        voluntaryActionsFrozen: false, lastTeleportTick: null,
      },
    ],
    bombs: [], flames: [], pickups: [], oilBarrels: [],
  }
}

export function admitted(revision = 1, lastActionId: number | null = null): Admitted {
  return {
    type: 'admitted' as const, sequence: 1, room: room(revision, { type: 'running' }),
    youSeat: 'one' as const, resumeToken, lastActionId, state: gameState(),
  }
}

export function snapshot(revision: number, roundId = 1): Snapshot {
  return { type: 'snapshot', room: room(revision, { type: 'running' }, roundId), state: gameState(), events: [] }
}

export function messages(socket: SocketStub): Array<Record<string, unknown>> {
  return socket.sent.map((message) => JSON.parse(message) as Record<string, unknown>)
}
