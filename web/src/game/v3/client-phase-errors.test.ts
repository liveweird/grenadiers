import { beforeEach, describe, expect, it, vi } from 'vitest'
import { authenticate, harness, messages, room, snapshot, SocketStub } from './testSupport'

describe('v3 GameClient phase-boundary control errors', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN }))

  it('clears only correlated control errors made obsolete by authoritative phase progress', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message({ type: 'roomState', room: room(1), youSeat: 'one' })
    socket.message(snapshot(2))
    expect(value.client.movement(null)).toBe(true)
    const stop = messages(socket).at(-1) as { sequence: number }
    socket.message({ type: 'error', code: 'invalidPhase', sequence: stop.sequence })
    expect(value.client.getState().failure).toBe('invalidPhase')

    socket.message({
      type: 'roomState', room: room(3, { type: 'result', nextCountdownAt: 183 }), youSeat: 'one',
    })
    expect(value.client.getState().failure).toBeNull()

    const coalesced = harness()
    const coalescedSocket = await authenticate(coalesced)
    coalescedSocket.message({ type: 'roomState', room: room(1), youSeat: 'one' })
    coalescedSocket.message(snapshot(2))
    expect(coalesced.client.movement('east')).toBe(true)
    const movement = messages(coalescedSocket).at(-1) as { sequence: number }
    coalescedSocket.message({
      type: 'roomState', room: room(3, { type: 'result', nextCountdownAt: 183 }), youSeat: 'one',
    })
    coalescedSocket.message({ type: 'error', code: 'staleRoom', sequence: movement.sequence })
    expect(coalesced.client.getState().failure).toBeNull()
  })

  it('keeps current-phase and non-control errors visible with bounded control context', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message({ type: 'roomState', room: room(1), youSeat: 'one' })
    socket.message(snapshot(2))
    for (let index = 0; index < 70; index += 1) expect(value.client.movement('north')).toBe(true)
    expect(Reflect.get(value.client, 'sentControls') as unknown[]).toHaveLength(64)
    const movement = messages(socket).at(-1) as { sequence: number }
    socket.message({ type: 'error', code: 'invalidPhase', sequence: movement.sequence })
    socket.message(snapshot(3))
    expect(value.client.getState().failure).toBe('invalidPhase')

    expect(value.client.placeBomb()).toBe(true)
    const bomb = messages(socket).at(-1) as { sequence: number }
    socket.message({ type: 'error', code: 'invalidPhase', sequence: bomb.sequence })
    socket.message({
      type: 'roomState', room: room(4, { type: 'result', nextCountdownAt: 184 }), youSeat: 'one',
    })
    expect(value.client.getState().failure).toBe('invalidPhase')
    socket.message({ type: 'error', code: 'duplicateAction', sequence: bomb.sequence })
    socket.message({
      type: 'roomState', room: room(5, { type: 'result', nextCountdownAt: 185 }), youSeat: 'one',
    })
    expect(value.client.getState().failure).toBe('duplicateAction')
  })
})
