import { vi } from 'vitest'
import { GameClient, type AdmissionIntent, type GameClientState } from './client'
import type { GameServerMessage, RoomPhase, RoomView, Snapshot } from './schema'

export const token = 'a'.repeat(64)
export const roomId = 'b'.repeat(32)

export class SocketStub {
  static readonly OPEN = 1
  readonly listeners = new Map<string, Array<(event: Event | MessageEvent) => void>>()
  readonly sent: string[] = []
  readonly close = vi.fn()
  readyState = 0

  addEventListener(name: string, listener: (event: Event | MessageEvent) => void): void {
    this.listeners.set(name, [...(this.listeners.get(name) ?? []), listener])
  }

  send(value: string): void { this.sent.push(value) }

  emit(name: string, data?: unknown): void {
    if (name === 'open') this.readyState = SocketStub.OPEN
    for (const listener of this.listeners.get(name) ?? []) listener({ data } as MessageEvent)
  }

  message(message: GameServerMessage | object): void { this.emit('message', JSON.stringify(message)) }
}

export interface Harness {
  client: GameClient
  fetcher: ReturnType<typeof vi.fn>
  sockets: SocketStub[]
  states: GameClientState[]
}

export function harness(payload: unknown = { token }): Harness {
  const sockets: SocketStub[] = []
  const states: GameClientState[] = []
  const fetcher = vi.fn().mockResolvedValue({ ok: true, json: async () => payload })
  const client = new GameClient(
    (state) => states.push(state),
    {
      fetch: fetcher as typeof fetch,
      socket: () => {
        const socket = new SocketStub()
        sockets.push(socket)
        return socket as unknown as WebSocket
      },
      location: { protocol: 'https:', host: 'play.test' } as Location,
    },
  )
  return { client, fetcher, sockets, states }
}

export async function authenticate(value: Harness, intent: AdmissionIntent = { type: 'create' }): Promise<SocketStub> {
  value.client.admit(intent)
  await vi.waitFor(() => expect(value.sockets).toHaveLength(1))
  const socket = value.sockets[0]
  socket.emit('open')
  socket.message({ type: 'authenticated', protocolVersion: 3 })
  return socket
}

export function room(revision: number, phase: RoomPhase = { type: 'waiting' }, roundId = 1): RoomView {
  return {
    roomId, roomCode: 'ABCDEFGH', roundId, revision, tick: revision,
    rulesVersion: 1, mapId: 'fixed_v1', roundNumber: roundId,
    scores: { one: 0, two: 0 }, lastRound: null,
    seats: [
      { seat: 'one', occupied: true, ready: false, rematchReady: false },
      { seat: 'two', occupied: true, ready: false, rematchReady: false },
    ],
    phase,
  }
}

export function snapshot(revision: number, roundId = 1, alive = true): Snapshot {
  return {
    type: 'snapshot', room: room(revision, { type: 'running' }, roundId),
    state: {
      runningUpdates: 1, remainingTicks: 10_799,
      arena: { width: 15, height: 13, terrain: Array.from({ length: 195 }, () => 'floor'), blocks: [] },
      actors: [
        {
          id: 1, seat: 'one', position: { x: 512, y: 512 }, heading: 'east', speedLevel: 0,
          alive, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
        },
        {
          id: 2, seat: 'two', position: { x: 14_848, y: 12_800 }, heading: 'west', speedLevel: 0,
          alive: true, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
        },
      ],
      bombs: [], flames: [], pickups: [],
    },
    events: [],
  }
}

export function messages(socket: SocketStub): Array<Record<string, unknown>> {
  return socket.sent.map((message) => JSON.parse(message) as Record<string, unknown>)
}
