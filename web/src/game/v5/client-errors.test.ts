import { beforeEach, describe, expect, it, vi } from 'vitest'
import { admitted, authenticate, harness, messages, room, roomId, snapshot, SocketStub } from './testSupport'

describe('v5 session and admission failures', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN }))

  it.each([
    null,
    {},
    { token: 'short' },
    { token: 'A'.repeat(64) },
    { token: 'a'.repeat(64), extra: true },
    ['a'.repeat(64)],
  ])('rejects a malformed session payload and permits a fresh admission: %j', async (payload) => {
    const value = harness(payload)
    expect(value.client.admit({ type: 'create' })).toBe(true)
    await vi.waitFor(() => expect(value.client.getState().failure).toBe('sessionFailed'))
    expect(value.sockets).toHaveLength(0)
    value.fetcher.mockResolvedValueOnce({ ok: true, json: async () => ({ token: 'a'.repeat(64) }) })
    expect(value.client.admit({ type: 'join', roomCode: 'ABCDEFGH' })).toBe(true)
  })

  it.each([
    ['HTTP rejection', vi.fn().mockResolvedValue({ ok: false })],
    ['network rejection', vi.fn().mockRejectedValue(new Error('offline'))],
    ['body rejection', vi.fn().mockResolvedValue({ ok: true, json: vi.fn().mockRejectedValue(new Error('bad body')) })],
  ])('settles %s without creating a socket', async (_label, fetcher) => {
    const value = harness()
    value.fetcher.mockImplementation(fetcher)
    value.client.admit({ type: 'create' })
    await vi.waitFor(() => expect(value.client.getState().failure).toBe('sessionFailed'))
    expect(value.sockets).toHaveLength(0)
    expect(value.client.getState().admissionPending).toBe(false)
  })

  it('ignores duplicate authentication and admission with wrong correlation', async () => {
    const value = harness()
    value.client.admit({ type: 'create' })
    await vi.waitFor(() => expect(value.sockets).toHaveLength(1))
    const socket = value.sockets[0]
    socket.emit('open')
    socket.message({ type: 'authenticated', protocolVersion: 5 })
    socket.message({ type: 'authenticated', protocolVersion: 5 })
    expect(messages(socket).filter((message) => message.type === 'createRoom')).toHaveLength(1)
    socket.message({ ...admitted(), sequence: 2 })
    expect(value.client.getState().room).toBeNull()
    socket.message(admitted())
    expect(value.client.getState().room?.roomId).toBe(roomId)
  })

  it('accepts a nonphysical admitted baseline with a fresh action floor', async () => {
    const value = harness()
    const socket = await authenticate(value)
    const baseline = admitted()
    baseline.room = room(1)
    baseline.state = null
    socket.message(baseline)
    expect(value.client.getState().snapshot).toBeNull()
    expect(Reflect.get(value.client, 'nextAction')).toBe(1)
  })

  it.each([
    [{ type: 'snapshot' }, 'invalidServerMessage'],
    ['not-json', 'invalidServerMessage'],
    [{ type: 'incompatibleProtocol', supportedVersion: 3 }, 'incompatibleProtocol'],
  ])('tears down a terminal protocol failure %#', async (frame, failure) => {
    const value = harness()
    const socket = await authenticate(value)
    if (typeof frame === 'string') socket.emit('message', frame)
    else socket.message(frame)
    expect(socket.close).toHaveBeenCalled()
    expect(value.client.getState()).toMatchObject({ connection: 'disconnected', failure })
    socket.emit('close')
    expect(value.scheduled).toHaveLength(0)
  })
})

describe('v5 authoritative updates and controls', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN }))

  it('accepts only newer same-room nonregressing shared facts', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted(2))
    const original = value.client.getState().snapshot
    socket.message({ type: 'roomState', room: room(3, { type: 'running' }) })
    expect(value.client.getState().room?.revision).toBe(3)
    expect(value.client.getState().snapshot).toBe(original)
    socket.message({ type: 'roomState', room: room(3, { type: 'running' }) })
    socket.message({ type: 'roomState', room: { ...room(4, { type: 'running' }), roomId: 'd'.repeat(32) } })
    expect(value.client.getState().room?.revision).toBe(3)

    socket.message(snapshot(4, 2))
    expect(value.client.getState().snapshot?.room.roundId).toBe(2)
    socket.message(snapshot(5, 1))
    expect(value.client.getState().snapshot?.room.revision).toBe(4)
  })

  it('resets world and bomb floor on a room-only epoch advance', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted(2, 9))
    socket.message({ type: 'roomState', room: room(3, { type: 'countdown', kind: 'interRound', endsAt: 183 }, 2) })
    expect(value.client.getState().snapshot).toBeNull()
    expect(Reflect.get(value.client, 'nextAction')).toBe(1)
  })

  it('enforces connected phase gates for ready, rematch and live controls', async () => {
    const value = harness()
    const socket = await authenticate(value)
    const waiting = admitted()
    waiting.room = room(1)
    waiting.state = null
    socket.message(waiting)
    expect(value.client.ready(true)).toBe(true)
    expect(value.client.movement('east')).toBe(false)

    const terminal = room(2, {
      type: 'matchResult', endedAt: 1, rematchAt: 3,
      result: { type: 'winner', seat: 'one', reason: 'roundWins' },
    }, 1)
    terminal.tick = 2
    socket.message({ type: 'roomState', room: terminal })
    expect(value.client.rematch(true)).toBe(false)
    terminal.revision = 3
    terminal.tick = 3
    terminal.seats[1].connection = { type: 'disconnected', expiresAt: 10 }
    socket.message({ type: 'roomState', room: terminal })
    expect(value.client.rematch(true)).toBe(false)
    terminal.revision = 4
    terminal.seats[1].connection = { type: 'connected' }
    socket.message({ type: 'roomState', room: terminal })
    expect(value.client.rematch(true)).toBe(true)
  })

  it('correlates leave acknowledgement and errors without clearing the room early', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    expect(value.client.leave()).toBe(true)
    expect(value.client.leave()).toBe(false)
    socket.message({ type: 'leftRoom', roomId, sequence: 99 })
    expect(value.client.getState().room).not.toBeNull()
    const sequence = (messages(socket).at(-1) as { sequence: number }).sequence
    socket.message({ type: 'error', code: 'invalidPhase', sequence })
    expect(value.client.getState().leavePending).toBe(false)
    expect(value.client.getState().room).not.toBeNull()
    socket.message({ type: 'roomState', room: room(2, { type: 'running' }) })
    expect(value.client.getState().room?.revision).toBe(2)
    socket.emit('close')
    expect(value.client.getState().connection).toBe('reconnecting')
    expect(value.scheduled).toHaveLength(1)
  })

  it('clears personal room state only for the matching leave acknowledgement', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    expect(value.client.leave()).toBe(true)
    const sequence = (messages(socket).at(-1) as { sequence: number }).sequence
    socket.message({ type: 'leftRoom', roomId: 'd'.repeat(32), sequence })
    expect(value.client.getState().room).not.toBeNull()
    socket.message({ type: 'leftRoom', roomId, sequence })
    expect(value.client.getState()).toMatchObject({ room: null, snapshot: null, youSeat: null, leavePending: false })
  })

  it('settles an admission error and permits another command on the authenticated socket', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message({ type: 'error', code: 'roomFull', sequence: 1 })
    expect(value.client.getState()).toMatchObject({ admissionPending: false, failure: 'roomFull' })
    expect(value.client.admit({ type: 'create' })).toBe(true)
    expect(messages(socket).at(-1)).toEqual({ type: 'createRoom', sequence: 2 })
  })

  it('clears a correlated obsolete movement error after authoritative progress', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    expect(value.client.movement('east')).toBe(true)
    const sequence = (messages(socket).at(-1) as { sequence: number }).sequence
    socket.message({ type: 'error', code: 'invalidPhase', sequence })
    expect(value.client.getState().failure).toBe('invalidPhase')
    socket.message({ type: 'roomState', room: room(2, { type: 'result', holdUntil: 181 }) })
    expect(value.client.getState().failure).toBeNull()
  })

  it('bounds transport sequences independently of resumed action IDs', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    Reflect.set(value.client, 'nextSequence', 2_147_483_648)
    expect(value.client.movement('north')).toBe(false)
    expect(value.client.getState().failure).toBe('sequenceExhausted')
  })
})

describe('v5 retry callback fences', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN }))

  it('does not run a canceled reconnect timer after explicit disconnect', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    socket.emit('close')
    const staleTimer = value.scheduled.shift()!
    value.client.disconnect()
    staleTimer()
    await Promise.resolve()
    expect(value.sockets).toHaveLength(1)
    expect(value.client.getState().room).toBeNull()
  })

  it('ignores callbacks from the replaced socket generation', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    socket.emit('close')
    value.scheduled.shift()!()
    await vi.waitFor(() => expect(value.sockets).toHaveLength(2))
    socket.message(snapshot(99))
    socket.emit('open')
    expect(value.client.getState().room?.revision).toBe(1)
  })

  it('bounds repeated asynchronous session failures during resume', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    value.fetcher.mockRejectedValue(new Error('offline'))
    socket.emit('close')
    for (let attempt = 1; attempt <= 3; attempt += 1) {
      value.scheduled.shift()!()
      await vi.waitFor(() => {
        if (attempt < 3) expect(value.client.getState().reconnectAttempt).toBe(attempt + 1)
        else expect(value.client.getState().connection).toBe('disconnected')
      })
    }
    expect(value.fetcher).toHaveBeenCalledTimes(4)
    expect(value.sockets).toHaveLength(1)
    expect(value.client.getState().failure).toBe('connectionFailed')
    expect(value.client.getState()).toMatchObject({
      room: null, snapshot: null, youSeat: null, admissionPending: false,
    })
  })

  it('ignores a delayed resume session body after disconnect cancels its generation', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    let resolve!: (response: Response) => void
    value.fetcher.mockReturnValueOnce(new Promise<Response>((done) => { resolve = done }))
    socket.emit('close')
    value.scheduled.shift()!()
    await vi.waitFor(() => expect(value.fetcher).toHaveBeenCalledTimes(2))
    value.client.disconnect()
    resolve({ ok: true, json: async () => ({ token: 'a'.repeat(64) }) } as Response)
    await Promise.resolve()
    expect(value.sockets).toHaveLength(1)
    expect(value.client.getState().room).toBeNull()
  })
})
