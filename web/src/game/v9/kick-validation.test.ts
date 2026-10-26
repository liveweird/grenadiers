import { describe, expect, it } from 'vitest'
import { snapshot } from '../validation/archived-v11-fixtures'
import { validateGameplayV9ServerMessage } from './validation'
import type { Snapshot } from './schema'

function archivedSnapshot(): Snapshot {
  const archived = structuredClone(snapshot(1)) as unknown as Snapshot & { state: { oilBarrels?: unknown } }
  archived.room.availableItems = archived.room.availableItems
    .filter((item) => item !== 'kick' && item !== 'punch' && item !== 'bracelet'
      && item !== 'wallPass' && item !== 'oilBarrel' && item !== 'teleporter')
  for (const actor of archived.state.actors) delete (actor as unknown as { lastTeleportTick?: number }).lastTeleportTick
  delete archived.state.oilBarrels
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
    expect(validateGameplayV9ServerMessage(archivedSnapshot())).toMatchObject({ valid: true })

    const roomCatalog = archivedSnapshot()
    roomCatalog.room.availableItems = ['punch', ...roomCatalog.room.availableItems]
    expect(validateGameplayV9ServerMessage(roomCatalog)).toMatchObject({ valid: false })

    const settings = archivedSnapshot()
    settings.room.settings.enabledItems = ['punch']
    expect(validateGameplayV9ServerMessage(settings)).toMatchObject({ valid: false })

    for (const field of ['equipment', 'pickup', 'event'] as const) {
      const message = archivedSnapshot()
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
      expect(validateGameplayV9ServerMessage(message)).toEqual({
        valid: false, source: 'semantics', error: 'unsupportedProfile',
      })
    }
  })
})

describe('gameplay v9 Bracelet rejection', () => {
  it('rejects Bracelet in each archived public position from a valid baseline', () => {
    expect(validateGameplayV9ServerMessage(archivedSnapshot())).toMatchObject({ valid: true })
    const mutations = [
      (message: Snapshot) => { message.room.availableItems = ['bracelet', ...message.room.availableItems] },
      (message: Snapshot) => { message.room.settings.enabledItems = ['bracelet'] },
      (message: Snapshot) => {
        const actor = message.state.actors[0]
        if (actor.type === 'visible') actor.inventory.equipment = 'bracelet'
      },
      (message: Snapshot) => { message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'bracelet' }] },
      (message: Snapshot) => { message.events = [{ tick: 1, index: 0, kind: {
        type: 'pickupCollected', id: 1, seat: 'one', cell: { column: 2, row: 2 }, kind: 'bracelet',
      } }] },
    ]
    for (const mutate of mutations) {
      const message = archivedSnapshot()
      mutate(message)
      expect(validateGameplayV9ServerMessage(message)).toMatchObject({
        valid: false, source: 'semantics',
      })
    }
    const privateHistory = structuredClone(archivedSnapshot()) as unknown as Record<string, unknown>
    const actors = (privateHistory.state as { actors: Array<Record<string, unknown>> }).actors
    actors[0].lastSuccessfulShoveTick = 1
    expect(validateGameplayV9ServerMessage(privateHistory)).toMatchObject({ valid: false, source: 'schema' })
  })
})
