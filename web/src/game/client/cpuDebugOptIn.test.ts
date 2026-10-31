import { describe, expect, it } from 'vitest'
import { cpuDebugOptedIn } from './cpuDebugOptIn'

describe('CPU debug URL opt-in', () => {
  it('requires the exact cpuDebug=1 query value', () => {
    expect(cpuDebugOptedIn('?cpuDebug=1')).toBe(true)
    expect(cpuDebugOptedIn('?mode=solo&cpuDebug=1')).toBe(true)
    expect(cpuDebugOptedIn('')).toBe(false)
    expect(cpuDebugOptedIn('?cpuDebug=0')).toBe(false)
    expect(cpuDebugOptedIn('?cpuDebug=true')).toBe(false)
  })
})
