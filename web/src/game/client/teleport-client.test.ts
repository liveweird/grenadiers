import { beforeEach, describe, expect, it, vi } from 'vitest'
import { admitted, authenticate, harness, snapshot, SocketStub } from './testSupport'
import { preservesTeleportHistory } from './teleportHistory'

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

function multiplayer(count: 3 | 4): ReturnType<typeof snapshot> {
  const message = snapshot(1)
  const one = message.state.actors[0]!
  const two = message.state.actors[1]!
  if (one.type !== 'visible' || two.type !== 'visible') throw new Error('fixture actors must be visible')
  message.state.actors.push({
    ...one, id: 3, seat: 'three', position: { x: 512, y: 12_800 },
  })
  if (count === 4) message.state.actors.push({
    ...two, id: 4, seat: 'four', position: { x: 14_848, y: 512 },
  })
  return message
}

function visible(message: ReturnType<typeof snapshot>, index: number) {
  const actor = message.state.actors[index]
  if (!actor || actor.type !== 'visible') throw new Error('fixture actor must be visible')
  return actor
}

describe('v12 teleport position epochs', () => {
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
    resumed.message({ type: 'authenticated', protocolVersion: 16 })
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

describe('v14 individual multiplayer teleport markers', () => {
  it('accepts an own-seat pair swap without changing the third actor marker', () => {
    const previous = multiplayer(3)
    const next = structuredClone(previous.state)
    next.physicalTick = 8
    Object.assign(next.actors[0]!, { position: { x: 700, y: 12_800 }, lastTeleportTick: 5 })
    Object.assign(next.actors[2]!, { position: { x: 900, y: 512 }, lastTeleportTick: 5 })
    expect(preservesTeleportHistory(previous, previous.room, next)).toBe(true)
    expect(next.actors[1]).toMatchObject({ lastTeleportTick: null })
  })

  it('accepts only the latest pair change while retaining an older four-player pair marker', () => {
    const previous = multiplayer(4)
    Object.assign(visible(previous, 0), { lastTeleportTick: 1 })
    Object.assign(visible(previous, 1), { lastTeleportTick: 1 })
    const next = structuredClone(previous.state)
    next.physicalTick = 9
    Object.assign(next.actors[2]!, { position: { x: 14_000, y: 700 }, lastTeleportTick: 6 })
    Object.assign(next.actors[3]!, { position: { x: 700, y: 12_000 }, lastTeleportTick: 6 })
    expect(preservesTeleportHistory(previous, previous.room, next)).toBe(true)
    expect(next.actors.slice(0, 2).map((actor) =>
      actor.type === 'visible' ? actor.lastTeleportTick : 'hidden')).toEqual([1, 1])
  })

  it('accepts visible timing for a privacy-hidden pair but still rejects a visible rollback', () => {
    const previous = multiplayer(4)
    const next = structuredClone(previous.state)
    const hidden = next.actors[3]!
    Object.assign(next.actors[2]!, {
      position: visible(previous, 3).position, lastTeleportTick: 2,
    })
    next.actors[3] = { type: 'hidden', id: hidden.id, seat: hidden.seat, alive: true }
    expect(preservesTeleportHistory(previous, previous.room, next)).toBe(true)

    Object.assign(visible(previous, 0), { lastTeleportTick: 2 })
    const rollback = structuredClone(previous.state)
    Object.assign(rollback.actors[0]!, { lastTeleportTick: 1 })
    expect(preservesTeleportHistory(previous, previous.room, rollback)).toBe(false)
  })

  it('rejects an observable unpaired marker change', () => {
    const previous = multiplayer(3)
    const next = structuredClone(previous.state)
    Object.assign(next.actors[0]!, { position: { x: 2_048, y: 512 }, lastTeleportTick: 2 })
    expect(preservesTeleportHistory(previous, previous.room, next)).toBe(false)
  })

  it('accepts a three-actor marker chain when the newest B/C pair is observable', () => {
    const previous = multiplayer(3)
    const next = structuredClone(previous.state)
    Object.assign(next.actors[0]!, { position: { x: 2_000, y: 700 }, lastTeleportTick: 4 })
    Object.assign(next.actors[1]!, { position: { x: 8_000, y: 8_000 }, lastTeleportTick: 7 })
    Object.assign(next.actors[2]!, { position: { x: 13_000, y: 1_000 }, lastTeleportTick: 7 })
    expect(preservesTeleportHistory(previous, previous.room, next)).toBe(true)
  })

  it('accepts two disjoint pairs batched into one four-player snapshot', () => {
    const previous = multiplayer(4)
    const next = structuredClone(previous.state)
    Object.assign(next.actors[0]!, { lastTeleportTick: 3 })
    Object.assign(next.actors[1]!, { lastTeleportTick: 3 })
    Object.assign(next.actors[2]!, { lastTeleportTick: 8 })
    Object.assign(next.actors[3]!, { lastTeleportTick: 8 })
    expect(preservesTeleportHistory(previous, previous.room, next)).toBe(true)
  })
})
