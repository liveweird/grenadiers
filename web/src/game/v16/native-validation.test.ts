import { describe, expect, it, vi } from 'vitest'

const legacyValidator = vi.fn(() => {
  throw new Error('v16 must not invoke the closed v15 validator')
})

vi.mock('../v15/validation', () => ({
  validateGameplayV15ServerMessage: legacyValidator,
}))

describe('gameplay v16 native validation', () => {
  it('does not fabricate or validate a rules-v7 compatibility frame', async () => {
    const { validateGameplayV16ServerMessage } = await import('./validation')
    expect(validateGameplayV16ServerMessage({ type: 'authenticated', protocolVersion: 16 }))
      .toEqual({
        valid: true,
        value: { type: 'authenticated', protocolVersion: 16 },
      })
    expect(legacyValidator).not.toHaveBeenCalled()
  })
})
