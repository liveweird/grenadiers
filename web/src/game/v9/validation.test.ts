import { describe, expect, it } from 'vitest'
import { validateServerMessage } from '../client/clientProtocol'
import { snapshot } from '../client/testSupport'
import { validateGameplayV8ServerMessage } from '../v8/validation'
import { validateGameplayV9ServerMessage } from './validation'

describe('gameplay v9 foundation validation', () => {
  const supported = [
    'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield',
    'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot',
    'shortFlame', 'longFuse', 'reverseControls',
  ] as const

  it.each(Array.from({ length: 2 ** supported.length }, (_, mask) => [
    supported.filter((_, index) => (mask & (1 << index)) !== 0),
  ] as const))('accepts the pinned supported enabled-item set %j', (enabledItems) => {
    const message = snapshot(1)
    message.room.settings.enabledItems = [...enabledItems]
    expect(validateGameplayV9ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('requires false frozen state on visible actors and keeps hidden actors exact', () => {
    const message = snapshot(1)
    expect(validateGameplayV9ServerMessage(message)).toEqual({ valid: true, value: message })
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({ valid: false, source: 'schema' })

    const missing = structuredClone(message) as unknown as Record<string, unknown>
    const missingState = missing.state as { actors: Array<Record<string, unknown>> }
    delete missingState.actors[0].voluntaryActionsFrozen
    expect(validateGameplayV9ServerMessage(missing)).toMatchObject({ valid: false, source: 'schema' })

    const frozen = structuredClone(message)
    const actor = frozen.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.voluntaryActionsFrozen = true
    expect(validateGameplayV9ServerMessage(frozen)).toEqual({
      valid: false, source: 'semantics', error: 'invalidFrozenState',
    })
    actor.alive = false
    expect(validateGameplayV9ServerMessage(frozen)).toEqual({
      valid: false, source: 'semantics', error: 'invalidFrozenState',
    })

    const hidden = structuredClone(message) as unknown as Record<string, unknown>
    const hiddenState = hidden.state as { actors: Array<Record<string, unknown>> }
    hiddenState.actors[1] = { type: 'hidden', id: 2, seat: 'two', alive: true,
      voluntaryActionsFrozen: false }
    expect(validateGameplayV9ServerMessage(hidden)).toMatchObject({ valid: false, source: 'schema' })
  })

  it('keeps Stopwatch and paused timed bombs unavailable in the foundation', () => {
    const stopwatch = snapshot(2)
    stopwatch.room.settings.enabledItems = ['stopwatch']
    expect(validateGameplayV9ServerMessage(stopwatch)).toMatchObject({
      valid: false, source: 'semantics', error: 'unavailableItem',
    })

    const paused = snapshot(2)
    paused.state.bombs = [{
      id: 3, cell: { column: 1, row: 1 }, owner: 1, range: 2, placedAt: 1,
      trigger: { type: 'timed', remainingTicks: 179, paused: true }, blast: 'standard',
    }]
    const owner = paused.state.actors[0]
    if (owner.type !== 'visible') throw new Error('expected visible actor')
    owner.liveBombCount = 1
    expect(validateGameplayV9ServerMessage(paused)).toMatchObject({
      valid: false, source: 'semantics', error: 'unsupportedProfile',
    })
  })

  it('retains recipient-private Cloak validation with the new visible shape', () => {
    const message = snapshot(3)
    message.room.settings.enabledItems = ['invisibilityCloak']
    const opponent = message.state.actors[1]
    if (opponent.type !== 'visible') throw new Error('expected visible actor')
    message.state.actors[1] = { type: 'hidden', id: opponent.id, seat: opponent.seat, alive: true }
    expect(validateServerMessage(message, 'one')).toBe(true)

    const hiddenSelf = structuredClone(message)
    hiddenSelf.state.actors[0] = { type: 'hidden', id: 1, seat: 'one', alive: true }
    expect(validateServerMessage(hiddenSelf, 'one')).toBe(false)

    const leaked = snapshot(3)
    leaked.room.settings.enabledItems = ['invisibilityCloak']
    const leakedOpponent = leaked.state.actors[1]
    if (leakedOpponent.type !== 'visible') throw new Error('expected visible actor')
    leakedOpponent.effects = [{ kind: 'invisibilityCloak', remainingTicks: 600 }]
    expect(validateServerMessage(leaked, 'one')).toBe(false)
  })
})
