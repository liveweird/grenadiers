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

  const supported = ['heart', 'piercingBomb', 'shield', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls'] as const
  const subsets = Array.from({ length: 2 ** supported.length }, (_, mask) => [
    supported.filter((_, index) => (mask & (1 << index)) !== 0),
  ] as const)
  it.each(subsets)(
    'accepts the supported enabled-item set %j', (enabledItems) => {
      const message = snapshot(2)
      message.room.settings.enabledItems = [...enabledItems]
      expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
    },
  )

  it.each([
    ['shield only', ['shield'] as const],
    ['Heart and Shield', ['heart', 'shield'] as const],
  ])('accepts Shield state and protection events with %s enabled', (_label, enabledItems) => {
    const message = snapshot(3)
    message.room.settings.enabledItems = [...enabledItems]
    const actor = message.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.effects = [{ kind: 'shield', remainingTicks: 300 }]
    message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'shield' }]
    message.events = [
      { tick: 1, index: 0, kind: { type: 'pickupCollected', id: actor.id, seat: actor.seat,
        cell: { column: 2, row: 2 }, kind: 'shield' } },
      { tick: 1, index: 1, kind: { type: 'damagePrevented', id: actor.id, seat: actor.seat,
        source: 'shield' } },
    ]
    expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it.each([1, 600])(
    'accepts Golden Flame collection and effect at %s ticks alongside Shield', (remainingTicks) => {
      const message = snapshot(3)
      message.room.settings.enabledItems = ['shield', 'goldenFlame']
      const actor = message.state.actors[0]
      if (actor.type !== 'visible') throw new Error('expected visible actor')
      actor.effects = [
        { kind: 'shield', remainingTicks: 300 },
        { kind: 'goldenFlame', remainingTicks },
      ]
      message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'goldenFlame' }]
      message.events = [{
        tick: 1, index: 0, kind: {
          type: 'pickupCollected', id: actor.id, seat: actor.seat,
          cell: { column: 2, row: 2 }, kind: 'goldenFlame',
        },
      }]
      expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
    },
  )

  it.each([1, 600])(
    'accepts Short Flame collection and effect at %s ticks alongside independent effects', (remainingTicks) => {
      const message = snapshot(3)
      message.room.settings.enabledItems = ['shield', 'goldenFlame', 'shortFuse', 'shortFlame']
      message.room.tick = 42
      message.state.physicalTick = 42
      const actor = message.state.actors[0]
      if (actor.type !== 'visible') throw new Error('expected visible actor')
      actor.effects = [
        { kind: 'shield', remainingTicks: 300 },
        { kind: 'goldenFlame', remainingTicks: 480 },
        { kind: 'shortFuse', remainingTicks: 240 },
        { kind: 'shortFlame', remainingTicks },
      ]
      message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'shortFlame' }]
      message.events = [{
        tick: 42, index: 0, kind: {
          type: 'pickupCollected', id: actor.id, seat: actor.seat,
          cell: { column: 2, row: 2 }, kind: 'shortFlame',
        },
      }]
      expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
    },
  )

  it.each([1, 600])(
    'accepts Short Fuse collection and effect at %s ticks alongside other timed effects', (remainingTicks) => {
      const message = snapshot(3)
      message.room.settings.enabledItems = ['shield', 'goldenFlame', 'shortFuse']
      const actor = message.state.actors[0]
      if (actor.type !== 'visible') throw new Error('expected visible actor')
      actor.effects = [
        { kind: 'shield', remainingTicks: 300 },
        { kind: 'goldenFlame', remainingTicks: 600 },
        { kind: 'shortFuse', remainingTicks },
      ]
      message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'shortFuse' }]
      message.events = [{
        tick: 1, index: 0, kind: {
          type: 'pickupCollected', id: actor.id, seat: actor.seat,
          cell: { column: 2, row: 2 }, kind: 'shortFuse',
        },
      }]
      expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
    },
  )

  it.each([1, 600])(
    'accepts Long Fuse collection and independent Short Fuse timers at %s ticks', (remainingTicks) => {
      const message = snapshot(3)
      message.room.settings.enabledItems = ['shortFuse', 'longFuse']
      const actor = message.state.actors[0]
      if (actor.type !== 'visible') throw new Error('expected visible actor')
      actor.effects = [
        { kind: 'shortFuse', remainingTicks: 240 },
        { kind: 'longFuse', remainingTicks },
      ]
      message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'longFuse' }]
      message.events = [{ tick: 1, index: 0, kind: {
        type: 'pickupCollected', id: actor.id, seat: actor.seat,
        cell: { column: 2, row: 2 }, kind: 'longFuse',
      } }]
      expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
    },
  )

})

describe('gameplay v8 expanded-profile rejection', () => {
  it.each([
    ['wrong identities', (message: ReturnType<typeof snapshot>) => { message.room.rulesVersion = 4 }],
    ['unknown availability', (message: ReturnType<typeof snapshot>) => {
      message.room.availableItems = ['heart', 'stopwatch']
      message.room.settings.enabledItems = []
    }],
    ['noncanonical enabled order', (message: ReturnType<typeof snapshot>) => {
      message.room.settings.enabledItems = ['goldenFlame', 'heart']
    }],
    ['disabled Heart pickup', (message: ReturnType<typeof snapshot>) => {
      message.room.settings.enabledItems = []
      message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'heart' }]
    }],
    ['disabled Heart inventory', (message: ReturnType<typeof snapshot>) => {
      message.room.settings.enabledItems = []
      const actor = message.state.actors[0]
      if (actor.type === 'visible') actor.inventory.equipment = 'heart'
    }],
    ['disabled Shield pickup', (message: ReturnType<typeof snapshot>) => {
      message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'shield' }]
    }],
    ['disabled Golden Flame pickup', (message: ReturnType<typeof snapshot>) => {
      message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'goldenFlame' }]
    }],
    ['disabled Golden Flame collection', (message: ReturnType<typeof snapshot>) => {
      message.events = [{
        tick: 1, index: 0, kind: {
          type: 'pickupCollected', id: 1, seat: 'one',
          cell: { column: 2, row: 2 }, kind: 'goldenFlame',
        },
      }]
    }],
    ['disabled Short Fuse pickup', (message: ReturnType<typeof snapshot>) => {
      message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'shortFuse' }]
    }],
    ['disabled Short Fuse collection', (message: ReturnType<typeof snapshot>) => {
      message.events = [{
        tick: 1, index: 0, kind: {
          type: 'pickupCollected', id: 1, seat: 'one',
          cell: { column: 2, row: 2 }, kind: 'shortFuse',
        },
      }]
    }],
    ['disabled Short Flame pickup', (message: ReturnType<typeof snapshot>) => {
      message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'shortFlame' }]
    }],
    ['disabled Short Flame collection', (message: ReturnType<typeof snapshot>) => {
      message.events = [{
        tick: 1, index: 0, kind: {
          type: 'pickupCollected', id: 1, seat: 'one',
          cell: { column: 2, row: 2 }, kind: 'shortFlame',
        },
      }]
    }],
    ['future equipment', (message: ReturnType<typeof snapshot>) => {
      const actor = message.state.actors[0]
      if (actor.type === 'visible') actor.inventory.equipment = 'kick'
    }],
    ['unsupported inventory', (message: ReturnType<typeof snapshot>) => {
      const actor = message.state.actors[0]
      if (actor.type === 'visible') actor.inventory.weapon = 'remoteBomb'
    }],
    ['future effect', (message: ReturnType<typeof snapshot>) => {
      const actor = message.state.actors[0]
      if (actor.type === 'visible') actor.effects = [{ kind: 'stopwatch', remainingTicks: 1 }]
    }],
    ['Shield effect while disabled', (message: ReturnType<typeof snapshot>) => {
      const actor = message.state.actors[0]
      if (actor.type === 'visible') actor.effects = [{ kind: 'shield', remainingTicks: 1 }]
    }],
    ['Golden Flame effect while disabled', (message: ReturnType<typeof snapshot>) => {
      const actor = message.state.actors[0]
      if (actor.type === 'visible') actor.effects = [{ kind: 'goldenFlame', remainingTicks: 1 }]
    }],
    ['Short Fuse effect while disabled', (message: ReturnType<typeof snapshot>) => {
      const actor = message.state.actors[0]
      if (actor.type === 'visible') actor.effects = [{ kind: 'shortFuse', remainingTicks: 1 }]
    }],
    ['Short Flame effect while disabled', (message: ReturnType<typeof snapshot>) => {
      const actor = message.state.actors[0]
      if (actor.type === 'visible') actor.effects = [{ kind: 'shortFlame', remainingTicks: 1 }]
    }],
    ['Long Fuse effect while disabled', (message: ReturnType<typeof snapshot>) => {
      const actor = message.state.actors[0]
      if (actor.type === 'visible') actor.effects = [{ kind: 'longFuse', remainingTicks: 1 }]
    }],
    ['disabled Heart grace', (message: ReturnType<typeof snapshot>) => {
      message.room.settings.enabledItems = []
      const actor = message.state.actors[0]
      if (actor.type === 'visible') actor.damageGraceRemainingTicks = 1
    }],
    ['Shield protection while disabled', (message: ReturnType<typeof snapshot>) => {
      message.events = [{
        tick: 1, index: 0, kind: { type: 'damagePrevented', id: 1, seat: 'one', source: 'shield' },
      }]
    }],
    ['future pickup', (message: ReturnType<typeof snapshot>) => {
      message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'stopwatch' }]
    }],
    ['hidden actor', (message: ReturnType<typeof snapshot>) => {
      message.state.actors[1] = { type: 'hidden', id: 2, seat: 'two', alive: true }
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

describe('gameplay v8 effect rejection', () => {
  it.each([0, 301])('rejects Shield effect duration %s during structural validation', (remainingTicks) => {
    const message = snapshot(4)
    message.room.settings.enabledItems = ['heart', 'shield']
    const actor = message.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.effects = [{ kind: 'shield', remainingTicks }]
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({ valid: false, source: 'schema' })
  })
  it.each([0, 601])('rejects Golden Flame effect duration %s during structural validation', (remainingTicks) => {
    const message = snapshot(4)
    message.room.settings.enabledItems = ['goldenFlame']
    const actor = message.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.effects = [{ kind: 'goldenFlame', remainingTicks }]
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({ valid: false, source: 'schema' })
  })

  it('rejects duplicate Golden Flame effects semantically', () => {
    const message = snapshot(4)
    message.room.settings.enabledItems = ['goldenFlame']
    const actor = message.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.effects = [
      { kind: 'goldenFlame', remainingTicks: 600 },
      { kind: 'goldenFlame', remainingTicks: 1 },
    ]
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'duplicateEffect',
    })
  })

  it.each([0, 601])('rejects Short Fuse effect duration %s during structural validation', (remainingTicks) => {
    const message = snapshot(4)
    message.room.settings.enabledItems = ['shortFuse']
    const actor = message.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.effects = [{ kind: 'shortFuse', remainingTicks }]
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({ valid: false, source: 'schema' })
  })

  it('rejects duplicate Short Fuse effects semantically', () => {
    const message = snapshot(4)
    message.room.settings.enabledItems = ['shortFuse']
    const actor = message.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.effects = [
      { kind: 'shortFuse', remainingTicks: 600 },
      { kind: 'shortFuse', remainingTicks: 1 },
    ]
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'duplicateEffect',
    })
  })

  it.each([0, 601])('rejects Short Flame effect duration %s during structural validation', (remainingTicks) => {
    const message = snapshot(4)
    message.room.settings.enabledItems = ['shortFlame']
    const actor = message.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.effects = [{ kind: 'shortFlame', remainingTicks }]
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({ valid: false, source: 'schema' })
  })

  it('rejects duplicate Short Flame effects semantically', () => {
    const message = snapshot(4)
    message.room.settings.enabledItems = ['shortFlame']
    const actor = message.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.effects = [
      { kind: 'shortFlame', remainingTicks: 600 },
      { kind: 'shortFlame', remainingTicks: 1 },
    ]
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'duplicateEffect',
    })
  })

  it.each([0, 601])('rejects Long Fuse effect duration %s during structural validation', (remainingTicks) => {
    const message = snapshot(4)
    message.room.settings.enabledItems = ['longFuse']
    const actor = message.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.effects = [{ kind: 'longFuse', remainingTicks }]
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({ valid: false, source: 'schema' })
  })

  it('rejects duplicate Long Fuse effects semantically', () => {
    const message = snapshot(4)
    message.room.settings.enabledItems = ['longFuse']
    const actor = message.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.effects = [
      { kind: 'longFuse', remainingTicks: 600 },
      { kind: 'longFuse', remainingTicks: 1 },
    ]
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'duplicateEffect',
    })
  })

})

it('rejects a disabled Long Fuse pickup', () => {
  const message = snapshot(3)
  message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'longFuse' }]
  expect(validateGameplayV8ServerMessage(message)).toMatchObject({
    valid: false, source: 'semantics', error: 'unsupportedProfile',
  })
})
