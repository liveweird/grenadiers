import { beforeEach, describe, expect, it, vi } from 'vitest'
import { gameplayUrl } from './client'
import { authenticate, harness, messages, room, roomId, snapshot, SocketStub, token } from './testSupport'

describe('v3 GameClient transport and epochs', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN }))

  it('authenticates as protocol 3 without placing credentials in the URL', async () => {
    const value = harness()
    const socket = await authenticate(value, { type: 'join', roomCode: 'ABCDEFGH' })
    expect(value.fetcher).toHaveBeenCalledWith('/api/v1/game/session', expect.objectContaining({ signal: expect.any(AbortSignal) }))
    expect(messages(socket)).toEqual([
      { type: 'authenticate', protocolVersion: 3, token },
      { type: 'joinRoom', sequence: 1, roomCode: 'ABCDEFGH' },
    ])
  })

  it('derives secure and local gameplay URLs', () => {
    expect(gameplayUrl({ protocol: 'https:', host: 'game.test' } as Location)).toBe('wss://game.test/api/v1/game/ws')
    expect(gameplayUrl({ protocol: 'http:', host: 'localhost:5177' } as Location)).toBe('ws://localhost:5177/api/v1/game/ws')
  })

  it('accepts a coalesced newer-epoch snapshot and resets only the action counter', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message({ type: 'roomState', room: room(1), youSeat: 'one' })
    socket.message(snapshot(2))
    expect(value.client.placeBomb()).toBe(true)

    const nextEpoch = snapshot(3, 4)
    nextEpoch.room.roundNumber = 2
    socket.message(nextEpoch)
    expect(value.client.getState().room?.roundId).toBe(4)
    expect(value.client.getState().snapshot).toEqual(nextEpoch)
    expect(value.client.placeBomb()).toBe(true)
    expect(messages(socket).slice(-2)).toEqual([
      { type: 'placeBomb', actionId: 1, sequence: 2, roomId, roundId: 1 },
      { type: 'placeBomb', actionId: 1, sequence: 3, roomId, roundId: 4 },
    ])

    socket.message(snapshot(4, 2))
    expect(value.client.placeBomb()).toBe(true)
    expect(messages(socket).at(-1)).toEqual({
      type: 'placeBomb', actionId: 2, sequence: 4, roomId, roundId: 4,
    })
  })

  it('rejects older epochs, nonincreasing revisions, wrong rooms, and seat changes', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message({ type: 'roomState', room: room(1), youSeat: 'one' })
    socket.message(snapshot(3, 3))
    const accepted = value.client.getState().snapshot

    socket.message(snapshot(4, 2))
    socket.message(snapshot(3, 4))
    socket.message({ ...snapshot(5, 4), room: { ...snapshot(5, 4).room, roomId: 'c'.repeat(32) } })
    socket.message({ type: 'roomState', room: room(6, { type: 'running' }, 4), youSeat: 'two' })
    expect(value.client.getState().snapshot).toBe(accepted)
    expect(value.client.getState().youSeat).toBe('one')
  })

  it('limits readiness to initial phases and rematch voting to the authoritative gate', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message({ type: 'roomState', room: room(1), youSeat: 'one' })
    expect(value.client.ready(true)).toBe(true)

    socket.message({ type: 'roomState', room: room(2, { type: 'countdown', kind: 'interRound', endsAt: 180 }, 2), youSeat: 'one' })
    expect(value.client.ready(true)).toBe(false)
    const terminal = room(3, { type: 'matchResult', endedAt: 3, rematchAt: 183, result: { type: 'winner', seat: 'one', reason: 'roundWins' } }, 2)
    terminal.tick = 182
    socket.message({ type: 'roomState', room: terminal, youSeat: 'one' })
    expect(value.client.rematch(true)).toBe(false)

    socket.message({ type: 'roomState', room: { ...terminal, revision: 4, tick: 183 }, youSeat: 'one' })
    expect(value.client.rematch(true)).toBe(true)
    expect(value.client.rematch(false)).toBe(true)
    expect(messages(socket).slice(-2)).toEqual([
      { type: 'rematch', ready: true, sequence: 3, roomId, roundId: 2 },
      { type: 'rematch', ready: false, sequence: 4, roomId, roundId: 2 },
    ])
  })

  it('requires both occupied seats before sending a rematch vote', async () => {
    const value = harness()
    const socket = await authenticate(value)
    const terminal = room(1, { type: 'matchResult', endedAt: 1, rematchAt: 1, result: { type: 'abandoned' } })
    terminal.seats[1].occupied = false
    socket.message({ type: 'roomState', room: terminal, youSeat: 'one' })
    expect(value.client.rematch(true)).toBe(false)
  })
})

describe('v3 GameClient failure and cleanup', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN }))

  it('closes on authentication failure and keeps the typed failure after close', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message({ type: 'error', code: 'authenticationFailed', sequence: null })
    expect(value.client.getState().failure).toBe('authenticationFailed')
    socket.emit('close')
    expect(value.client.getState().failure).toBe('authenticationFailed')

    const malformed = harness()
    const malformedSocket = await authenticate(malformed)
    malformedSocket.message({ type: 'snapshot' })
    expect(malformedSocket.close).toHaveBeenCalled()
    expect(malformed.client.getState().failure).toBe('invalidServerMessage')
  })

  it('correlates leave, stops room commands, and retains the authenticated transport', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message({ type: 'roomState', room: room(1), youSeat: 'one' })
    expect(value.client.leave()).toBe(true)
    expect(value.client.ready(true)).toBe(false)
    socket.message({ type: 'leftRoom', roomId, sequence: 99 })
    expect(value.client.getState().room).not.toBeNull()
    socket.message({ type: 'leftRoom', roomId, sequence: 2 })
    expect(value.client.getState().room).toBeNull()
    expect(value.client.getState().connection).toBe('authenticated')
  })

  it('bounds transport sequences and action identifiers', async () => {
    const sequence = harness()
    const sequenceSocket = await authenticate(sequence)
    sequenceSocket.message({ type: 'roomState', room: room(1), youSeat: 'one' })
    Reflect.set(sequence.client, 'nextSequence', 2_147_483_648)
    expect(sequence.client.ready(true)).toBe(false)
    expect(sequence.client.getState().failure).toBe('sequenceExhausted')

    const action = harness()
    const actionSocket = await authenticate(action)
    actionSocket.message({ type: 'roomState', room: room(1), youSeat: 'one' })
    actionSocket.message(snapshot(2))
    Reflect.set(action.client, 'nextAction', 2_147_483_648)
    expect(action.client.placeBomb()).toBe(false)
    expect(action.client.getState().failure).toBe('sequenceExhausted')
  })

  it('aborts session work and closes the socket when disposed', async () => {
    const value = harness()
    value.client.admit({ type: 'create' })
    await vi.waitFor(() => expect(value.sockets).toHaveLength(1))
    const signal = value.fetcher.mock.calls[0][1].signal as AbortSignal
    const socket = value.sockets[0]
    value.client.dispose()
    expect(signal.aborted).toBe(true)
    expect(socket.close).toHaveBeenCalled()
  })
})
