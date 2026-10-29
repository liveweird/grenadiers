import { describe, expect, it } from 'vitest'
import corpus from '../../../../api/fixtures/gameplay-v16-server-conformance.json'
import type { Snapshot } from './schema'
import { validateGameplayV16ServerMessage } from './validation'

function populated(): Snapshot {
  const value = corpus.cases.find(({ caseId }) => caseId === 'populated11x9Lairs-one')
  if (!value) throw new Error('missing populated lair fixture')
  const frame = JSON.parse(value.frame) as Snapshot
  if (frame.type !== 'snapshot') throw new Error('expected snapshot fixture')
  return frame
}

describe('gameplay v16 lair evidence', () => {
  it('requires each canonical lair cell to remain a hard wall', () => {
    const frame = populated()
    const { column, row } = frame.state.lairs[0]!.cell
    frame.state.arena.terrain[row * frame.state.arena.width + column] = 'floor'
    expect(validateGameplayV16ServerMessage(frame)).toMatchObject({ valid: false })
  })

  it('requires a live visible stopwatch holder when visibility proves the mob freeze state', () => {
    const active = populated()
    active.room.settings.enabledItems.push('stopwatch')
    const holder = active.state.actors[0]!
    const opponent = active.state.actors[1]!
    if (holder.type !== 'visible' || opponent.type !== 'visible') throw new Error('expected visible actors')
    holder.effects.push({ kind: 'stopwatch', remainingTicks: 1 })
    opponent.voluntaryActionsFrozen = true
    for (const mob of active.state.mobs) mob.frozen = true
    expect(validateGameplayV16ServerMessage(active)).toMatchObject({ valid: true })

    holder.alive = false
    opponent.voluntaryActionsFrozen = false
    expect(validateGameplayV16ServerMessage(active)).toMatchObject({ valid: false })
  })
})
