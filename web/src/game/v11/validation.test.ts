import { describe, expect, it } from 'vitest'
import { checkedCatalogMaskRanges, executeCatalogMaskRange } from '../../test/catalogMasks'
import { snapshot } from '../validation/archived-v11-fixtures'
import { validateGameplayV11ServerMessage } from './validation'

const supported = [
  'kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch',
  'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame',
  'longFuse', 'reverseControls', 'invisibleBombs', 'oilBarrel', 'teleporter',
] as const
const supportedRanges = checkedCatalogMaskRanges({
  protocol: 'gameplay v11', catalog: supported, expectedCount: 22, expectedOrder: supported,
})

describe('gameplay v11 catalog and Oil validation', () => {
  it('pins twenty-two unique settings entries and complete 1024-mask ranges', () => {
    expect(supported).toHaveLength(22)
    expect(new Set(supported)).toHaveLength(22)
    expect(2 ** supported.length).toBe(4_194_304)
    expect(supportedRanges).toHaveLength(4_096)
    expect(supportedRanges[0]).toEqual({ start: 0, end: 1_024 })
    expect(supportedRanges.at(-1)).toEqual({ start: 4_193_280, end: 4_194_304 })
  })

  it.each(supportedRanges)('accepts every supported enabled-item mask in [$start, $end)', (range) => {
    executeCatalogMaskRange('gameplay v11', supported, range, (_mask, enabledItems) => {
      const message = snapshot(2)
      message.room.settings.enabledItems = [...enabledItems]
      expect(validateGameplayV11ServerMessage(message)).toEqual({ valid: true, value: message })
    })
  })

  it('accepts sorted materialized Oil states only when Oil is enabled', () => {
    const message = snapshot(2)
    message.room.settings.enabledItems = ['oilBarrel']
    message.state.oilBarrels = [
      { cell: { column: 3, row: 1 }, state: 'unlit' },
      { cell: { column: 5, row: 1 }, state: 'burning' },
    ]
    expect(validateGameplayV11ServerMessage(message)).toEqual({ valid: true, value: message })
    message.room.settings.enabledItems = []
    expect(validateGameplayV11ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidOilBarrels',
    })
  })

  it('rejects missing, malformed, duplicate, unsorted, and oversized Oil arrays', () => {
    const valid = snapshot(2)
    valid.room.settings.enabledItems = ['oilBarrel']
    const missing = structuredClone(valid) as unknown as Record<string, unknown>
    delete (missing.state as Record<string, unknown>).oilBarrels
    expect(validateGameplayV11ServerMessage(missing)).toMatchObject({ valid: false, source: 'schema' })

    for (const oilBarrels of [
      [{ cell: { column: 3, row: 1 }, state: 'wet' }],
      [{ cell: { column: 3, row: 1 }, state: 'burning', expiresAt: 600 }],
      [{ cell: { column: 3, row: 1 }, state: 'unlit' },
        { cell: { column: 3, row: 1 }, state: 'burning' }],
      [{ cell: { column: 5, row: 1 }, state: 'unlit' },
        { cell: { column: 3, row: 1 }, state: 'unlit' }],
      Array.from({ length: 9 }, (_, index) => ({
        cell: { column: 1 + index, row: 1 }, state: 'unlit',
      })),
    ]) {
      const message = structuredClone(valid) as unknown as Record<string, unknown>
      const state = message.state as Record<string, unknown>
      state.oilBarrels = oilBarrels
      expect(validateGameplayV11ServerMessage(message)).toMatchObject({ valid: false })
    }
  })

  it('rejects public Oil conflicts, collection history, and private facts', () => {
    const base = snapshot(2)
    base.room.settings.enabledItems = ['oilBarrel']
    base.state.oilBarrels = [{ cell: { column: 3, row: 1 }, state: 'unlit' }]
    const conflicted = [
      (message: typeof base) => { message.state.flames = [{ cell: { column: 3, row: 1 }, expiresAt: 3 }] },
      (message: typeof base) => { message.state.arena.blocks = [{ column: 3, row: 1 }] },
      (message: typeof base) => { message.state.bombs = [{
        id: 3, cell: { column: 3, row: 1 }, owner: 1, range: 1, placedAt: 1,
        trigger: { type: 'timed', remainingTicks: 2, paused: false }, blast: 'standard',
      }] },
      (message: typeof base) => { message.state.pickups = [{ cell: { column: 2, row: 1 }, kind: 'oilBarrel' }] },
      (message: typeof base) => { message.events = [{ tick: 2, index: 0, kind: {
        type: 'pickupCollected', id: 1, seat: 'one', cell: { column: 2, row: 1 }, kind: 'oilBarrel',
      } }] },
    ]
    for (const conflict of conflicted) {
      const message = structuredClone(base)
      conflict(message)
      expect(validateGameplayV11ServerMessage(message)).toMatchObject({ valid: false })
    }
    const leaked = structuredClone(base) as unknown as Record<string, unknown>
    const leakedState = leaked.state as Record<string, unknown>
    leakedState.oilSites = [{ cell: { column: 4, row: 1 }, phase: 'pending' }]
    expect(validateGameplayV11ServerMessage(leaked)).toMatchObject({ valid: false, source: 'schema' })
  })

  it('keeps hidden actors exact while Oil remains equally public', () => {
    const message = snapshot(2)
    message.room.settings.enabledItems = ['invisibilityCloak', 'oilBarrel']
    message.state.actors[1] = { type: 'hidden', id: 2, seat: 'two', alive: true }
    message.state.oilBarrels = [{ cell: { column: 3, row: 1 }, state: 'burning' }]
    expect(validateGameplayV11ServerMessage(message)).toEqual({ valid: true, value: message })
    const leaked = structuredClone(message) as unknown as Record<string, unknown>
    const actors = (leaked.state as { actors: Array<Record<string, unknown>> }).actors
    actors[1].position = { x: 3072, y: 1024 }
    expect(validateGameplayV11ServerMessage(leaked)).toMatchObject({ valid: false, source: 'schema' })
  })
})
