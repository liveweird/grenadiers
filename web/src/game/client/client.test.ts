import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  admitted, authenticate, harness, messages, resumeToken, room, roomId, sessionToken,
  snapshot, SocketStub,
} from './testSupport'
import { gameplayUrl } from './client'

function multiplayerAdmission(seat: 'three' | 'four', revision: number) {
  const message = admitted(revision)
  message.youSeat = seat
  message.room.rulesVersion = 8
  message.room.configuration = {
    controllers: ['human', 'human', 'human', 'human'], cpuDifficulty: null,
    board: { width: 15, height: 13 },
    environment: { stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 3 } },
  }
  message.room.seats.push(
    { seat: 'three', controller: { type: 'human' }, occupied: true, ready: false,
      rematchReady: false, connection: { type: 'connected' } },
    { seat: 'four', controller: { type: 'human' }, occupied: true, ready: false,
      rematchReady: false, connection: { type: 'connected' } },
  )
  message.room.scores.push({ seat: 'three', wins: 0 }, { seat: 'four', wins: 0 })
  const one = message.state?.actors[0]
  const two = message.state?.actors[1]
  if (!message.state || one?.type !== 'visible' || two?.type !== 'visible') {
    throw new Error('admission fixture actors must be visible')
  }
  message.state.actors.push(
    { ...one, id: 3, seat: 'three', position: { x: 512, y: 12_800 } },
    { ...two, id: 4, seat: 'four', position: { x: 14_848, y: 512 } },
  )
  return message
}

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
      { type: 'authenticate', protocolVersion: 16, token: sessionToken },
      { type: 'joinRoom', sequence: 1, roomCode: 'ABCDEFGH' },
    ])
    socket.message(admitted())
    expect(value.client.getState().youSeat).toBe('one')
    expect(JSON.stringify(value.client.getState())).not.toContain(resumeToken)
    expect(socket.sent.join()).not.toContain(resumeToken)
    expect(localStorage).toHaveLength(0)
    expect(sessionStorage).toHaveLength(0)
  })

  it.each(['three', 'four'] as const)('admits and resumes own seat %s with controls intact', async (seat) => {
    const value = harness()
    const socket = await authenticate(value, { type: 'join', roomCode: 'ABCDEFGH' })
    const initial = multiplayerAdmission(seat, 1)
    initial.room.settings.enabledItems = ['teleporter']
    socket.message(initial)
    expect(value.client.getState()).toMatchObject({ youSeat: seat, connection: 'authenticated' })
    expect(value.client.movement('east')).toBe(true)

    socket.emit('close')
    value.scheduled.shift()!()
    await vi.waitFor(() => expect(value.sockets).toHaveLength(2))
    const resumed = value.sockets[1]
    resumed.emit('open')
    resumed.message({ type: 'authenticated', protocolVersion: 16 })
    const baseline = multiplayerAdmission(seat, 2)
    baseline.room.settings.enabledItems = ['teleporter']
    if (!baseline.state) throw new Error('resume fixture must contain state')
    baseline.state.physicalTick = 8
    baseline.state.runningUpdates = 8
    baseline.room.tick = 8
    const partner = seat === 'three' ? 2 : 3
    Object.assign(baseline.state.actors[0]!, {
      position: { x: 900, y: 12_000 }, lastTeleportTick: 5,
    })
    Object.assign(baseline.state.actors[partner]!, {
      position: { x: 2_000, y: 700 }, lastTeleportTick: 5,
    })
    resumed.message(baseline)
    expect(value.client.getState()).toMatchObject({
      youSeat: seat, connection: 'authenticated', admissionPending: false, failure: null,
    })
    expect(value.client.movement('west')).toBe(true)
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
    baseline.room.settings.enabledItems = baseline.room.settings.enabledItems
      .filter((item) => item !== 'punch')
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
    resumed.message({ type: 'authenticated', protocolVersion: 16 })
    expect(messages(resumed)).toEqual([
      { type: 'authenticate', protocolVersion: 16, token: sessionToken },
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
    resumed.message({ type: 'authenticated', protocolVersion: 16 })
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
      roomId, revision: 8, tick: 8, physicalTick: 8,
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
    resumed.message({ type: 'authenticated', protocolVersion: 16 })
    stop(resumed)
    expect(value.client.getState()).toMatchObject({
      connection: 'disconnected', room: null, snapshot: null, youSeat: null,
      admissionPending: false,
    })
    expect(value.scheduled).toHaveLength(0)
    expect(value.client.admit({ type: 'create' })).toBe(true)
  })
})
