import { describe, expect, it } from 'vitest'
import { snapshot } from './testSupport'
import { validateGameplayV8ServerMessage } from './validation'

function reverseMessage(remainingTicks: number) {
  const message = snapshot(3)
  message.room.settings.enabledItems = ['reverseControls']
  const actor = message.state.actors[0]
  if (actor.type !== 'visible') throw new Error('expected visible actor')
  actor.effects = [{ kind: 'reverseControls', remainingTicks }]
  return { message, actor }
}

describe('Reverse Controls v8 validation', () => {
  it.each([1, 600])('accepts collection and effect at %s ticks', (remainingTicks) => {
    const { message, actor } = reverseMessage(remainingTicks)
    message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'reverseControls' }]
    message.events = [{ tick: 1, index: 0, kind: {
      type: 'pickupCollected', id: actor.id, seat: actor.seat,
      cell: { column: 2, row: 2 }, kind: 'reverseControls',
    } }]
    expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it.each([0, 601])('rejects effect duration %s structurally', (remainingTicks) => {
    const { message } = reverseMessage(remainingTicks)
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({ valid: false, source: 'schema' })
  })

  it('rejects an effect while disabled', () => {
    const { message } = reverseMessage(1)
    message.room.settings.enabledItems = []
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'unsupportedProfile',
    })
  })

  it('rejects duplicate effects', () => {
    const { message, actor } = reverseMessage(600)
    actor.effects.push({ kind: 'reverseControls', remainingTicks: 1 })
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'duplicateEffect',
    })
  })

  it('rejects a pickup while disabled', () => {
    const message = snapshot(3)
    message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'reverseControls' }]
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'unsupportedProfile',
    })
  })
})
