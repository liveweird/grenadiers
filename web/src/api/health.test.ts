import { describe, expect, it, vi } from 'vitest'
import { getHealth } from './health'

describe('getHealth', () => {
  it('returns a valid health response', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, json: async () => ({ status: 'ok' }) }))
    await expect(getHealth()).resolves.toEqual({ status: 'ok' })
  })

  it('rejects a non-success response', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false, status: 503 }))
    await expect(getHealth()).rejects.toThrow('Health request failed (503)')
  })
})
