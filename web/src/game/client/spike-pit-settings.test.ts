import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { GameSettings } from '../v20/schema'
import { admitted, authenticate, harness, room, SocketStub } from './testSupport'

describe('v18 authoritative spike-pit settings', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN }))

  it('rejects fractional and out-of-range tick settings', async () => {
    const value = harness()
    const socket = await authenticate(value)
    const baseline = admitted()
    baseline.room = room(1)
    baseline.state = null
    socket.message(baseline)
    const sent = socket.sent.length; const settings = baseline.room.settings
    for (const spikePits of [
      { enabled: true, warningTicks: 5, raisedTicks: 180 },
      { enabled: true, warningTicks: 121, raisedTicks: 180 },
      { enabled: true, warningTicks: 36.5, raisedTicks: 180 },
      { enabled: true, warningTicks: 36, raisedTicks: 59 },
      { enabled: true, warningTicks: 36, raisedTicks: 601 },
    ]) {
      expect(value.client.updateSettings({
        ...settings, environment: { ...settings.environment,
          spikePits: spikePits as GameSettings['environment']['spikePits'] },
      })).toBe(false)
    }
    expect(socket.sent).toHaveLength(sent)
  })
})
