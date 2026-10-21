import { describe, expect, it } from 'vitest'
import { snapshot } from '../client/testSupport'
import { validateGameplayV9ServerMessage } from './validation'
import type { Snapshot } from './schema'

function archivedSnapshot(): Snapshot {
  const archived = structuredClone(snapshot(1)) as unknown as Snapshot
  archived.room.availableItems = archived.room.availableItems
    .filter((item) => item !== 'kick' && item !== 'punch' && item !== 'wallPass' && item !== 'teleporter')
  for (const actor of archived.state.actors) delete (actor as unknown as { lastTeleportTick?: number }).lastTeleportTick
  return archived
}

describe('gameplay v9 Kick rejection', () => {
  it('rejects Kick in every archived public capability-bearing position', () => {
    const roomCatalog = archivedSnapshot()
    roomCatalog.room.availableItems = ['kick', ...roomCatalog.room.availableItems]
    expect(validateGameplayV9ServerMessage(roomCatalog)).toMatchObject({ valid: false })

    for (const field of ['equipment', 'pickup', 'event'] as const) {
      const message = archivedSnapshot()
      if (field === 'equipment') {
        const actor = message.state.actors[0]
        if (actor.type !== 'visible') throw new Error('expected visible actor')
        actor.inventory.equipment = 'kick'
      } else if (field === 'pickup') {
        message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'kick' }]
      } else {
        message.events = [{ tick: 1, index: 0, kind: {
          type: 'pickupCollected', id: 1, seat: 'one',
          cell: { column: 2, row: 2 }, kind: 'kick',
        } }]
      }
      expect(validateGameplayV9ServerMessage(message)).toEqual({
        valid: false, source: 'semantics', error: 'unsupportedProfile',
      })
    }
  })
})

describe('gameplay v9 Punch rejection', () => {
  it('rejects Punch in every archived public capability-bearing position', () => {
    const roomCatalog = archivedSnapshot()
    roomCatalog.room.availableItems = ['punch', ...roomCatalog.room.availableItems]
    expect(validateGameplayV9ServerMessage(roomCatalog)).toMatchObject({ valid: false })

    for (const field of ['equipment', 'pickup', 'event'] as const) {
      const message = archivedSnapshot()
      message.room.settings.enabledItems = ['punch']
      if (field === 'equipment') {
        const actor = message.state.actors[0]
        if (actor.type !== 'visible') throw new Error('expected visible actor')
        actor.inventory.equipment = 'punch'
      } else if (field === 'pickup') {
        message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'punch' }]
      } else {
        message.events = [{ tick: 1, index: 0, kind: {
          type: 'pickupCollected', id: 1, seat: 'one',
          cell: { column: 2, row: 2 }, kind: 'punch',
        } }]
      }
      expect(validateGameplayV9ServerMessage(message)).toMatchObject({ valid: false })
    }
  })
})
