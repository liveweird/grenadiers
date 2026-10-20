import { describe, expect, it } from 'vitest'
import { snapshot } from './testSupport'
import { preservesTeleportHistory } from './teleportHistory'

function markers(one: number | null, two: number | null) {
  const state = snapshot(1).state
  for (const [index, marker] of [one, two].entries()) {
    const actor = state.actors[index]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.lastTeleportTick = marker
  }
  return state
}

describe('teleport history', () => {
  it('allows initialization, equality, and forward progress', () => {
    const prior = snapshot(1)
    prior.state = markers(null, null)
    expect(preservesTeleportHistory(prior, snapshot(2).room, markers(2, 2))).toBe(true)
    prior.state = markers(2, 2)
    expect(preservesTeleportHistory(prior, snapshot(2).room, markers(2, 2))).toBe(true)
    expect(preservesTeleportHistory(prior, snapshot(2).room, markers(3, 3))).toBe(true)
  })

  it('rejects clearing or lowering a visible actor marker', () => {
    const prior = snapshot(1)
    prior.state = markers(3, 3)
    expect(preservesTeleportHistory(prior, snapshot(2).room, markers(null, null))).toBe(false)
    expect(preservesTeleportHistory(prior, snapshot(2).room, markers(2, 2))).toBe(false)
  })

  it('initializes a newly revealed actor without exposing hidden history', () => {
    const previous = markers(3, 3)
    previous.actors[1] = { type: 'hidden', id: 2, seat: 'two', alive: true }
    const prior = snapshot(1)
    prior.state = previous
    expect(preservesTeleportHistory(prior, snapshot(2).room, markers(4, 4))).toBe(true)
  })
})
