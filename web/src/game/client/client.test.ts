import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  admitted, authenticate, harness, messages, resumeToken, room, roomId, sessionToken,
  snapshot, SocketStub,
} from './testSupport'
import { gameplayUrl } from './client'

describe('v12 admission and authoritative baselines', () => {
  beforeEach(() => {
    vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN })
    localStorage.clear()
    sessionStorage.clear()
  })

  it('uses the existing gameplay endpoint without placing credentials in its URL', () => {
    expect(gameplayUrl({ protocol: 'https:', host: 'play.test' } as Location))
      .toBe('wss://play.test/api/v1/game/ws')
    expect(gameplayUrl({ protocol: 'http:', host: 'localhost:5177' } as Location))
      .toBe('ws://localhost:5177/api/v1/game/ws')
  })

  it('authenticates with v12 and keeps the seat capability out of public state and storage', async () => {
    const value = harness()
    const socket = await authenticate(value, { type: 'join', roomCode: 'ABCDEFGH' })
    expect(messages(socket)).toEqual([
      { type: 'authenticate', protocolVersion: 12, token: sessionToken },
      { type: 'joinRoom', sequence: 1, roomCode: 'ABCDEFGH' },
    ])
    socket.message(admitted())
    expect(value.client.getState().youSeat).toBe('one')
    expect(JSON.stringify(value.client.getState())).not.toContain(resumeToken)
    expect(socket.sent.join()).not.toContain(resumeToken)
    expect(localStorage).toHaveLength(0)
    expect(sessionStorage).toHaveLength(0)
  })

  it('installs an admitted full baseline with no replayed events and continues bomb IDs', async () => {
    const value = harness()
    const socket = await authenticate(value)
    const baseline = admitted(4, 41)
    baseline.state!.bombs.push({
      id: 9, cell: { column: 1, row: 1 }, owner: 1, range: 2, placedAt: 1,
      trigger: { type: 'timed', remainingTicks: 180, paused: false }, blast: 'standard',
    })
    socket.message(baseline)
    expect(value.client.getState().snapshot).toEqual({
      type: 'snapshot', room: baseline.room, state: baseline.state, events: [],
    })
    expect(value.client.placeBomb()).toBe(true)
    expect(messages(socket).at(-1)).toMatchObject({ type: 'manualAction', action: 'bomb', actionId: 42 })
    const punchEnabled = snapshot(5)
    punchEnabled.room.settings.enabledItems = ['punch']
    socket.message(punchEnabled)
    expect(value.client.useEquipment()).toBe(true)
    expect(messages(socket).at(-1)).toMatchObject({
      type: 'manualAction', action: 'equipment', actionId: 43,
    })
  })

  it('guards Equipment by Punch settings without consuming the shared manual-action ID', async () => {
    const value = harness()
    const socket = await authenticate(value)
    const baseline = admitted(1, 8)
    socket.message(baseline)
    expect(value.client.useEquipment()).toBe(false)
    expect(messages(socket).some((message) => message.action === 'equipment')).toBe(false)

    const passive = snapshot(2)
    passive.room.settings.enabledItems = ['bracelet']
    socket.message(passive)
    expect(value.client.useEquipment()).toBe(false)
    expect(messages(socket).some((message) => message.action === 'equipment')).toBe(false)

    const enabled = snapshot(3)
    enabled.room.settings.enabledItems = ['punch']
    socket.message(enabled)
    expect(value.client.useEquipment()).toBe(true)
    expect(messages(socket).at(-1)).toMatchObject({ action: 'equipment', actionId: 9 })
    expect(value.client.placeBomb()).toBe(true)
    expect(messages(socket).at(-1)).toMatchObject({ action: 'bomb', actionId: 10 })
  })

  it('keeps MAX manual action exhaustion and never resends an action after reconnect', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted(1, 2_147_483_647))
    expect(value.client.placeBomb()).toBe(false)
    expect(value.client.getState().failure).toBe('sequenceExhausted')

    socket.emit('close')
    value.scheduled.shift()!()
    await vi.waitFor(() => expect(value.sockets).toHaveLength(2))
    const resumed = value.sockets[1]
    resumed.emit('open')
    resumed.message({ type: 'authenticated', protocolVersion: 12 })
    expect(messages(resumed)).toEqual([
      { type: 'authenticate', protocolVersion: 12, token: sessionToken },
      { type: 'resumeRoom', sequence: 1, roomId, resumeToken },
    ])
    expect(messages(resumed).some((message) => message.type === 'manualAction')).toBe(false)
  })

  it('rejects shared updates until the personal resume baseline and rejects stale admission facts', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted(5))
    const originalBaseline = value.client.getState().admissionBaseline
    const retained = value.client.getState().snapshot
    socket.emit('close')
    value.scheduled.shift()!()
    await vi.waitFor(() => expect(value.sockets).toHaveLength(2))
    const resumed = value.sockets[1]
    resumed.emit('open')
    resumed.message({ type: 'authenticated', protocolVersion: 12 })
    resumed.message({ type: 'roomState', room: room(6, { type: 'running' }) })
    resumed.message(snapshot(7))
    expect(value.client.getState().snapshot).toBe(retained)
    expect(value.client.movement('east')).toBe(false)

    resumed.message({ ...admitted(4), room: { ...admitted(4).room, revision: 4 } })
    expect(value.client.getState().admissionPending).toBe(true)
    resumed.message({ ...admitted(8), youSeat: 'two' })
    resumed.message({ ...admitted(8), resumeToken: 'd'.repeat(64) })
    expect(value.client.getState().admissionPending).toBe(true)
    resumed.message({ ...admitted(8), sequence: 99 })
    expect(value.client.getState().admissionPending).toBe(true)
    resumed.message(admitted(8))
    resumed.message(snapshot(9))
    expect(value.client.getState().room?.revision).toBe(9)
    expect(value.client.getState().admissionPending).toBe(false)
    expect(value.client.getState().snapshot?.room.revision).toBe(9)
    expect(value.client.getState().admissionBaseline).toEqual({
      roomId, revision: 8, tick: 8, physicalTick: 1,
    })
    expect(value.client.getState().admissionBaseline).not.toBe(originalBaseline)
  })
})

describe('v12 recovery lifecycle', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN }))

  it('uses a fresh single-use HTTP session on every bounded reconnect attempt', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    socket.emit('close')
    expect(value.client.getState()).toMatchObject({ connection: 'reconnecting', reconnectAttempt: 1 })
    expect(value.client.getState().room?.roomId).toBe(roomId)

    for (let attempt = 1; attempt <= 3; attempt += 1) {
      value.scheduled.shift()!()
      await vi.waitFor(() => expect(value.sockets).toHaveLength(attempt + 1))
      value.sockets[attempt].emit('close')
    }
    expect(value.fetcher).toHaveBeenCalledTimes(4)
    expect(value.client.getState()).toMatchObject({ connection: 'disconnected', failure: 'connectionFailed' })
    expect(value.scheduled).toHaveLength(0)
  })

  it('fences stale fetch/socket callbacks and clears capability on explicit disposal', async () => {
    let resolve!: (response: Response) => void
    const pending = new Promise<Response>((done) => { resolve = done })
    const value = harness()
    value.fetcher.mockReset().mockReturnValueOnce(pending)
    value.client.admit({ type: 'create' })
    value.client.dispose()
    resolve({ ok: true, json: async () => ({ token: sessionToken }) } as Response)
    await Promise.resolve()
    expect(value.sockets).toHaveLength(0)
    expect(value.client.getState().room).toBeNull()
  })

  it('does not resume after an explicit leave even if transport closes before acknowledgement', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    expect(value.client.leave()).toBe(true)
    socket.emit('close')
    expect(value.scheduled).toHaveLength(0)
    expect(value.client.getState()).toMatchObject({
      connection: 'disconnected', room: null, snapshot: null, youSeat: null,
      admissionPending: false, leavePending: false,
    })
    expect(value.client.admit({ type: 'create' })).toBe(true)
  })

  it.each([
    ['seatReplaced frame', (socket: SocketStub) => socket.message({ type: 'seatReplaced', roomId })],
    ['replacement close', (socket: SocketStub) => socket.emit('close', 4001)],
    ['resume rejection', (socket: SocketStub) => socket.message({ type: 'error', code: 'resumeRejected', sequence: 1 })],
  ])('stops retry after %s', async (_label, stop) => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    socket.emit('close')
    value.scheduled.shift()!()
    await vi.waitFor(() => expect(value.sockets).toHaveLength(2))
    const resumed = value.sockets[1]
    resumed.emit('open')
    resumed.message({ type: 'authenticated', protocolVersion: 12 })
    stop(resumed)
    expect(value.client.getState()).toMatchObject({
      connection: 'disconnected', room: null, snapshot: null, youSeat: null,
      admissionPending: false,
    })
    expect(value.scheduled).toHaveLength(0)
    expect(value.client.admit({ type: 'create' })).toBe(true)
  })
})
