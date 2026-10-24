import { describe, expect, it } from 'vitest'
import { snapshot } from './testSupport'
import { validateGameplayV8ServerMessage } from './validation'

describe('gameplay v8 Punch rejection', () => {
  it('rejects Punch in every archived public capability-bearing position', () => {
    const mutations = [
      (message: ReturnType<typeof snapshot>) => { message.room.availableItems.push('punch') },
      (message: ReturnType<typeof snapshot>) => { message.room.settings.enabledItems.push('punch') },
      (message: ReturnType<typeof snapshot>) => {
        const actor = message.state.actors[0]
        if (actor.type === 'visible') actor.inventory.equipment = 'punch'
      },
      (message: ReturnType<typeof snapshot>) => {
        message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'punch' }]
      },
      (message: ReturnType<typeof snapshot>) => {
        message.events = [{ tick: 1, index: 0, kind: {
          type: 'pickupCollected', id: 1, seat: 'one',
          cell: { column: 2, row: 2 }, kind: 'punch',
        } }]
      },
    ]
    for (const mutate of mutations) {
      const message = snapshot(1)
      mutate(message)
      expect(validateGameplayV8ServerMessage(message)).toMatchObject({
        valid: false, source: 'semantics',
      })
    }
  })
})

describe('gameplay v8 Bracelet rejection', () => {
  it('rejects Bracelet in each archived public position from a valid baseline', () => {
    expect(validateGameplayV8ServerMessage(snapshot(1))).toMatchObject({ valid: true })
    const mutations = [
      (message: ReturnType<typeof snapshot>) => { message.room.availableItems.push('bracelet') },
      (message: ReturnType<typeof snapshot>) => { message.room.settings.enabledItems.push('bracelet') },
      (message: ReturnType<typeof snapshot>) => {
        const actor = message.state.actors[0]
        if (actor.type === 'visible') actor.inventory.equipment = 'bracelet'
      },
      (message: ReturnType<typeof snapshot>) => {
        message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'bracelet' }]
      },
      (message: ReturnType<typeof snapshot>) => { message.events = [{ tick: 1, index: 0, kind: {
        type: 'pickupCollected', id: 1, seat: 'one', cell: { column: 2, row: 2 }, kind: 'bracelet',
      } }] },
    ]
    for (const mutate of mutations) {
      const message = snapshot(1)
      mutate(message)
      expect(validateGameplayV8ServerMessage(message)).toMatchObject({
        valid: false, source: 'semantics',
      })
    }
    const privateHistory = structuredClone(snapshot(1)) as unknown as Record<string, unknown>
    const actors = (privateHistory.state as { actors: Array<Record<string, unknown>> }).actors
    actors[0].lastSuccessfulShoveTick = 1
    expect(validateGameplayV8ServerMessage(privateHistory)).toMatchObject({ valid: false, source: 'schema' })
  })
})
