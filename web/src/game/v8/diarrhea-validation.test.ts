import { describe, expect, it } from 'vitest'
import { snapshot } from './testSupport'
import { validateGameplayV8ServerMessage } from './validation'

function diarrheaMessage(remainingTicks: number) {
  const message = snapshot(3)
  message.room.settings.enabledItems = ['diarrhea']
  const actor = message.state.actors[0]
  if (actor.type !== 'visible') throw new Error('expected visible actor')
  actor.effects = [{ kind: 'diarrhea', remainingTicks }]
  return { message, actor }
}

describe('Diarrhea v8 validation', () => {
  it.each([1, 600])('accepts collection and effect at %s ticks', (remainingTicks) => {
    const { message, actor } = diarrheaMessage(remainingTicks)
    message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'diarrhea' }]
    message.events = [{ tick: 1, index: 0, kind: {
      type: 'pickupCollected', id: actor.id, seat: actor.seat,
      cell: { column: 2, row: 2 }, kind: 'diarrhea',
    } }]
    expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it.each([0, 601])('rejects effect duration %s structurally', (remainingTicks) => {
    expect(validateGameplayV8ServerMessage(diarrheaMessage(remainingTicks).message))
      .toMatchObject({ valid: false, source: 'schema' })
  })

  it('rejects disabled, duplicate, and disabled-pickup representations', () => {
    const disabled = diarrheaMessage(1).message
    disabled.room.settings.enabledItems = []
    expect(validateGameplayV8ServerMessage(disabled)).toMatchObject({
      valid: false, source: 'semantics', error: 'unsupportedProfile',
    })
    const duplicate = diarrheaMessage(600)
    duplicate.actor.effects.push({ kind: 'diarrhea', remainingTicks: 1 })
    expect(validateGameplayV8ServerMessage(duplicate.message)).toMatchObject({
      valid: false, source: 'semantics', error: 'duplicateEffect',
    })
    const pickup = snapshot(3)
    pickup.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'diarrhea' }]
    expect(validateGameplayV8ServerMessage(pickup)).toMatchObject({
      valid: false, source: 'semantics', error: 'unsupportedProfile',
    })
  })
})
