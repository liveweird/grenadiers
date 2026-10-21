import { beforeEach, describe, expect, it, vi } from 'vitest'
import { admitted, authenticate, harness, messages, room, SocketStub } from './testSupport'

const changed = {
  startingSpeedLevel: 4,
  turnAssistUnits: 64 as const,
  winsToFinish: 1,
  bombFuseTicks: 60 as const,
  enabledItems: [],
}

describe('v11 authoritative gameplay settings', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN }))

  it('sends an atomic revision-fenced update and fences readiness until acceptance', async () => {
    const value = harness()
    const socket = await authenticate(value)
    const baseline = admitted()
    baseline.room = room(1)
    baseline.state = null
    socket.message(baseline)

    expect(value.client.updateSettings(changed)).toBe(true)
    expect(messages(socket).at(-1)).toEqual({
      type: 'updateSettings', sequence: 2, roomId: baseline.room.roomId, roundId: 1,
      settingsRevision: 1, settings: changed,
    })
    expect(value.client.getState().settingsPending).toBe(true)
    expect(value.client.ready(true)).toBe(false)

    const accepted = room(2)
    accepted.settings = changed
    accepted.settingsRevision = 2
    socket.message({ type: 'roomState', room: accepted })
    expect(value.client.getState()).toMatchObject({ settingsPending: false, room: accepted })
    expect(value.client.ready(true)).toBe(true)
    expect(messages(socket).at(-1)).toMatchObject({
      type: 'ready', settingsRevision: 2, ready: true,
    })
  })

  it('settles an explicit no-op response but rejects non-editor and ready edits', async () => {
    const value = harness()
    const socket = await authenticate(value)
    const baseline = admitted()
    baseline.room = room(1)
    baseline.state = null
    socket.message(baseline)
    expect(value.client.updateSettings(baseline.room.settings)).toBe(true)
    expect(value.client.getState().settingsPending).toBe(true)
    socket.message({ type: 'roomState', room: baseline.room })
    expect(value.client.getState().settingsPending).toBe(false)
    const count = socket.sent.length

    baseline.room.settingsEditor = 'two'
    socket.message({ type: 'roomState', room: { ...baseline.room, revision: 2 } })
    expect(value.client.updateSettings(changed)).toBe(false)
    const readyRoom = room(3)
    readyRoom.seats[0].ready = true
    socket.message({ type: 'roomState', room: readyRoom })
    expect(value.client.updateSettings(changed)).toBe(false)
    expect(socket.sent).toHaveLength(count)
  })

  it('clears pending state on correlated errors and reconnect boundaries', async () => {
    const value = harness()
    const socket = await authenticate(value)
    const baseline = admitted()
    baseline.room = room(1)
    baseline.state = null
    socket.message(baseline)
    value.client.updateSettings(changed)
    socket.message({ type: 'error', code: 'staleSettings', sequence: 2 })
    expect(value.client.getState()).toMatchObject({ settingsPending: false, failure: 'staleSettings' })

    expect(value.client.updateSettings(changed)).toBe(true)
    socket.emit('close')
    expect(value.client.getState()).toMatchObject({ connection: 'reconnecting', settingsPending: false })
  })

  it('same-revision acknowledgements cannot replace authoritative facts or apply changed rules', async () => {
    const value = harness()
    const socket = await authenticate(value)
    const baseline = admitted()
    baseline.room = room(1)
    baseline.state = null
    socket.message(baseline)
    const accepted = value.client.getState().room

    value.client.updateSettings(changed)
    socket.message({ type: 'roomState', room: { ...baseline.room, settings: changed } })
    expect(value.client.getState().settingsPending).toBe(true)
    expect(value.client.getState().room).toBe(accepted)
    socket.message({ type: 'error', code: 'staleSettings', sequence: 2 })

    value.client.updateSettings(baseline.room.settings)
    socket.message({ type: 'roomState', room: { ...baseline.room, tick: 900 } })
    expect(value.client.getState().settingsPending).toBe(false)
    expect(value.client.getState().room).toBe(accepted)
    expect(value.client.getState().room?.tick).toBe(baseline.room.tick)
    expect(value.client.getState().snapshot).toBeNull()
  })

  it('sends independent optional-item masks but rejects unavailable item configuration', async () => {
    const value = harness()
    const socket = await authenticate(value)
    const baseline = admitted()
    baseline.room = room(1)
    baseline.state = null
    socket.message(baseline)
    const sent = socket.sent.length

    const allItems = [...baseline.room.availableItems]
    expect(allItems).toHaveLength(22)
    expect(value.client.updateSettings({ ...baseline.room.settings, enabledItems: allItems })).toBe(true)
    expect(socket.sent).toHaveLength(sent + 1)
    expect(messages(socket).at(-1)).toMatchObject({
      type: 'updateSettings', settings: { enabledItems: allItems },
    })
    socket.message({ type: 'error', code: 'staleSettings', sequence: 2 })
    const tooMany: typeof allItems = [...allItems, 'kick']
    expect(value.client.updateSettings({ ...baseline.room.settings, enabledItems: tooMany })).toBe(false)
    expect(value.client.updateSettings({
      ...baseline.room.settings, enabledItems: ['kick', 'kick'],
    })).toBe(false)
    expect(socket.sent).toHaveLength(sent + 1)
    expect(value.client.updateSettings({
      ...baseline.room.settings, enabledItems: ['goldenFlame', 'shortFuse', 'shortFlame'],
    })).toBe(true)
    socket.message({ type: 'error', code: 'staleSettings', sequence: 3 })
    expect(value.client.updateSettings({
      ...baseline.room.settings, enabledItems: ['shortFlame'],
    })).toBe(true)
    socket.message({ type: 'error', code: 'staleSettings', sequence: 4 })
    expect(value.client.updateSettings({
      ...baseline.room.settings, enabledItems: ['shortFuse', 'heart'],
    })).toBe(false)
    expect(value.client.updateSettings({
      ...baseline.room.settings, enabledItems: ['teleporter'],
    })).toBe(true)
    expect(socket.sent).toHaveLength(sent + 4)
  })
})
