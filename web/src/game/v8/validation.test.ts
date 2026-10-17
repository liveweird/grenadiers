import { describe, expect, it } from 'vitest'
import { snapshot } from '../client/testSupport'
import { gameplayV8ServerSchemaValidator, validateGameplayV8ServerMessage } from './validation'

describe('gameplay v8 validation and active profile', () => {
  it('runs AJV before semantics', () => {
    const invalid = structuredClone(snapshot(1)) as unknown as Record<string, unknown>
    delete invalid.events
    const result = validateGameplayV8ServerMessage(invalid)
    expect(result.valid).toBe(false)
    if (!result.valid) expect(result.source).toBe('schema')
  })

  it('accepts enabled Heart state and protection events', () => {
    const message = snapshot(2)
    const actor = message.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.inventory.equipment = 'heart'
    actor.damageGraceRemainingTicks = 60
    message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'heart' }]
    message.events = [{
      tick: 1, index: 0, kind: { type: 'damagePrevented', id: actor.id, seat: actor.seat, source: 'heart' },
    }]
    expect(gameplayV8ServerSchemaValidator()(message)).toBe(true)
    expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it.each([
    ['wrong identities', (message: ReturnType<typeof snapshot>) => { message.room.rulesVersion = 4 }],
    ['unknown availability', (message: ReturnType<typeof snapshot>) => {
      message.room.availableItems = ['shield']
      message.room.settings.enabledItems = []
    }],
    ['disabled Heart pickup', (message: ReturnType<typeof snapshot>) => {
      message.room.settings.enabledItems = []
      message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'heart' }]
    }],
    ['unsupported inventory', (message: ReturnType<typeof snapshot>) => {
      const actor = message.state.actors[0]
      if (actor.type === 'visible') actor.inventory.weapon = 'remoteBomb'
    }],
    ['unsupported effect', (message: ReturnType<typeof snapshot>) => {
      const actor = message.state.actors[0]
      if (actor.type === 'visible') actor.effects = [{ kind: 'shield', remainingTicks: 1 }]
    }],
    ['disabled Heart grace', (message: ReturnType<typeof snapshot>) => {
      message.room.settings.enabledItems = []
      const actor = message.state.actors[0]
      if (actor.type === 'visible') actor.damageGraceRemainingTicks = 1
    }],
    ['unsupported protection source', (message: ReturnType<typeof snapshot>) => {
      message.events = [{
        tick: 1, index: 0, kind: { type: 'damagePrevented', id: 1, seat: 'one', source: 'shield' },
      }]
    }],
  ])('rejects %s after structural validation', (_label, mutate) => {
    const message = snapshot(3)
    mutate(message)
    expect(gameplayV8ServerSchemaValidator()(message)).toBe(true)
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'unsupportedProfile',
    })
  })
})
