import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { SpikePitView } from '../v19/schema'
import {
  admitted, authenticate, harness, room, snapshot, SocketStub,
} from './testSupport'

const layoutA = (phase: 'idle' | 'warning' = 'idle', remainingTicks = 0): SpikePitView[] => [
  { column: 4, row: 4 }, { column: 10, row: 4 },
  { column: 4, row: 8 }, { column: 10, row: 8 },
].map((cell) => phase === 'idle'
  ? { cell, phase, remainingTicks: 0 }
  : { cell, phase, remainingTicks })
const layoutB = (): SpikePitView[] => [
  { cell: { column: 3, row: 3 }, phase: 'idle', remainingTicks: 0 },
  { cell: { column: 11, row: 3 }, phase: 'idle', remainingTicks: 0 },
  { cell: { column: 3, row: 9 }, phase: 'idle', remainingTicks: 0 },
  { cell: { column: 11, row: 9 }, phase: 'idle', remainingTicks: 0 },
]

describe('v18 spike-pit client history', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN }))

  it('accepts authoritative phase progress but rejects same-round cell removal or replacement', async () => {
    for (const replacement of [[], layoutB()]) {
      const value = harness()
      const socket = await authenticate(value)
      const baseline = admitted(1)
      baseline.state!.spikePits = layoutA()
      socket.message(baseline)
      const phaseUpdate = snapshot(2)
      phaseUpdate.state.spikePits = layoutA('warning', 30)
      socket.message(phaseUpdate)
      expect(value.client.getState().snapshot?.state.spikePits[0]).toMatchObject({
        phase: 'warning', remainingTicks: 30,
      })
      const changed = snapshot(3)
      changed.state.spikePits = replacement
      socket.message(changed)
      expect(value.client.getState()).toMatchObject({
        connection: 'disconnected', failure: 'invalidServerMessage',
      })
    }
  })

  it('accepts a new-round layout and a same-round reconnect baseline with lower timers', async () => {
    const value = harness()
    const socket = await authenticate(value)
    const baseline = admitted(5)
    baseline.state!.spikePits = layoutA('warning', 30)
    socket.message(baseline)
    socket.emit('close')
    value.scheduled.shift()!()
    await vi.waitFor(() => expect(value.sockets).toHaveLength(2))
    const resumed = value.sockets[1]
    resumed.emit('open')
    resumed.message({ type: 'authenticated', protocolVersion: 19 })
    const reconnect = admitted(8)
    reconnect.state!.spikePits = layoutA('warning', 12)
    resumed.message(reconnect)
    expect(value.client.getState().snapshot?.state.spikePits[0]?.remainingTicks).toBe(12)

    const nextRound = snapshot(9, 2)
    nextRound.state.spikePits = layoutB()
    resumed.message(nextRound)
    expect(value.client.getState().snapshot?.state.spikePits).toEqual(layoutB())
  })

  it('rejects mismatched spike tuning from an otherwise valid roomState', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted(1))
    const changed = room(2, { type: 'running' })
    changed.configuration.environment.spikePits.warningTicks = 37
    socket.message({ type: 'roomState', room: changed })
    expect(value.client.getState()).toMatchObject({
      connection: 'disconnected', failure: 'invalidServerMessage',
    })
  })
})
