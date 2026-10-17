import { beforeEach, describe, expect, it, vi } from 'vitest'
import { GameClient, gameplayUrl, type AdmissionIntent, type GameClientState } from './client'
import type { GameServerMessage, RoomView, Snapshot } from './schema'

const token = 'a'.repeat(64)
const roomId = 'b'.repeat(32)

class GameSocketStub {
  static readonly OPEN = 1
  readonly listeners = new Map<string, Array<(event: Event | MessageEvent) => void>>()
  readonly sent: string[] = []
  readonly close = vi.fn()
  readonly url: string
  readyState = 0

  constructor(url: string) { this.url = url }

  addEventListener(name: string, listener: (event: Event | MessageEvent) => void): void {
    this.listeners.set(name, [...(this.listeners.get(name) ?? []), listener])
  }

  send(value: string): void { this.sent.push(value) }

  emit(name: string, data?: unknown): void {
    if (name === 'open') this.readyState = GameSocketStub.OPEN
    for (const listener of this.listeners.get(name) ?? []) listener({ data } as MessageEvent)
  }

  message(message: GameServerMessage | object): void {
    this.emit('message', JSON.stringify(message))
  }
}

interface Harness {
  client: GameClient
  fetcher: ReturnType<typeof vi.fn>
  sockets: GameSocketStub[]
  states: GameClientState[]
}

function harness(payload: unknown = { token }): Harness {
  const sockets: GameSocketStub[] = []
  const states: GameClientState[] = []
  const fetcher = vi.fn().mockResolvedValue({ ok: true, json: async () => payload })
  const client = new GameClient(
    (state) => states.push(state),
    {
      fetch: fetcher as typeof fetch,
      socket: (url) => {
        const socket = new GameSocketStub(url)
        sockets.push(socket)
        return socket as unknown as WebSocket
      },
      location: { protocol: 'https:', host: 'play.test' } as Location,
    },
  )
  return { client, fetcher, sockets, states }
}

async function authenticate(value: Harness, intent: AdmissionIntent = { type: 'create' }): Promise<GameSocketStub> {
  expect(value.client.admit(intent)).toBe(true)
  await vi.waitFor(() => expect(value.sockets).toHaveLength(1))
  const socket = value.sockets[0]
  socket.emit('open')
  socket.message({ type: 'authenticated', protocolVersion: 1 })
  return socket
}

function room(revision: number, phase: RoomView['phase'] = { type: 'waiting' }, roundId = 1): RoomView {
  return {
    roomId,
    roomCode: 'ABCDEFGH',
    roundId,
    revision,
    tick: revision,
    rulesVersion: 1,
    mapId: 'fixed_v1',
    seats: [
      { seat: 'one', occupied: true, ready: false },
      { seat: 'two', occupied: true, ready: false },
    ],
    phase,
  }
}

function snapshot(revision: number, roundId = 1, alive = true): Snapshot {
  const currentRoom = room(revision, { type: 'running' }, roundId)
  return {
    type: 'snapshot',
    room: currentRoom,
    state: {
      runningUpdates: 1,
      remainingTicks: 10_799,
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
      bombs: [],
      flames: [],
    },
    events: [],
  }
}

function messages(socket: GameSocketStub): object[] {
  return socket.sent.map((message) => JSON.parse(message) as object)
}

describe('GameClient session and connection lifecycle', () => {
  beforeEach(() => {
    vi.stubGlobal('WebSocket', { OPEN: GameSocketStub.OPEN })
    localStorage.clear()
  })

  it('keeps the credential in memory, authenticates first, then sends one admission', async () => {
    const value = harness()
    const socket = await authenticate(value, { type: 'join', roomCode: 'ABCDEFGH' })

    expect(value.fetcher).toHaveBeenCalledWith('/api/v1/game/session', expect.objectContaining({
      method: 'POST',
      signal: expect.any(AbortSignal),
    }))
    expect(socket.url).toBe('wss://play.test/api/v1/game/ws')
    expect(socket.url).not.toContain(token)
    expect(messages(socket)).toEqual([
      { type: 'authenticate', protocolVersion: 1, token },
      { type: 'joinRoom', sequence: 1, roomCode: 'ABCDEFGH' },
    ])
    expect(localStorage).toHaveLength(0)
    expect(value.client.admit({ type: 'create' })).toBe(false)
  })

  it('rejects malformed or open session payloads before creating a socket', async () => {
    for (const payload of [{ token, extra: true }, { token: 'A'.repeat(64) }, null]) {
      const value = harness(payload)
      value.client.admit({ type: 'create' })
      await vi.waitFor(() => expect(value.client.getState().failure).toBe('sessionFailed'))
      expect(value.sockets).toHaveLength(0)
    }

    const retry = harness({ token: 'short' })
    retry.client.admit({ type: 'create' })
    await vi.waitFor(() => expect(retry.client.getState().failure).toBe('sessionFailed'))
    retry.fetcher.mockResolvedValueOnce({ ok: true, json: async () => ({ token }) })
    expect(retry.client.admit({ type: 'create' })).toBe(true)
    await vi.waitFor(() => expect(retry.sockets).toHaveLength(1))
  })

  it('aborts cleanup and ignores stale fetch and socket callbacks', async () => {
    let resolveFirst!: (response: Response) => void
    const first = new Promise<Response>((resolve) => { resolveFirst = resolve })
    const value = harness()
    value.fetcher.mockReset().mockReturnValueOnce(first).mockResolvedValueOnce({ ok: true, json: async () => ({ token }) })

    value.client.admit({ type: 'create' })
    const firstSignal = value.fetcher.mock.calls[0][1].signal as AbortSignal
    value.client.disconnect()
    expect(firstSignal.aborted).toBe(true)
    value.client.admit({ type: 'create' })
    await vi.waitFor(() => expect(value.sockets).toHaveLength(1))
    resolveFirst({ ok: true, json: async () => ({ token }) } as Response)
    await Promise.resolve()
    expect(value.sockets).toHaveLength(1)

    const stale = value.sockets[0]
    stale.emit('open')
    value.client.disconnect()
    expect(stale.close).toHaveBeenCalled()
    stale.message({ type: 'authenticated', protocolVersion: 1 })
    stale.emit('close')
    expect(value.client.getState().connection).toBe('disconnected')
  })

  it('ignores a session body that resolves after its generation is replaced', async () => {
    let resolveBody!: (body: unknown) => void
    const body = new Promise<unknown>((resolve) => { resolveBody = resolve })
    const value = harness()
    value.fetcher.mockReset()
      .mockResolvedValueOnce({ ok: true, json: () => body })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ token }) })

    value.client.admit({ type: 'create' })
    await vi.waitFor(() => expect(value.fetcher).toHaveBeenCalledTimes(1))
    value.client.disconnect()
    value.client.admit({ type: 'join', roomCode: 'ABCDEFGH' })
    await vi.waitFor(() => expect(value.sockets).toHaveLength(1))
    resolveBody({ token: 'invalid-stale-body' })
    await Promise.resolve()

    expect(value.client.getState().failure).toBeNull()
    expect(value.client.getState().connection).toBe('connecting')
    expect(value.sockets).toHaveLength(1)
  })

  it('accepts authentication once and ignores room state before admission is sent', async () => {
    const value = harness()
    value.client.admit({ type: 'create' })
    await vi.waitFor(() => expect(value.sockets).toHaveLength(1))
    const socket = value.sockets[0]
    socket.emit('open')

    socket.message({ type: 'roomState', room: room(1), youSeat: 'one' })
    expect(value.client.getState().room).toBeNull()
    socket.message({ type: 'authenticated', protocolVersion: 1 })
    socket.message({ type: 'authenticated', protocolVersion: 1 })
    expect(messages(socket)).toEqual([
      { type: 'authenticate', protocolVersion: 1, token },
      { type: 'createRoom', sequence: 1 },
    ])

    socket.message({ type: 'roomState', room: room(1), youSeat: 'one' })
    expect(value.client.getState().room?.roomId).toBe(roomId)
  })

  it('rejects invalid server frames and preserves typed failures across close', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message({ type: 'error', code: 'invalidPhase', sequence: 1 })
    expect(value.client.getState().failure).toBe('invalidPhase')
    expect(value.client.getState().connection).toBe('authenticated')
    socket.emit('close')
    expect(value.client.getState().failure).toBe('invalidPhase')

    const invalid = harness()
    const invalidSocket = await authenticate(invalid)
    invalidSocket.message({ type: 'snapshot' })
    expect(invalidSocket.close).toHaveBeenCalled()
    invalidSocket.emit('close')
    expect(invalid.client.getState().failure).toBe('invalidServerMessage')

    const malformed = harness()
    const malformedSocket = await authenticate(malformed)
    malformedSocket.emit('message', '{')
    expect(malformed.client.getState().failure).toBe('invalidServerMessage')

    const incompatible = harness()
    const incompatibleSocket = await authenticate(incompatible)
    incompatibleSocket.message({ type: 'incompatibleProtocol', supportedVersion: 1 })
    expect(incompatible.client.getState().failure).toBe('incompatibleProtocol')
    expect(incompatibleSocket.close).toHaveBeenCalled()
  })
})

describe('GameClient authoritative room state and commands', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: GameSocketStub.OPEN }))

  it('requires admission and increasing revisions before accepting public state', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(snapshot(1))
    expect(value.client.getState().room).toBeNull()

    socket.message({ type: 'roomState', room: room(1), youSeat: 'one' })
    expect(value.client.getState().room?.revision).toBe(1)
    expect(value.client.getState().youSeat).toBe('one')

    socket.message({ type: 'roomState', room: room(1, { type: 'countdown', endsAt: 180 }), youSeat: 'two' })
    socket.message({ type: 'roomState', room: { ...room(2), roomId: 'c'.repeat(32) }, youSeat: 'two' })
    socket.message(snapshot(2, 2))
    expect(value.client.getState().room?.phase.type).toBe('waiting')
    expect(value.client.getState().youSeat).toBe('one')

    socket.message(snapshot(2))
    expect(value.client.getState().snapshot?.room.revision).toBe(2)
    socket.message(snapshot(2))
    expect(value.client.getState().snapshot?.room.revision).toBe(2)
  })

  it('sends phase-safe commands with fresh room data and independent action IDs', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message({ type: 'roomState', room: room(1), youSeat: 'one' })
    expect(value.client.ready(true)).toBe(true)
    expect(value.client.movement('east')).toBe(false)
    expect(value.client.placeBomb()).toBe(false)

    socket.message({ type: 'roomState', room: room(2, { type: 'running' }), youSeat: 'one' })
    socket.message(snapshot(3))
    expect(value.client.movement('east')).toBe(true)
    expect(value.client.movement(null)).toBe(true)
    expect(value.client.placeBomb()).toBe(true)
    expect(value.client.placeBomb()).toBe(true)
    expect(messages(socket).slice(1)).toEqual([
      { type: 'createRoom', sequence: 1 },
      { type: 'ready', ready: true, sequence: 2, roomId, roundId: 1 },
      { type: 'movement', direction: 'east', sequence: 3, roomId, roundId: 1 },
      { type: 'movement', direction: null, sequence: 4, roomId, roundId: 1 },
      { type: 'placeBomb', actionId: 1, sequence: 5, roomId, roundId: 1 },
      { type: 'placeBomb', actionId: 2, sequence: 6, roomId, roundId: 1 },
    ])

    socket.message({ type: 'roomState', room: room(4, { type: 'running' }, 2), youSeat: 'one' })
    socket.message(snapshot(5, 2))
    expect(value.client.placeBomb()).toBe(true)
    expect(messages(socket).at(-1)).toEqual({ type: 'placeBomb', actionId: 1, sequence: 7, roomId, roundId: 2 })
  })

  it('blocks dead-player input, correlates leave acknowledgements, and checks counter overflow', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message({ type: 'roomState', room: room(1, { type: 'running' }), youSeat: 'one' })
    socket.message(snapshot(2, 1, false))
    expect(value.client.movement('north')).toBe(false)
    expect(value.client.placeBomb()).toBe(false)

    expect(value.client.leave()).toBe(true)
    const leave = messages(socket).at(-1) as { sequence: number }
    socket.message({ type: 'error', code: 'invalidPhase', sequence: leave.sequence })
    expect(value.client.getState().leavePending).toBe(false)
    expect(value.client.leave()).toBe(true)
    const retriedLeave = messages(socket).at(-1) as { sequence: number }
    socket.message({ type: 'leftRoom', roomId, sequence: leave.sequence + 100 })
    expect(value.client.getState().room).not.toBeNull()
    socket.message({ type: 'leftRoom', roomId, sequence: retriedLeave.sequence })
    expect(value.client.getState().room).toBeNull()
    expect(value.client.getState().connection).toBe('authenticated')

    Reflect.set(value.client, 'nextSequence', 2_147_483_648)
    expect(value.client.admit({ type: 'create' })).toBe(false)
    expect(value.client.getState().failure).toBe('sequenceExhausted')
  })

  it('stops room commands as soon as leave is pending', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message({ type: 'roomState', room: room(1), youSeat: 'one' })
    expect(value.client.leave()).toBe(true)
    const afterLeave = socket.sent.length
    expect(value.client.ready(true)).toBe(false)

    socket.message({ type: 'roomState', room: room(2, { type: 'running' }), youSeat: 'one' })
    socket.message(snapshot(3))
    expect(value.client.movement('north')).toBe(false)
    expect(value.client.placeBomb()).toBe(false)
    expect(socket.sent).toHaveLength(afterLeave)
    expect(value.client.getState().failure).toBeNull()
  })
})

it('derives secure and local gameplay URLs', () => {
  expect(gameplayUrl({ protocol: 'https:', host: 'game.test' } as Location)).toBe('wss://game.test/api/v1/game/ws')
  expect(gameplayUrl({ protocol: 'http:', host: 'localhost:5177' } as Location)).toBe('ws://localhost:5177/api/v1/game/ws')
})
