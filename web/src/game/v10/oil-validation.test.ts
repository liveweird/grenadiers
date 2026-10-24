import { describe, expect, it } from 'vitest'
import { snapshot as currentSnapshot } from '../client/testSupport'
import { validateGameplayV10ServerMessage } from './validation'
import type { Snapshot } from './schema'

function archivedSnapshot(): Snapshot {
  const message = structuredClone(currentSnapshot(2)) as unknown as Record<string, unknown>
  delete (message.state as Record<string, unknown>).oilBarrels
  message.room = structuredClone(message.room)
  const room = message.room as { availableItems: string[] }
  room.availableItems = room.availableItems.filter((item) => item !== 'oilBarrel')
  return message as unknown as Snapshot
}

describe('gameplay v10 Oil rejection', () => {
  it('keeps the pinned Bracelet catalog and independently rejects each future public fact', () => {
    expect(validateGameplayV10ServerMessage(archivedSnapshot())).toMatchObject({ valid: true })
    const mutations = [
      (message: Snapshot) => { message.room.availableItems.push('oilBarrel') },
      (message: Snapshot) => { message.room.settings.enabledItems.push('oilBarrel') },
      (message: Snapshot) => { message.state.pickups = [{ cell: { column: 2, row: 1 }, kind: 'oilBarrel' }] },
      (message: Snapshot) => { message.events = [{ tick: 2, index: 0, kind: {
        type: 'pickupCollected', id: 1, seat: 'one', cell: { column: 2, row: 1 }, kind: 'oilBarrel',
      } }] },
    ]
    for (const mutate of mutations) {
      const message = archivedSnapshot()
      mutate(message)
      expect(validateGameplayV10ServerMessage(message)).toMatchObject({ valid: false })
    }
  })

  it('rejects materialized Oil and every private phase or grace fact', () => {
    for (const field of ['oilBarrels', 'oilSites', 'pendingOilSites', 'unlitOilGrace'] as const) {
      const message = structuredClone(archivedSnapshot()) as unknown as Record<string, unknown>
      if (field === 'unlitOilGrace') {
        const actors = (message.state as { actors: Array<Record<string, unknown>> }).actors
        actors[0][field] = [{ column: 3, row: 1 }]
      } else {
        const state = message.state as Record<string, unknown>
        state[field] = [{
          cell: { column: 3, row: 1 }, state: 'unlit',
        }]
      }
      expect(validateGameplayV10ServerMessage(message)).toMatchObject({ valid: false, source: 'schema' })
    }
  })
})
