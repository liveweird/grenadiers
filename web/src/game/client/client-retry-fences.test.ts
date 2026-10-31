import { beforeEach, describe, expect, it, vi } from 'vitest'
import { admitted, authenticate, harness, snapshot, SocketStub } from './testSupport'

describe('v12 retry callback fences', () => {
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
