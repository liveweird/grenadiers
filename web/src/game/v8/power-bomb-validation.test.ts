import { describe, expect, it } from 'vitest'
import type { BombView } from './schema'
import { snapshot } from './testSupport'
import { validateGameplayV8ServerMessage } from './validation'

function powerMessage() {
  const message = snapshot(2)
  message.room.settings.enabledItems = ['powerBomb']
  const actor = message.state.actors[0]
  if (actor.type !== 'visible') throw new Error('expected visible actor')
  actor.inventory.weapon = 'powerBomb'
  message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'powerBomb' }]
  const bomb = {
    id: 3, cell: { column: 3, row: 1 }, owner: actor.id, range: 2, placedAt: 1,
    trigger: { type: 'timed' as const, remainingTicks: 179, paused: false },
    blast: 'power' as const,
  }
  message.state.bombs = [bomb]
  actor.liveBombCount = 1
  message.events = [
    { index: 0, tick: 2, kind: { type: 'pickupCollected' as const, id: actor.id,
      seat: actor.seat, cell: { column: 2, row: 2 }, kind: 'powerBomb' as const } },
    { index: 1, tick: 2, kind: { type: 'bombPlaced' as const, bomb } },
    { index: 2, tick: 2, kind: { type: 'bombDetonated' as const, bomb } },
  ]
  return { message, actor }
}

describe('Power Bomb v8 semantics', () => {
  it('accepts enabled inventory, pickup, live bomb, and placement/detonation events', () => {
    const { message } = powerMessage()
    expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('rejects every Power representation when disabled', () => {
    for (const source of ['weapon', 'pickup', 'live', 'placed', 'detonated'] as const) {
      const { message, actor } = powerMessage()
      message.room.settings.enabledItems = []
      message.state.pickups = source === 'pickup' ? message.state.pickups : []
      actor.inventory.weapon = source === 'weapon' ? 'powerBomb' : null
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
  })

  it('keeps reserved triggers and blast combinations closed', () => {
    const triggers: BombView['trigger'][] = [
      { type: 'timed', remainingTicks: 179, paused: true },
      { type: 'remote' },
      { type: 'contact' },
    ]
    const blasts: BombView['blast'][] = ['standard', 'piercing', 'power']
    for (const trigger of triggers) {
      for (const blast of blasts) {
        const { message } = powerMessage()
        message.state.bombs[0] = { ...message.state.bombs[0], trigger, blast }
        expect(validateGameplayV8ServerMessage(message).valid, `${trigger.type}/${blast}`).toBe(false)
      }
    }
  })
})
