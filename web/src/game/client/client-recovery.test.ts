import { beforeEach, describe, expect, it, vi } from 'vitest'
import { admitted, authenticate, FakeClock, harness, messages, roomId, SocketStub } from './testSupport'

async function beginResume() {
  const value = harness()
  const original = await authenticate(value)
  original.message(admitted())
  original.emit('close')
  value.scheduled[0]()
  await vi.waitFor(() => expect(value.sockets).toHaveLength(2))
  const socket = value.sockets[1]
  socket.emit('open')
  socket.message({ type: 'authenticated', protocolVersion: 19 })
  return { value, socket }
}

function throttle(socket: SocketStub, sequence: number): void {
  socket.message({ type: 'error', code: 'rateLimited', sequence })
}

describe('v12 coordinated resume recovery', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN }))

  it('runs a scheduled fake-clock task only once', () => {
    const clock = new FakeClock()
    const callback = vi.fn()
    clock.schedule(callback, 1)
    const task = clock.scheduled[0]
    task()
    task()
    expect(callback).toHaveBeenCalledOnce()
  })

  it('retries only the correlated resume three times on the authenticated socket', async () => {
    const { value, socket } = await beginResume()
    expect(messages(socket).at(-1)).toMatchObject({ type: 'resumeRoom', sequence: 1 })
    expect(value.client.leave()).toBe(false)
    expect(messages(socket).some(({ type }) => type === 'leaveRoom')).toBe(false)

    for (const [sequence, expectedTime] of [[1, 500], [2, 1_000], [3, 1_750]] as const) {
      throttle(socket, sequence)
      value.scheduled[0]()
      expect(value.clock.now).toBe(expectedTime)
      expect(messages(socket).at(-1)).toMatchObject({ type: 'resumeRoom', sequence: sequence + 1 })
    }

    expect(value.fetcher).toHaveBeenCalledTimes(2)
    expect(value.sockets).toHaveLength(2)
    expect(messages(socket).filter(({ type }) => type === 'resumeRoom')).toHaveLength(4)
    throttle(socket, 4)
    expect(value.client.getState()).toMatchObject({
      connection: 'disconnected', room: null, snapshot: null, failure: 'rateLimited',
    })
    expect(socket.close).toHaveBeenCalled()
    expect(value.scheduled).toHaveLength(0)
  })

  it('does not treat null, stale, or duplicate errors as a pending resume correlation', async () => {
    const { value, socket } = await beginResume()
    socket.message({ type: 'error', code: 'rateLimited', sequence: null })
    socket.message({ type: 'error', code: 'rateLimited', sequence: 99 })
    expect(messages(socket).filter(({ type }) => type === 'resumeRoom')).toHaveLength(1)
    expect(value.scheduled).toHaveLength(1)

    throttle(socket, 1)
    const retry = value.scheduled[0]
    throttle(socket, 1)
    retry()
    expect(messages(socket).filter(({ type }) => type === 'resumeRoom')).toHaveLength(2)
    expect(messages(socket).some(({ type }) => type === 'manualAction')).toBe(false)
  })

  it('lets an initial throttled admission be retried manually on the same socket', async () => {
    const value = harness()
    const socket = await authenticate(value)
    throttle(socket, 1)
    expect(value.client.getState()).toMatchObject({ admissionPending: false, failure: 'rateLimited' })
    expect(value.scheduled).toHaveLength(0)

    expect(value.client.admit({ type: 'join', roomCode: 'ABCDEFGH' })).toBe(true)
    expect(messages(socket).at(-1)).toEqual({ type: 'joinRoom', sequence: 2, roomCode: 'ABCDEFGH' })
    expect(value.fetcher).toHaveBeenCalledTimes(1)
  })

  it('terminates a correlated non-throttle resume error without exposing the retained room', async () => {
    const { value, socket } = await beginResume()
    socket.message({ type: 'error', code: 'roomNotFound', sequence: 1 })
    expect(value.client.getState()).toMatchObject({
      connection: 'disconnected', room: null, snapshot: null, admissionPending: false,
      failure: 'roomNotFound',
    })
    expect(value.client.leave()).toBe(false)
  })

  it.each(['fetch', 'authentication', 'admission'] as const)(
    'enforces the absolute deadline while %s is pending',
    async (phase) => {
      const value = harness()
      const original = await authenticate(value)
      original.message(admitted())
      if (phase === 'fetch') value.fetcher.mockReturnValueOnce(new Promise<Response>(() => undefined))
      original.emit('close')
      if (phase !== 'fetch') {
        value.scheduled[0]()
        await vi.waitFor(() => expect(value.sockets).toHaveLength(2))
        if (phase === 'admission') {
          value.sockets[1].emit('open')
          value.sockets[1].message({ type: 'authenticated', protocolVersion: 19 })
        }
      }
      value.clock.advanceBy(10_000 - value.clock.now)
      expect(value.client.getState()).toMatchObject({
        connection: 'disconnected', room: null, snapshot: null, failure: 'connectionFailed',
      })
      const signal = (value.fetcher.mock.calls.at(-1)?.[1] as RequestInit | undefined)?.signal
      expect(signal?.aborted).toBe(true)
    },
  )

  it('cancels saved retry and deadline callbacks after an authoritative baseline', async () => {
    const { value, socket } = await beginResume()
    throttle(socket, 1)
    const retryTask = value.scheduled[0]
    const retry = value.clock.rawCallback(retryTask)
    retryTask()
    const deadline = value.clock.rawCallback(value.scheduled[0])
    const baseline = { ...admitted(8), sequence: 2 }
    socket.message(baseline)
    expect(value.client.getState()).toMatchObject({
      room: baseline.room, snapshot: { room: baseline.room }, admissionPending: false, failure: null,
    })
    expect(value.client.getState().admissionBaseline).toMatchObject({ roomId, revision: 8 })

    deadline()
    retry()
    expect(value.client.getState().room?.revision).toBe(8)
    expect(messages(socket).filter(({ type }) => type === 'resumeRoom')).toHaveLength(2)
  })

  it('rejects an admitted baseline observed after the absolute deadline', async () => {
    const { value, socket } = await beginResume()
    const deadline = value.clock.rawCallback(value.scheduled[0])
    value.clock.now = 10_000
    socket.message(admitted(8))

    expect(value.client.getState()).toMatchObject({
      connection: 'disconnected', room: null, snapshot: null, admissionPending: false,
      failure: 'connectionFailed',
    })
    expect(socket.close).toHaveBeenCalled()
    deadline()
    expect(value.client.getState().failure).toBe('connectionFailed')
  })

  it('uses the deadline failure when a correlated throttle arrives at the boundary', async () => {
    const { value, socket } = await beginResume()
    value.clock.now = 10_000
    throttle(socket, 1)
    expect(value.client.getState()).toMatchObject({
      connection: 'disconnected', room: null, snapshot: null, failure: 'connectionFailed',
    })
  })

  it('does not let an old retry callback detach a newer same-episode timer', async () => {
    const { value, socket } = await beginResume()
    throttle(socket, 1)
    const retry = value.clock.rawCallback(value.scheduled[0])
    socket.emit('close')
    retry()
    value.client.dispose()

    expect(messages(socket).filter(({ type }) => type === 'resumeRoom')).toHaveLength(1)
    expect(value.scheduled).toHaveLength(0)
  })

  it('does not let an old deadline callback detach a new recovery deadline', async () => {
    const { value, socket } = await beginResume()
    const deadline = value.clock.rawCallback(value.scheduled[0])
    socket.message(admitted(8))
    socket.emit('close')
    deadline()
    value.client.dispose()

    expect(value.scheduled).toHaveLength(0)
  })

  it.each(['dispose', 'close', 'replacement'] as const)(
    'fences a saved same-socket retry after %s',
    async (event) => {
      const { value, socket } = await beginResume()
      throttle(socket, 1)
      const retry = value.clock.rawCallback(value.scheduled[0])
      if (event === 'dispose') value.client.dispose()
      else if (event === 'close') {
        socket.emit('close')
        value.scheduled[0]()
        await vi.waitFor(() => expect(value.sockets).toHaveLength(3))
      } else {
        socket.message({ type: 'seatReplaced', roomId })
        expect(value.client.admit({ type: 'create' })).toBe(true)
        await vi.waitFor(() => expect(value.sockets).toHaveLength(3))
      }
      retry()
      expect(messages(socket).filter(({ type }) => type === 'resumeRoom')).toHaveLength(1)
    },
  )
})
