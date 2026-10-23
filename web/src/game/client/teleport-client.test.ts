import { beforeEach, describe, expect, it, vi } from 'vitest'
import { admitted, authenticate, harness, snapshot, SocketStub } from './testSupport'

function mark<T extends ReturnType<typeof admitted> | ReturnType<typeof snapshot>>(
  message: T, marker: number | null,
): T {
  message.room.settings.enabledItems = ['teleporter']
  if (!message.state) return message
  for (const actor of message.state.actors) {
    if (actor.type === 'visible') actor.lastTeleportTick = marker
  }
  return message
}

describe('v10 teleport position epochs', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN }))

  it('accepts equal and increasing history but rejects a same-round rollback atomically', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(mark(admitted(), 1))
    const unchanged = mark(snapshot(2), 1)
    unchanged.state.physicalTick = 2
    unchanged.state.runningUpdates = 2
    socket.message(unchanged)
    expect(value.client.getState().snapshot?.room.revision).toBe(2)

    const rollback = mark(snapshot(3), null)
    rollback.state.physicalTick = 3
    rollback.state.runningUpdates = 3
    socket.message(rollback)
    expect(value.states.some((state) => state.room?.revision === 3)).toBe(false)
    expect(value.client.getState()).toMatchObject({
      connection: 'disconnected', room: null, snapshot: null, failure: 'invalidServerMessage',
    })
  })

  it('ignores a stale rollback before checking history', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(mark(admitted(2), 1))
    socket.message(mark(snapshot(1), null))
    expect(value.client.getState()).toMatchObject({ connection: 'authenticated', failure: null })
    expect(value.client.getState().snapshot?.room.revision).toBe(2)
  })

  it('retains history across resume admission and initializes it for a new round', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(mark(admitted(), 1))
    socket.emit('close')
    value.scheduled.shift()!()
    await vi.waitFor(() => expect(value.sockets).toHaveLength(2))
    const resumed = value.sockets[1]
    resumed.emit('open')
    resumed.message({ type: 'authenticated', protocolVersion: 10 })
    resumed.message(mark(admitted(2), null))
    expect(value.client.getState().failure).toBe('invalidServerMessage')

    const fresh = harness()
    const freshSocket = await authenticate(fresh)
    freshSocket.message(mark(admitted(), 1))
    const nextRound = mark(snapshot(2, 2), null)
    freshSocket.message(nextRound)
    expect(fresh.client.getState().snapshot?.room.roundId).toBe(2)
    expect(fresh.client.getState().failure).toBeNull()
  })
})
