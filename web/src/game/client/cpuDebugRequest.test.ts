import { describe, expect, it } from 'vitest'
import { admitted, authenticate, harness, resumeToken } from './testSupport'

describe('private CPU debug requests', () => {
  it('uses the private resume capability without copying it into observable state', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    const response = new Response('{}', { status: 200 })
    value.fetcher.mockResolvedValueOnce(response)

    const result = await value.client.requestCpuDebug(new AbortController().signal)

    expect(result.type).toBe('response')
    expect(value.fetcher).toHaveBeenLastCalledWith('/api/v1/game/cpu-debug', expect.objectContaining({
      method: 'POST', cache: 'no-store', credentials: 'omit', body: JSON.stringify({ roomId: 'b'.repeat(32) }),
      headers: expect.objectContaining({ Authorization: `Bearer ${resumeToken}` }),
    }))
    expect(JSON.stringify(value.client.getState())).not.toContain(resumeToken)
  })

  it('returns unavailable without a private admitted capability and makes no diagnostic request', async () => {
    const value = harness()
    const result = await value.client.requestCpuDebug(new AbortController().signal)
    expect(result).toEqual({ type: 'unavailable' })
    expect(value.fetcher).not.toHaveBeenCalled()
  })

  it('drops a response after the connection generation changes', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    let resolveResponse!: (response: Response) => void
    value.fetcher.mockImplementationOnce(() => new Promise((resolve) => { resolveResponse = resolve }))
    const pending = value.client.requestCpuDebug(new AbortController().signal)
    value.client.disconnect()
    resolveResponse(new Response('{}', { status: 200 }))
    await expect(pending).resolves.toEqual({ type: 'stale' })
  })

  it('passes the caller abort signal to the credentialed fetch', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    value.fetcher.mockResolvedValueOnce(new Response('{}', { status: 200 }))
    const controller = new AbortController()
    await value.client.requestCpuDebug(controller.signal)
    expect(value.fetcher).toHaveBeenLastCalledWith('/api/v1/game/cpu-debug',
      expect.objectContaining({ signal: controller.signal }))
  })
})
