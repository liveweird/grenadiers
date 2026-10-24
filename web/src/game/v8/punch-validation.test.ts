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
