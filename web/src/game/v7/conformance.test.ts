import Ajv, { type ValidateFunction } from 'ajv'
import { describe, expect, it } from 'vitest'
import gameplayV7Schema from '../../../../api/gameplay-v7.schema.json'
import type { GameClientMessage, GameServerMessage } from './schema'
import { bomb, room, roomId, snapshot, state, visibleActor } from './testSupport'

const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV7Schema)
const token = 'a'.repeat(64)

function validator<T>(definition: string): ValidateFunction<T> {
  const reference = `${gameplayV7Schema.$id}#/$defs/${definition}`
  const resolved = ajv.getSchema<T>(reference)
  if (!resolved) throw new Error(`AJV did not resolve ${reference}`)
  return resolved
}

function valid(validate: ValidateFunction, value: unknown): boolean {
  return validate(value)
}

const validateClient = validator<GameClientMessage>('GameClientMessage')
const validateServer = validator<GameServerMessage>('GameServerMessage')
const validateState = validator('GameState')
const validateActor = validator('ActorView')
const validateBomb = validator('BombView')
const validateEffect = validator('EffectView')
const validateEvents = validator('EventKind')
const validateSettings = validator('GameSettings')

describe('gameplay v7 client and admission contract', () => {
  it('fixes successful authentication to v7 while retaining bounded auth input', () => {
    expect(valid(validateClient, { type: 'authenticate', protocolVersion: 7, token })).toBe(true)
    expect(valid(validateClient, { type: 'authenticate', protocolVersion: 6, token })).toBe(true)
    expect(valid(validateServer, { type: 'authenticated', protocolVersion: 7 })).toBe(true)
    expect(valid(validateServer, { type: 'authenticated', protocolVersion: 6 })).toBe(false)
  })

  it('replaces placeBomb with both manual action choices and closes the command', () => {
    for (const action of ['bomb', 'equipment']) {
      const message = { type: 'manualAction', sequence: 1, roomId, roundId: 1, actionId: 1, action }
      expect(valid(validateClient, message), JSON.stringify(validateClient.errors)).toBe(true)
      expect(valid(validateClient, { ...message, extra: true })).toBe(false)
    }
    expect(valid(validateClient, { type: 'placeBomb', sequence: 1, roomId, roundId: 1, actionId: 1 })).toBe(false)
    expect(valid(validateClient, { type: 'manualAction', sequence: 1, roomId, roundId: 1, actionId: 1, action: 'kick' })).toBe(false)
  })

  it('renames the required nullable admitted action acknowledgement', () => {
    const admitted = {
      type: 'admitted', sequence: 1, room, youSeat: 'one', resumeToken: token, state: null, lastActionId: null,
    }
    expect(valid(validateServer, admitted), JSON.stringify(validateServer.errors)).toBe(true)
    expect(valid(validateServer, { ...admitted, lastActionId: 1 })).toBe(true)
    const missing = structuredClone(admitted) as Record<string, unknown>
    delete missing.lastActionId
    expect(valid(validateServer, missing)).toBe(false)
    expect(valid(validateServer, { ...admitted, lastBombActionId: null })).toBe(false)
  })
})

describe('gameplay v7 physical state', () => {
  it('requires the physical tick and exactly the selected actor shape', () => {
    expect(valid(validateState, state)).toBe(true)
    const missingTick = structuredClone(state) as unknown as Record<string, unknown>
    delete missingTick.physicalTick
    expect(valid(validateState, missingTick)).toBe(false)

    expect(valid(validateActor, visibleActor)).toBe(true)
    const hidden = { type: 'hidden', id: 2, seat: 'two', alive: true }
    expect(valid(validateActor, hidden)).toBe(true)
    expect(valid(validateActor, { ...hidden, position: { x: 1, y: 1 } })).toBe(false)
    expect(valid(validateActor, { ...visibleActor, inventory: { equipment: 'heart', weapon: 'remoteBomb' } })).toBe(true)
    expect(valid(validateActor, { ...visibleActor, inventory: { equipment: null, weapon: null }, damageGraceRemainingTicks: 60 })).toBe(true)
    expect(valid(validateActor, { ...visibleActor, damageGraceRemainingTicks: 61 })).toBe(false)
  })

  it('accepts every effect union branch at its timer boundary', () => {
    const maximums = {
      shield: 300,
      stopwatch: 180,
      invisibilityCloak: 600,
      goldenFlame: 600,
      shortFuse: 600,
      diarrhea: 600,
      slowFoot: 600,
      shortFlame: 600,
      longFuse: 600,
      reverseControls: 600,
      invisibleBombs: 600,
    } as const
    for (const [kind, maximum] of Object.entries(maximums)) {
      expect(valid(validateEffect, { kind, remainingTicks: 1 }), kind).toBe(true)
      expect(valid(validateEffect, { kind, remainingTicks: maximum }), kind).toBe(true)
      expect(valid(validateEffect, { kind, remainingTicks: 0 }), kind).toBe(false)
      expect(valid(validateEffect, { kind, remainingTicks: maximum + 1 }), kind).toBe(false)
    }
    expect(valid(validateEffect, { kind: 'heart', remainingTicks: 1 })).toBe(false)
  })

  it('leaves seven slow-foot entries structurally valid for semantic rejection', () => {
    const sevenSlow = Array.from({ length: 7 }, (_, index) => ({ kind: 'slowFoot', remainingTicks: index + 1 }))
    expect(valid(validateActor, { ...visibleActor, effects: sevenSlow })).toBe(true)
    expect(valid(validateActor, { ...visibleActor, effects: Array.from({ length: 17 }, () => sevenSlow[0]) })).toBe(false)
  })

  it('accepts every trigger and blast branch and removes absolute detonation time', () => {
    const triggers = [
      { type: 'timed', remainingTicks: 0, paused: true },
      { type: 'timed', remainingTicks: 600, paused: false },
      { type: 'remote' },
      { type: 'contact' },
    ]
    for (const trigger of triggers) {
      for (const blast of ['standard', 'piercing', 'power']) {
        expect(valid(validateBomb, { ...bomb, trigger, blast }), `${trigger.type}.${blast}`).toBe(true)
      }
    }
    expect(valid(validateBomb, { ...bomb, trigger: { type: 'timed', remainingTicks: 601, paused: false } })).toBe(false)
    expect(valid(validateBomb, { ...bomb, detonatesAt: 220 })).toBe(false)
  })
})

describe('gameplay v7 items and events', () => {
  it('preserves three pickup IDs and accepts all twenty-two item IDs', () => {
    const itemIds = gameplayV7Schema.$defs.ItemId.enum
    const pickupIds = gameplayV7Schema.$defs.PickupKind.enum
    expect(pickupIds.slice(0, 3)).toEqual(['bombCapacity', 'blastRange', 'movementSpeed'])
    expect(pickupIds.slice(3)).toEqual(itemIds)
    expect(itemIds).toHaveLength(22)
    for (const kind of pickupIds) {
      expect(valid(validateState, { ...state, pickups: [{ cell: { column: 0, row: 0 }, kind }] }), kind).toBe(true)
    }
    expect(valid(validateState, { ...state, pickups: [{ cell: { column: 0, row: 0 }, kind: 'unknown' }] })).toBe(false)
  })

  it('bounds and deduplicates available and enabled item arrays', () => {
    const allItems = gameplayV7Schema.$defs.ItemId.enum
    expect(valid(validateSettings, { ...room.settings, enabledItems: allItems })).toBe(true)
    expect(valid(validateSettings, { ...room.settings, enabledItems: [allItems[0], allItems[0]] })).toBe(false)
    expect(valid(validateServer, { ...snapshot, room: { ...room, availableItems: allItems } })).toBe(true)
    expect(valid(validateServer, { ...snapshot, room: { ...room, availableItems: [allItems[0], allItems[0]] } })).toBe(false)
    expect(valid(validateSettings, { ...room.settings, enabledItems: ['unknown'] })).toBe(false)
  })

  it('accepts every event union and enforces the expanded event window', () => {
    const events = [
      { type: 'bombPlaced', bomb },
      { type: 'bombDetonated', bomb },
      { type: 'actorEliminated', id: 1, seat: 'one' },
      { type: 'roundEnded', result: { type: 'winner', seat: 'one' } },
      { type: 'pickupCollected', id: 1, seat: 'one', cell: { column: 1, row: 1 }, kind: 'kick' },
      { type: 'damagePrevented', id: 1, seat: 'one', source: 'heart' },
      { type: 'damagePrevented', id: 1, seat: 'one', source: 'shield' },
    ]
    for (const event of events) expect(valid(validateEvents, event), event.type).toBe(true)
    expect(valid(validateEvents, { ...events[5], source: 'bracelet' })).toBe(false)

    const wireEvent = { tick: 42, index: 95, kind: events[0] }
    expect(valid(validateServer, { ...snapshot, events: [wireEvent] })).toBe(true)
    expect(valid(validateServer, { ...snapshot, events: [{ ...wireEvent, index: 96 }] })).toBe(false)
    expect(valid(validateServer, { ...snapshot, events: Array.from({ length: 96 }, () => wireEvent) })).toBe(true)
    expect(valid(validateServer, { ...snapshot, events: Array.from({ length: 97 }, () => wireEvent) })).toBe(false)
  })
})
