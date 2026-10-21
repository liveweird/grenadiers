import { describe, expect, it } from 'vitest'
import { snapshot } from '../client/testSupport'
import { validateGameplayV8ServerMessage } from './validation'

function remoteMessage() {
  const message = snapshot(2)
  message.room.settings.enabledItems = ['remoteBomb']
  const actor = message.state.actors[0]
  if (actor.type !== 'visible') throw new Error('expected visible actor')
  actor.inventory.weapon = 'remoteBomb'
  message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'remoteBomb' }]
  const bomb = {
    id: 3, cell: { column: 3, row: 1 }, owner: actor.id, range: 2, placedAt: 1,
    trigger: { type: 'remote' as const }, blast: 'standard' as const,
  }
  message.state.bombs = [bomb]
  actor.liveBombCount = 1
  message.events = [
    { index: 0, tick: 2, kind: { type: 'pickupCollected' as const, id: actor.id,
      seat: actor.seat, cell: { column: 2, row: 2 }, kind: 'remoteBomb' as const } },
    { index: 1, tick: 2, kind: { type: 'bombPlaced' as const, bomb } },
    { index: 2, tick: 2, kind: { type: 'bombDetonated' as const, bomb } },
  ]
  return { message, actor, bomb }
}

describe('Remote Bomb v8 semantics', () => {
  it('accepts enabled inventory, pickup, live bomb, and placement/detonation events', () => {
    const { message } = remoteMessage()
    expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('rejects every Remote representation when disabled', () => {
    for (const source of ['weapon', 'pickup', 'live', 'placed', 'detonated'] as const) {
      const { message, actor } = remoteMessage()
      message.room.settings.enabledItems = []
      message.state.pickups = source === 'pickup' ? message.state.pickups : []
      actor.inventory.weapon = source === 'weapon' ? 'remoteBomb' : null
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

  it('rejects duplicate live owners, special Remote blasts, Contact, and paused Timed bombs', () => {
    const { message, actor, bomb } = remoteMessage()
    message.state.bombs.push({ ...bomb, id: 4, cell: { column: 4, row: 1 } })
    actor.liveBombCount = 2
    expect(validateGameplayV8ServerMessage(message).valid).toBe(false)

    for (const change of [
      { trigger: { type: 'remote' as const }, blast: 'piercing' as const },
      { trigger: { type: 'remote' as const }, blast: 'power' as const },
      { trigger: { type: 'contact' as const }, blast: 'standard' as const },
      { trigger: { type: 'timed' as const, remainingTicks: 179, paused: true }, blast: 'standard' as const },
    ]) {
      const candidate = remoteMessage()
      candidate.message.state.bombs[0] = { ...candidate.bomb, ...change }
      expect(validateGameplayV8ServerMessage(candidate.message).valid).toBe(false)
    }
  })

  it('accepts event-only Remote history without a current owner record', () => {
    const { message, actor, bomb } = remoteMessage()
    actor.alive = false
    actor.inventory.weapon = null
    actor.liveBombCount = 0
    message.state.pickups = []
    message.state.bombs = []
    message.events = message.events.filter((event) => event.kind.type === 'bombDetonated')
    if (message.events[0]?.kind.type !== 'bombDetonated') throw new Error('expected detonation')
    message.events[0].kind.bomb = { ...bomb, owner: 999 }
    expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('requires every live bomb owner to exist while allowing a known dead empty owner', () => {
    const known = remoteMessage()
    known.actor.alive = false
    known.actor.inventory.weapon = null
    expect(validateGameplayV8ServerMessage(known.message)).toEqual({
      valid: true, value: known.message,
    })

    const missing = remoteMessage()
    missing.message.state.bombs[0] = { ...missing.bomb, owner: 999 }
    expect(validateGameplayV8ServerMessage(missing.message)).toEqual({
      valid: false, source: 'semantics', error: 'unsupportedProfile',
    })
  })

  it('accepts an old detonation and a newly placed Remote from one owner in one event window', () => {
    const { message, actor, bomb } = remoteMessage()
    actor.inventory.weapon = null
    actor.liveBombCount = 1
    message.state.pickups = []
    const replacement = { ...bomb, id: 4, cell: { column: 4, row: 1 }, placedAt: 2 }
    message.state.bombs = [replacement]
    message.events = [
      { index: 0, tick: 2, kind: { type: 'bombDetonated' as const, bomb } },
      { index: 1, tick: 2, kind: { type: 'bombPlaced' as const, bomb: replacement } },
    ]
    expect(validateGameplayV8ServerMessage(message)).toEqual({ valid: true, value: message })
  })
})
