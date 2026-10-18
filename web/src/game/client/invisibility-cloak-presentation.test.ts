import { describe, expect, it } from 'vitest'
import { initialGameClientState } from './clientTypes'
import { presentSnapshot } from './presentation'
import { snapshot } from './testSupport'

describe('Invisibility Cloak presentation', () => {
  it('keeps the owner timer and pickup while removing every hidden opponent detail', () => {
    const wire = snapshot(2)
    const own = wire.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.effects = [{ kind: 'invisibilityCloak', remainingTicks: 599 }]
    wire.state.actors[1] = { type: 'hidden', id: 2, seat: 'two', alive: true }
    wire.state.pickups = [{ cell: { column: 3, row: 1 }, kind: 'invisibilityCloak' }]
    const presented = presentSnapshot({
      ...initialGameClientState(), connection: 'authenticated', room: wire.room, snapshot: wire,
      youSeat: 'one', admissionBaseline: null,
    })!
    expect(presented.state.actors).toEqual([
      expect.objectContaining({ seat: 'one', invisibilityCloakRemainingTicks: 599 }),
    ])
    expect(presented.state.pickups).toEqual(wire.state.pickups)
  })
})
