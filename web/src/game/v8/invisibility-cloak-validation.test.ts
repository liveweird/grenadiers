import { describe, expect, it } from 'vitest'
import { snapshot, validateRecipient } from './testSupport'
import { validateGameplayV8ServerMessage } from './validation'

function cloakedSnapshot() {
  const message = snapshot(2)
  message.room.settings.enabledItems = ['invisibilityCloak']
  const own = message.state.actors[0]
  const opponent = message.state.actors[1]
  if (own.type !== 'visible' || opponent.type !== 'visible') throw new Error('expected visible actors')
  opponent.effects = [{ kind: 'invisibilityCloak', remainingTicks: 600 }]
  message.state.actors[1] = { type: 'hidden', id: opponent.id, seat: opponent.seat, alive: true }
  message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'invisibilityCloak' }]
  return message
}

describe('Invisibility Cloak recipient validation', () => {
  it('accepts an exact living hidden opponent while preserving the visible recipient', () => {
    const message = cloakedSnapshot()
    expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
    expect(validateRecipient(message, 'one')).toBe(true)
  })

  it('rejects hidden self, both hidden, dead hidden, disabled hidden, and leaked fields', () => {
    const cases = [
      () => {
        const message = cloakedSnapshot()
        const opponent = snapshot(2).state.actors[1]
        message.state.actors[1] = opponent
        message.state.actors[0] = { type: 'hidden', id: 1, seat: 'one', alive: true }
        return message
      },
      () => {
        const message = cloakedSnapshot()
        message.state.actors[0] = { type: 'hidden', id: 1, seat: 'one', alive: true }
        return message
      },
      () => {
        const message = cloakedSnapshot()
        message.state.actors[1] = { type: 'hidden', id: 2, seat: 'two', alive: false }
        return message
      },
      () => {
        const message = cloakedSnapshot()
        message.room.settings.enabledItems = []
        message.state.pickups = []
        return message
      },
      () => {
        const message = cloakedSnapshot()
        message.state.actors = [message.state.actors[1]]
        return message
      },
      () => {
        const message = cloakedSnapshot()
        message.state.actors[1] = {
          type: 'hidden', id: 2, seat: 'two', alive: true, position: { x: 1, y: 1 },
        } as never
        return message
      },
    ]
    for (const make of cases) expect(validateRecipient(make(), 'one')).toBe(false)
  })

  it('requires a cloaked opponent to use the hidden variant but keeps a known dead actor visible', () => {
    const leaked = snapshot(2)
    leaked.room.settings.enabledItems = ['invisibilityCloak']
    const opponent = leaked.state.actors[1]
    if (opponent.type !== 'visible') throw new Error('expected visible actor')
    opponent.effects = [{ kind: 'invisibilityCloak', remainingTicks: 1 }]
    expect(validateGameplayV8ServerMessage(leaked).valid).toBe(true)
    expect(validateRecipient(leaked, 'one')).toBe(false)

    opponent.effects = []
    opponent.alive = false
    expect(validateRecipient(leaked, 'one')).toBe(true)
  })

  it('accepts event-only cloak history after the current actor is visible', () => {
    const message = snapshot(3)
    message.room.settings.enabledItems = ['invisibilityCloak']
    message.events = [{
      tick: 2, index: 0, kind: {
        type: 'pickupCollected', id: 2, seat: 'two', cell: { column: 3, row: 1 },
        kind: 'invisibilityCloak',
      },
    }]
    expect(validateRecipient(message, 'one')).toBe(true)
  })
})
