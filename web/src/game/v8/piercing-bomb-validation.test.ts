import { describe, expect, it } from 'vitest'
import { snapshot } from '../client/testSupport'
import type { BombTrigger, BombView } from './schema'
import { validateGameplayV8ServerMessage } from './validation'

function piercingMessage() {
  const message = snapshot(3)
  message.room.settings.enabledItems = ['piercingBomb']
  const actor = message.state.actors[0]
  if (actor.type !== 'visible') throw new Error('expected visible actor')
  actor.inventory.weapon = 'piercingBomb'
  message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'piercingBomb' }]
  const bomb = {
    id: 9, cell: { column: 3, row: 1 }, owner: actor.id, range: 2, placedAt: 3,
    trigger: { type: 'timed' as const, remainingTicks: 180, paused: false },
    blast: 'piercing' as const,
  }
  message.state.bombs = [bomb]
  message.events = [
    { tick: 3, index: 0, kind: { type: 'pickupCollected' as const, id: actor.id,
      seat: actor.seat, cell: { column: 2, row: 2 }, kind: 'piercingBomb' as const } },
    { tick: 3, index: 1, kind: { type: 'bombPlaced' as const, bomb } },
    { tick: 3, index: 2, kind: { type: 'bombDetonated' as const, bomb } },
  ]
  return { message, actor }
}

describe('Piercing Bomb v8 semantics', () => {
  it('accepts enabled pickup, held charge, live bomb, and bomb events', () => {
    const { message } = piercingMessage()
    expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('rejects every Piercing representation when disabled', () => {
    const sources = ['pickup', 'weapon', 'live', 'placed', 'detonated'] as const
    for (const source of sources) {
      const { message, actor } = piercingMessage()
      message.room.settings.enabledItems = []
      message.state.pickups = source === 'pickup' ? message.state.pickups : []
      actor.inventory.weapon = source === 'weapon' ? 'piercingBomb' : null
      message.state.bombs = source === 'live' ? message.state.bombs : []
      message.events = message.events.filter(({ kind }) =>
        (source === 'placed' && kind.type === 'bombPlaced')
        || (source === 'detonated' && kind.type === 'bombDetonated'))
      expect(validateGameplayV8ServerMessage(message)).toEqual({
        valid: false, source: 'semantics', error: 'unsupportedProfile',
      })
    }
  })

  it('rejects reserved weapon and blast variants', () => {
    const { message, actor } = piercingMessage()
    actor.inventory.weapon = 'powerBomb'
    expect(validateGameplayV8ServerMessage(message).valid).toBe(false)
    actor.inventory.weapon = null
    message.state.bombs[0].blast = 'power'
    expect(validateGameplayV8ServerMessage(message).valid).toBe(false)
  })

  it('rejects every unsupported trigger for live, placed, and detonated bombs', () => {
    const triggers: BombTrigger[] = [
      { type: 'remote' },
      { type: 'contact' },
      { type: 'timed', remainingTicks: 180, paused: true },
    ]
    const sources = ['live', 'placed', 'detonated'] as const
    const blasts: BombView['blast'][] = ['standard', 'piercing']
    for (const trigger of triggers) {
      for (const source of sources) {
        for (const blast of blasts) {
          const { message } = piercingMessage()
          const bomb = { ...message.state.bombs[0], trigger, blast }
          message.state.bombs = source === 'live' ? [bomb] : []
          message.events = source === 'live' ? [] : [{
            tick: 3,
            index: 0,
            kind: { type: source === 'placed' ? 'bombPlaced' : 'bombDetonated', bomb },
          }]
          expect(validateGameplayV8ServerMessage(message)).toEqual({
            valid: false, source: 'semantics', error: 'unsupportedProfile',
          })
        }
      }
    }
  })
})
