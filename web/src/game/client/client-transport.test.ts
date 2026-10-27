import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { GameClient } from './client'
import { authenticate, harness, sessionToken, SocketStub } from './testSupport'

describe('v13 session transport', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN }))
  afterEach(() => vi.unstubAllGlobals())

  it('posts for a session and opens the derived socket URL without credentials', async () => {
    const value = harness()
    const socket = await authenticate(value)

    expect(value.fetcher).toHaveBeenCalledWith('/api/v1/game/session', expect.objectContaining({
      method: 'POST',
      signal: expect.any(AbortSignal),
    }))
    expect(socket.url).toBe('wss://play.test/api/v1/game/ws')
    expect(socket.url).not.toContain(sessionToken)
  })

  it('keeps the global receiver when using the native fetch dependency', async () => {
    const fetcher = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ token: sessionToken }),
    })
    vi.stubGlobal('fetch', fetcher)
    const socketFactory = vi.fn((url: string) => new SocketStub(url) as unknown as WebSocket)
    const client = new GameClient(() => undefined, { socket: socketFactory })

    try {
      client.admit({ type: 'create' })
      await vi.waitFor(() => expect(socketFactory).toHaveBeenCalledOnce())
      expect(fetcher.mock.contexts).toEqual([globalThis])
    } finally {
      client.dispose()
    }
  })
})
