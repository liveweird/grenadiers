import { describe, expect, it } from 'vitest'
import { snapshot, state } from './testSupport'
import { gameplayV7ServerSchemaValidator, validateGameplayV7ServerMessage } from './validation'
import type { Snapshot } from './schema'

describe('gameplay v7 composed validation', () => {
  it('runs structural validation before semantic validation', () => {
    const invalid = structuredClone(snapshot) as Record<string, unknown>
    delete invalid.events
    const result = validateGameplayV7ServerMessage(invalid)
    expect(result.valid).toBe(false)
    if (!result.valid) expect(result.source).toBe('schema')
  })

  it('reports duplicate effects after the message passes AJV', () => {
    const duplicateState = structuredClone(state)
    const actor = duplicateState.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.effects = [
      { kind: 'shield', remainingTicks: 300 },
      { kind: 'shield', remainingTicks: 1 },
    ]
    const result = validateGameplayV7ServerMessage({ ...snapshot, state: duplicateState })
    expect(result).toEqual({ valid: false, source: 'semantics', error: 'duplicateEffect' })
  })

  it('keeps the slow-foot cap semantic rather than structural', () => {
    const slowState = structuredClone(state)
    const actor = slowState.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.effects = Array.from({ length: 7 }, (_, index) => ({
      kind: 'slowFoot',
      remainingTicks: index + 1,
    }))
    const message = { ...snapshot, state: slowState }

    expect(gameplayV7ServerSchemaValidator()(message)).toBe(true)
    expect(validateGameplayV7ServerMessage(message)).toEqual({
      valid: false,
      source: 'semantics',
      error: 'slowFootLimit',
    })
  })

  it('checks settings against the available items carried by that room', () => {
    const result = validateGameplayV7ServerMessage({
      ...snapshot,
      room: { ...snapshot.room, settings: { ...snapshot.room.settings, enabledItems: ['teleporter'] } },
    })
    expect(result).toEqual({ valid: false, source: 'semantics', error: 'unavailableItem' })
  })

  it('rejects Wall Pass throughout the archived room, state, and event profile', () => {
    const mutations = [
      (message: Snapshot) => { message.room.availableItems.push('wallPass') },
      (message: Snapshot) => { message.room.settings.enabledItems.push('wallPass') },
      (message: Snapshot) => {
        const actor = message.state.actors[0]
        if (actor.type === 'visible') actor.inventory.equipment = 'wallPass'
      },
      (message: Snapshot) => {
        message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'wallPass' }]
      },
      (message: Snapshot) => {
        message.events = [{ tick: 42, index: 0, kind: {
          type: 'pickupCollected', id: 1, seat: 'one',
          cell: { column: 2, row: 2 }, kind: 'wallPass',
        } }]
      },
    ]
    for (const mutate of mutations) {
      const message = structuredClone(snapshot) as unknown as Snapshot
      mutate(message)
      expect(validateGameplayV7ServerMessage(message)).toEqual({
        valid: false, source: 'semantics', error: 'unsupportedProfile',
      })
    }
  })

  it('rejects Punch throughout the archived room, state, and event profile', () => {
    const mutations = [
      (message: Snapshot) => { message.room.availableItems.push('punch') },
      (message: Snapshot) => { message.room.settings.enabledItems.push('punch') },
      (message: Snapshot) => {
        const actor = message.state.actors[0]
        if (actor.type === 'visible') actor.inventory.equipment = 'punch'
      },
      (message: Snapshot) => {
        message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'punch' }]
      },
      (message: Snapshot) => {
        message.events = [{ tick: 42, index: 0, kind: {
          type: 'pickupCollected', id: 1, seat: 'one',
          cell: { column: 2, row: 2 }, kind: 'punch',
        } }]
      },
    ]
    for (const mutate of mutations) {
      const message = structuredClone(snapshot) as unknown as Snapshot
      mutate(message)
      expect(validateGameplayV7ServerMessage(message)).toEqual({
        valid: false, source: 'semantics', error: 'unsupportedProfile',
      })
    }
  })

  it('accepts a structurally and semantically valid dormant snapshot', () => {
    expect(validateGameplayV7ServerMessage(snapshot)).toEqual({ valid: true, value: snapshot })
  })
})
