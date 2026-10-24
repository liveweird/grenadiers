import { describe, expect, it } from 'vitest'
import { snapshot } from './testSupport'
import { validateGameplayV8ServerMessage } from './validation'

describe('gameplay v8 Oil rejection', () => {
  it('starts from a valid neutral archive and rejects each future public Oil position', () => {
    expect(validateGameplayV8ServerMessage(snapshot(1))).toMatchObject({ valid: true })
    const mutations = [
      (message: Record<string, unknown>) => {
        const room = message.room as { availableItems: string[] }
        room.availableItems.push('oilBarrel')
      },
      (message: Record<string, unknown>) => {
        const room = message.room as { settings: { enabledItems: string[] } }
        room.settings.enabledItems.push('oilBarrel')
      },
      (message: Record<string, unknown>) => {
        const state = message.state as Record<string, unknown>
        state.pickups = [{ cell: { column: 2, row: 1 }, kind: 'oilBarrel' }]
      },
      (message: Record<string, unknown>) => {
        message.events = [{ tick: 1, index: 0, kind: {
          type: 'pickupCollected', id: 1, seat: 'one',
          cell: { column: 2, row: 1 }, kind: 'oilBarrel',
        } }]
      },
    ]
    for (const mutate of mutations) {
      const message = structuredClone(snapshot(1)) as unknown as Record<string, unknown>
      mutate(message)
      expect(validateGameplayV8ServerMessage(message)).toMatchObject({ valid: false })
    }
  })

  it('rejects materialized Oil and private lifecycle facts as unknown archive fields', () => {
    for (const field of ['oilBarrels', 'oilSites', 'pendingOilSites', 'unlitOilGrace']) {
      const message = structuredClone(snapshot(1)) as unknown as Record<string, unknown>
      const state = message.state as Record<string, unknown>
      if (field === 'unlitOilGrace') {
        const actors = state.actors as Array<Record<string, unknown>>
        actors[0][field] = [{ column: 3, row: 1 }]
      } else state[field] = [{ cell: { column: 3, row: 1 }, state: 'unlit' }]
      expect(validateGameplayV8ServerMessage(message)).toMatchObject({ valid: false, source: 'schema' })
    }
  })
})
