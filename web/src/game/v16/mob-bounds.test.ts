import { describe, expect, it } from 'vitest'
import corpus from '../../../../api/fixtures/gameplay-v16-server-conformance.json'
import type { Snapshot } from './schema'
import { validateGameplayV16ServerMessage } from './validation'

const populated = corpus.cases.find(({ caseId }) => caseId === 'populated11x9Lairs-one')

function snapshot(): Snapshot {
  if (!populated) throw new Error('missing populated 11x9 fixture')
  const frame = JSON.parse(populated.frame) as Snapshot
  if (frame.type !== 'snapshot') throw new Error('expected snapshot fixture')
  return frame
}

describe('gameplay v16 mob footprint bounds', () => {
  it.each([
    ['x minimum', 0, 4_608], ['x below footprint', 319, 4_608],
    ['x above footprint', 10_945, 4_608], ['x extent', 11_264, 4_608],
    ['y minimum', 3_584, 0], ['y below footprint', 3_584, 319],
    ['y above footprint', 3_584, 8_897], ['y extent', 3_584, 9_216],
  ] as const)('rejects 11x9 mob edge center: %s', (_label, x, y) => {
    const frame = snapshot()
    frame.state.mobs[0]!.position = { x, y }
    expect(validateGameplayV16ServerMessage(frame)).toMatchObject({ valid: false })
  })
})
