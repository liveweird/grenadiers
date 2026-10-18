import { describe, expect, it } from 'vitest'
import type { BombView } from './schema'
import { snapshot } from './testSupport'
import { validateGameplayV8ServerMessage } from './validation'

function landmineMessage() {
  const message = snapshot(2)
  message.room.settings.enabledItems = ['landmine']
  const actor = message.state.actors[0]
  if (actor.type !== 'visible') throw new Error('expected visible actor')
  actor.inventory.weapon = 'landmine'
  message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'landmine' }]
  const bomb: BombView = {
    id: 3, cell: { column: 3, row: 1 }, owner: actor.id, range: 2, placedAt: 1,
    trigger: { type: 'contact' }, blast: 'standard',
  }
  message.state.bombs = [bomb]
  actor.liveBombCount = 1
  message.events = [
    { index: 0, tick: 2, kind: { type: 'pickupCollected', id: actor.id,
      seat: actor.seat, cell: { column: 2, row: 2 }, kind: 'landmine' } },
    { index: 1, tick: 2, kind: { type: 'bombPlaced', bomb } },
    { index: 2, tick: 2, kind: { type: 'bombDetonated', bomb } },
  ]
  return { message, actor, bomb }
}

describe('Landmine v8 semantics', () => {
  it('accepts enabled inventory, pickup, live Contact bomb, and event-only history', () => {
    const { message, actor } = landmineMessage()
    expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
    message.state.bombs = []
    actor.liveBombCount = 0
    actor.inventory.weapon = null
    expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('rejects each disabled representation and nonstandard or reserved trigger combinations', () => {
    for (const source of ['weapon', 'pickup', 'live', 'placed', 'detonated'] as const) {
      const { message, actor } = landmineMessage()
      message.room.settings.enabledItems = []
      message.state.pickups = source === 'pickup' ? message.state.pickups : []
      actor.inventory.weapon = source === 'weapon' ? 'landmine' : null
      message.state.bombs = source === 'live' ? message.state.bombs : []
      actor.liveBombCount = message.state.bombs.length
      message.events = message.events.filter((event) =>
        (source === 'pickup' && event.kind.type === 'pickupCollected')
        || (source === 'placed' && event.kind.type === 'bombPlaced')
        || (source === 'detonated' && event.kind.type === 'bombDetonated'))
      expect(validateGameplayV8ServerMessage(message), source).toEqual({
        valid: false, source: 'semantics', error: 'unsupportedProfile',
      })
    }
    for (const variant of [
      { trigger: { type: 'contact' as const }, blast: 'piercing' as const },
      { trigger: { type: 'contact' as const }, blast: 'power' as const },
      { trigger: { type: 'remote' as const }, blast: 'standard' as const },
    ]) {
      const { message } = landmineMessage()
      message.state.bombs[0] = { ...message.state.bombs[0], ...variant }
      expect(validateGameplayV8ServerMessage(message).valid).toBe(false)
    }
  })

  it('requires live mine owners in the full world but permits a known dead owner', () => {
    const missing = landmineMessage()
    missing.message.state.bombs[0].owner = 999
    expect(validateGameplayV8ServerMessage(missing.message).valid).toBe(false)

    const known = landmineMessage()
    known.actor.alive = false
    known.actor.inventory.weapon = null
    expect(validateGameplayV8ServerMessage(known.message)).toEqual({ valid: true, value: known.message })
  })
})
