import { describe, expect, it } from 'vitest'
import { snapshot } from '../client/testSupport'
import { validateGameplayV18ServerMessage } from './validation'

function frame() {
  const value = snapshot(10)
  value.state.spikePits = [
    { cell: { column: 4, row: 4 }, phase: 'idle', remainingTicks: 0 },
    { cell: { column: 10, row: 4 }, phase: 'warning', remainingTicks: 36 },
    { cell: { column: 4, row: 8 }, phase: 'raised', remainingTicks: 180 },
    { cell: { column: 10, row: 8 }, phase: 'idle', remainingTicks: 0 },
  ]
  return value
}

describe('live gameplay v18 spike-pit semantics', () => {
  it('accepts canonical public phases and an enabled empty layout after blocks are removed', () => {
    const value = frame()
    expect(validateGameplayV18ServerMessage(value)).toMatchObject({ valid: true })
    value.state.spikePits = []
    value.state.arena.blocks = []
    expect(validateGameplayV18ServerMessage(value)).toMatchObject({ valid: true })
  })

  it.each([
    ['order', (value: ReturnType<typeof frame>) => value.state.spikePits.reverse()],
    ['symmetry', (value: ReturnType<typeof frame>) => { value.state.spikePits.pop() }],
    ['warning timer', (value: ReturnType<typeof frame>) => { value.state.spikePits[1]!.remainingTicks = 37 }],
    ['raised timer', (value: ReturnType<typeof frame>) => { value.state.spikePits[2]!.remainingTicks = 181 }],
    ['visible block', (value: ReturnType<typeof frame>) => value.state.arena.blocks.push({ column: 4, row: 4 })],
    ['visible oil', (value: ReturnType<typeof frame>) => value.state.oilBarrels.push({
      cell: { column: 4, row: 4 }, state: 'burning',
    })],
  ])('rejects invalid public pit facts: %s', (_label, mutate) => {
    const value = frame()
    mutate(value)
    expect(validateGameplayV18ServerMessage(value)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidSpikePitState',
    })
  })

  it('rejects an idle pit carrying a timer at the strict schema boundary', () => {
    const value = frame()
    value.state.spikePits[0]!.remainingTicks = 1
    expect(validateGameplayV18ServerMessage(value)).toMatchObject({ valid: false, source: 'schema' })
  })

  it('rejects private activator data at the schema boundary', () => {
    const value = frame() as unknown as { state: { spikePits: Array<Record<string, unknown>> } }
    value.state.spikePits[0]!.activatorId = 1
    expect(validateGameplayV18ServerMessage(value)).toMatchObject({ valid: false, source: 'schema' })
  })
})
