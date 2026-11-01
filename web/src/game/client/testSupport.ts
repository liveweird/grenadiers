import { vi } from 'vitest'
import { GameClient, type AdmissionIntent, type GameClientState } from './client'
import type { Admitted, GameServerMessage, GameState, RoomPhase, RoomView, Snapshot } from '../v19/schema'

export const sessionToken = 'a'.repeat(64)
export const resumeToken = 'c'.repeat(64)
export const roomId = 'b'.repeat(32)

export class SocketStub {
  static readonly OPEN = 1
  readonly listeners: { [K in keyof WebSocketEventMap]: Array<(event: WebSocketEventMap[K]) => void> } = {
    close: [], error: [], message: [], open: [],
  }
  readonly sent: string[] = []
  readonly close = vi.fn()
  readonly url: string
  readyState = 0

  constructor(url: string) { this.url = url }

  addEventListener<K extends keyof WebSocketEventMap>(
    name: K, listener: (event: WebSocketEventMap[K]) => void,
  ): void {
    this.listeners[name].push(listener)
  }

  send(value: string): void { this.sent.push(value) }

  emit(name: 'open' | 'error'): void
  emit(name: 'message', data: unknown): void
  emit(name: 'close', code?: number): void
  emit(name: keyof WebSocketEventMap, data?: unknown): void {
    switch (name) {
      case 'open':
        this.readyState = SocketStub.OPEN
        for (const listener of this.listeners.open) listener(new Event(name))
        break
      case 'error':
        for (const listener of this.listeners.error) listener(new Event(name))
        break
      case 'message':
        for (const listener of this.listeners.message) listener(new MessageEvent(name, { data }))
        break
      case 'close':
        for (const listener of this.listeners.close) {
          listener(new CloseEvent(name, { code: typeof data === 'number' ? data : 1006 }))
        }
        break
    }
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
  socket.message({ type: 'authenticated', protocolVersion: 19 })
  return socket
}

export function room(revision: number, phase: RoomPhase = { type: 'waiting' }, roundId = 1): RoomView {
  const completed = phase.type === 'result'
    ? { roundId, roundNumber: 1, endedAt: phase.holdUntil - 180,
      result: { type: 'draw' as const, reason: 'mutualElimination' as const } }
    : phase.type === 'matchResult' && phase.result.type === 'winner'
      ? { roundId, roundNumber: 1, endedAt: phase.endedAt,
        result: { type: 'winner' as const, seat: phase.result.seat } }
      : null
  const roundNumber = ['running', 'result', 'matchResult'].includes(phase.type) ? 1 : 0
  const terminalWinner = phase.type === 'matchResult' && phase.result.type === 'winner'
    ? phase.result.seat : null
  return {
    roomId, roomCode: 'ABCDEFGH', roundId, revision, tick: revision,
    configuration: { controllers: ['human', 'human'], cpuDifficulty: null, board: { width: 15, height: 13 },
      environment: { stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 },
        spikePits: { enabled: true, warningTicks: 36, raisedTicks: 180 } } },
    rulesVersion: 10, mapId: 'generated_v2', roundNumber,
    scores: [{ seat: 'one', wins: terminalWinner === 'one' ? 1 : 0 },
      { seat: 'two', wins: terminalWinner === 'two' ? 1 : 0 }], lastRound: completed,
    settings: {
      startingSpeedLevel: 0, turnAssistUnits: 256,
      winsToFinish: terminalWinner ? 1 : 3, bombFuseTicks: 180,
      enabledItems: ['kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb',
        'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak',
        'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse',
        'reverseControls', 'invisibleBombs', 'oilBarrel', 'teleporter'],
      environment: { stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 },
        spikePits: { enabled: true, warningTicks: 36, raisedTicks: 180 } },
    },
    availableItems: ['kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'oilBarrel', 'teleporter'],
    settingsRevision: 1, settingsEditor: 'one',
    seats: [
      { seat: 'one', controller: { type: 'human' }, occupied: true, ready: false, rematchReady: false, connection: { type: 'connected' } },
      { seat: 'two', controller: { type: 'human' }, occupied: true, ready: false, rematchReady: false, connection: { type: 'connected' } },
    ],
    phase,
  }
}

function gameState(physicalTick: number, alive = true): GameState {
  return {
    runningUpdates: physicalTick, remainingTicks: 10_800 - physicalTick, physicalTick,
    arena: { width: 15 as const, height: 13 as const,
      terrain: Array.from({ length: 195 }, (_, index) => index === 96 || index === 98 ? 'wall' as const : 'floor' as const),
      blocks: [] },
    actors: [
      {
        type: 'visible',
        id: 1, seat: 'one' as const, position: { x: 512, y: 512 }, heading: 'east' as const,
        speedLevel: 0, alive, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
        inventory: { equipment: null, weapon: null }, effects: [], damageGraceRemainingTicks: 0,
        voluntaryActionsFrozen: false, lastTeleportTick: null, stormSlowRemainingTicks: 0,
      },
      {
        type: 'visible',
        id: 2, seat: 'two' as const, position: { x: 14_848, y: 12_800 }, heading: 'west' as const,
        speedLevel: 0, alive: true, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
        inventory: { equipment: null, weapon: null }, effects: [], damageGraceRemainingTicks: 0,
        voluntaryActionsFrozen: false, lastTeleportTick: null, stormSlowRemainingTicks: 0,
      },
    ],
    bombs: [], flames: [], pickups: [], oilBarrels: [],
    stormCloud: { position: { x: 7_680, y: 6_656 }, direction: 'east', legRemainingTicks: 128 },
    lairs: [
      { id: 0, cell: { column: 6, row: 6 }, spawnRemainingTicks: 600, livingMobs: 0 },
      { id: 1, cell: { column: 8, row: 6 }, spawnRemainingTicks: 600, livingMobs: 0 },
    ],
    spikePits: [], mobs: [],
  }
}

export function admitted(revision = 1, lastActionId: number | null = null): Admitted {
  return {
    type: 'admitted' as const, sequence: 1, room: room(revision, { type: 'running' }),
    youSeat: 'one' as const, resumeToken, lastActionId, state: gameState(revision),
  }
}

export function snapshot(revision: number, roundId = 1): Snapshot {
  return {
    type: 'snapshot', room: room(revision, { type: 'running' }, roundId),
    state: gameState(revision), events: [],
  }
}

export function messages(socket: SocketStub): Array<Record<string, unknown>> {
  return socket.sent.map((message) => JSON.parse(message) as Record<string, unknown>)
}
