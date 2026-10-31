import { describe, expect, it } from 'vitest'
import validateCpuDebugResponse from '../../validation/generated/cpu-debug.mjs'

function validResponse(): Record<string, unknown> {
  return {
    protocolVersion: 1, roomId: 'b'.repeat(32), roundId: 1, tick: 20, revision: 20, youSeat: 'one',
    cpus: [{
      seat: 'two', status: 'available', observedTick: 20, objective: 'attack',
      target: { type: 'opponent', seat: 'one' }, approach: { column: 2, row: 3 },
      phase: 'approach', switchReason: 'initial',
    }],
  }
}

describe('CPU debug response schema', () => {
  it('accepts the bounded protocol response', () => {
    expect(validateCpuDebugResponse(validResponse())).toBe(true)
  })

  it.each([
    ['unknown objective enum', (value: Record<string, unknown>) => {
      (value.cpus as Array<Record<string, unknown>>)[0].objective = 'wander'
    }],
    ['extra available-row field', (value: Record<string, unknown>) => {
      (value.cpus as Array<Record<string, unknown>>)[0].path = []
    }],
    ['extra unavailable-row field', (value: Record<string, unknown>) => {
      value.cpus = [{ seat: 'two', status: 'unavailable', objective: 'wait' }]
    }],
    ['more than three rows', (value: Record<string, unknown>) => {
      const row = (value.cpus as unknown[])[0]
      value.cpus = [row, row, row, row]
    }],
    ['unknown top-level field', (value: Record<string, unknown>) => { value.rawIntent = 'private' }],
  ])('rejects %s', (_name, mutate) => {
    const value = validResponse()
    mutate(value)
    expect(validateCpuDebugResponse(value)).toBe(false)
  })
})
