import { describe, expect, it } from 'vitest'
import { checkedCatalogMaskRanges, executeCatalogMaskRange } from '../../test/catalogMasks'
import { validateServerMessage } from '../client/clientProtocol'
import { snapshot } from '../client/testSupport'
import { snapshot as v11Snapshot } from '../validation/archived-v11-fixtures'
import { validateGameplayV8ServerMessage } from '../v8/validation'
import { validateGameplayV9ServerMessage } from './validation'
import type { Snapshot as V9Snapshot } from './schema'

function archivedValue(value: unknown): V9Snapshot {
  const archived = structuredClone(value) as {
    room: { availableItems: string[]; mode?: unknown; configuration?: unknown; scores: unknown;
      settings: { environment?: unknown };
      seats: Array<{ controller?: unknown }> }
    state: { actors: Array<Record<string, unknown>>; oilBarrels?: unknown; stormCloud?: unknown;
      lairs?: unknown; mobs?: unknown; spikePits?: unknown }
  }
  delete archived.room.mode
  delete archived.room.configuration
  delete archived.room.settings.environment
  Object.assign(archived.room, { rulesVersion: 5 })
  if (Array.isArray(archived.room.scores)) {
    archived.room.scores = Object.fromEntries(archived.room.scores.map((score) => {
      const value = score as { seat: string; wins: number }
      return [value.seat, value.wins]
    }))
  }
  for (const seat of archived.room.seats) delete seat.controller
  archived.room.availableItems = archived.room.availableItems
    .filter((item) => item !== 'kick' && item !== 'punch' && item !== 'bracelet'
      && item !== 'wallPass' && item !== 'oilBarrel' && item !== 'teleporter')
  for (const actor of archived.state.actors) {
    delete actor.lastTeleportTick
    delete actor.stormSlowRemainingTicks
  }
  delete archived.state.oilBarrels
  delete archived.state.stormCloud
  delete archived.state.lairs
  delete archived.state.mobs
  delete archived.state.spikePits
  return archived as unknown as V9Snapshot
}
const archivedSnapshot = (revision: number): V9Snapshot => archivedValue(v11Snapshot(revision))

const supported = [
  'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch',
  'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot',
  'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs',
] as const
const expectedSupported = [
  'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch',
  'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot',
  'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs',
] as const
const supportedRanges = checkedCatalogMaskRanges({
  protocol: 'gameplay v9', catalog: supported, expectedCount: 16,
  expectedOrder: expectedSupported,
})

describe('gameplay v9 Stopwatch validation', () => {
  it('pins sixteen unique catalog entries in canonical order before enumerating mask ranges', () => {
    expect(supported).toHaveLength(16)
    expect(new Set(supported)).toHaveLength(16)
    expect(supported).toEqual(expectedSupported)
  })

  it.each(supportedRanges)('accepts every pinned supported enabled-item mask in [$start, $end)', (range) => {
    executeCatalogMaskRange('gameplay v9', supported, range, (_mask, enabledItems) => {
      const message = archivedSnapshot(1)
      message.room.settings.enabledItems = [...enabledItems]
      expect(validateGameplayV9ServerMessage(message)).toEqual({ valid: true, value: message })
    })
  })

  it('rejects Wall Pass in every archived capability-bearing position', () => {
    const roomCatalog = archivedSnapshot(1)
    roomCatalog.room.availableItems = ['wallPass', ...roomCatalog.room.availableItems]
    expect(validateGameplayV9ServerMessage(roomCatalog)).toMatchObject({ valid: false })

    for (const field of ['equipment', 'pickup', 'event'] as const) {
      const message = archivedSnapshot(1)
      if (field === 'equipment') {
        const actor = message.state.actors[0]
        if (actor.type !== 'visible') throw new Error('expected visible actor')
        actor.inventory.equipment = 'wallPass'
      } else if (field === 'pickup') {
        message.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'wallPass' }]
      } else {
        message.events = [{ tick: 1, index: 0, kind: {
          type: 'pickupCollected', id: 1, seat: 'one',
          cell: { column: 2, row: 2 }, kind: 'wallPass',
        } }]
      }
      expect(validateGameplayV9ServerMessage(message)).toEqual({
        valid: false, source: 'semantics', error: 'unsupportedProfile',
      })
    }
  })


  it('binds visible frozen state to the living opponent Stopwatch and keeps hidden actors exact', () => {
    const message = archivedSnapshot(1)
    expect(validateGameplayV9ServerMessage(message)).toEqual({ valid: true, value: message })
    expect(validateGameplayV8ServerMessage(message)).toMatchObject({ valid: false, source: 'schema' })

    const missing = structuredClone(message) as unknown as Record<string, unknown>
    const missingState = missing.state as { actors: Array<Record<string, unknown>> }
    delete missingState.actors[0].voluntaryActionsFrozen
    expect(validateGameplayV9ServerMessage(missing)).toMatchObject({ valid: false, source: 'schema' })

    const frozen = structuredClone(message)
    const actor = frozen.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.voluntaryActionsFrozen = true
    expect(validateGameplayV9ServerMessage(frozen)).toEqual({
      valid: false, source: 'semantics', error: 'invalidFrozenState',
    })
    frozen.room.settings.enabledItems = ['stopwatch']
    const opponent = frozen.state.actors[1]
    if (opponent.type !== 'visible') throw new Error('expected visible opponent')
    opponent.effects = [{ kind: 'stopwatch', remainingTicks: 180 }]
    expect(validateGameplayV9ServerMessage(frozen)).toEqual({ valid: true, value: frozen })
    actor.alive = false
    expect(validateGameplayV9ServerMessage(frozen)).toEqual({
      valid: false, source: 'semantics', error: 'invalidFrozenState',
    })

    const hidden = structuredClone(message) as unknown as Record<string, unknown>
    const hiddenState = hidden.state as { actors: Array<Record<string, unknown>> }
    hiddenState.actors[1] = { type: 'hidden', id: 2, seat: 'two', alive: true,
      voluntaryActionsFrozen: false }
    expect(validateGameplayV9ServerMessage(hidden)).toMatchObject({ valid: false, source: 'schema' })
  })

  it('accepts Stopwatch and paused timed bombs only with the enabled capability', () => {
    const stopwatch = archivedSnapshot(2)
    stopwatch.room.settings.enabledItems = ['stopwatch']
    expect(validateGameplayV9ServerMessage(stopwatch)).toEqual({ valid: true, value: stopwatch })

    const paused = archivedSnapshot(2)
    paused.state.bombs = [{
      id: 3, cell: { column: 1, row: 1 }, owner: 1, range: 2, placedAt: 1,
      trigger: { type: 'timed', remainingTicks: 179, paused: true }, blast: 'standard',
    }]
    const owner = paused.state.actors[0]
    if (owner.type !== 'visible') throw new Error('expected visible actor')
    owner.liveBombCount = 1
    expect(validateGameplayV9ServerMessage(paused)).toMatchObject({
      valid: false, source: 'semantics', error: 'unsupportedProfile',
    })
    paused.room.settings.enabledItems = ['stopwatch']
    expect(validateGameplayV9ServerMessage(paused)).toEqual({ valid: true, value: paused })
  })

  it.each([false, true])('rejects a live Timed bomb at zero with paused=%s', (paused) => {
    const message = archivedSnapshot(2)
    message.room.settings.enabledItems = ['stopwatch']
    message.state.bombs = [{
      id: 3, cell: { column: 1, row: 1 }, owner: 1, range: 2, placedAt: 1,
      trigger: { type: 'timed', remainingTicks: 0, paused }, blast: 'standard',
    }]
    const owner = message.state.actors[0]
    if (owner.type !== 'visible') throw new Error('expected visible actor')
    owner.liveBombCount = 1
    expect(validateGameplayV9ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'unsupportedProfile',
    })
  })

  it('keeps zero natural and positive paused chain detonation history valid', () => {
    const message = archivedSnapshot(2)
    message.room.settings.enabledItems = ['stopwatch']
    const eventBomb = (id: number, remainingTicks: number, paused: boolean) => ({
      id, cell: { column: id, row: 1 }, owner: 1, range: 2, placedAt: 1,
      trigger: { type: 'timed' as const, remainingTicks, paused }, blast: 'standard' as const,
    })
    message.events = [
      { tick: 2, index: 0, kind: { type: 'bombDetonated', bomb: eventBomb(3, 0, false) } },
      { tick: 2, index: 1, kind: { type: 'bombDetonated', bomb: eventBomb(4, 179, true) } },
    ]
    expect(validateGameplayV9ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('retains recipient-private Cloak validation with the new visible shape', () => {
    const message = snapshot(3)
    message.room.settings.enabledItems = ['invisibilityCloak']
    const opponent = message.state.actors[1]
    if (opponent.type !== 'visible') throw new Error('expected visible actor')
    message.state.actors[1] = { type: 'hidden', id: opponent.id, seat: opponent.seat, alive: true }
    expect(validateServerMessage(message, 'one')).toBe(true)

    const hiddenSelf = structuredClone(message)
    hiddenSelf.state.actors[0] = { type: 'hidden', id: 1, seat: 'one', alive: true }
    expect(validateServerMessage(hiddenSelf, 'one')).toBe(false)

    const leaked = snapshot(3)
    leaked.room.settings.enabledItems = ['invisibilityCloak']
    const leakedOpponent = leaked.state.actors[1]
    if (leakedOpponent.type !== 'visible') throw new Error('expected visible actor')
    leakedOpponent.effects = [{ kind: 'invisibilityCloak', remainingTicks: 600 }]
    expect(validateServerMessage(leaked, 'one')).toBe(false)
  })

  it('requires an afflicted recipient to receive an empty live-bomb list', () => {
    const message = snapshot(3)
    message.room.settings.enabledItems = ['invisibleBombs']
    const own = message.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.effects = [{ kind: 'invisibleBombs', remainingTicks: 600 }]
    expect(validateServerMessage(message, 'one')).toBe(true)

    own.liveBombCount = 1
    message.state.bombs = [{
      id: 3, cell: { column: 1, row: 1 }, owner: own.id, range: 2, placedAt: 1,
      trigger: { type: 'remote' }, blast: 'standard',
    }]
    message.room.settings.enabledItems = ['remoteBomb', 'invisibleBombs']
    const archived = archivedValue(message)
    expect(validateGameplayV9ServerMessage(archived)).toEqual({ valid: true, value: archived })
    expect(validateServerMessage(message, 'one')).toBe(false)

    own.effects = []
    const opponent = message.state.actors[1]
    if (opponent.type !== 'visible') throw new Error('expected visible opponent')
    opponent.effects = [{ kind: 'invisibleBombs', remainingTicks: 600 }]
    expect(validateServerMessage(message, 'one')).toBe(true)
    expect(validateServerMessage(message, 'two')).toBe(false)
  })

  it('accepts visible pre-collection history without requiring a live bomb', () => {
    const message = snapshot(4)
    message.room.settings.enabledItems = ['invisibleBombs']
    const own = message.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.effects = [{ kind: 'invisibleBombs', remainingTicks: 599 }]
    message.events = [{
      tick: 3, index: 0, kind: { type: 'bombPlaced', bomb: {
        id: 3, cell: { column: 1, row: 1 }, owner: 1, range: 2, placedAt: 3,
        trigger: { type: 'timed', remainingTicks: 180, paused: false }, blast: 'standard',
      } },
    }]
    expect(validateServerMessage(message, 'one')).toBe(true)
  })
})
