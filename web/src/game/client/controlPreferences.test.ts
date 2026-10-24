import { describe, expect, it, vi } from 'vitest'
import { defaultKeyboardBindings, freezeBindings } from '../arena/bindings'
import {
  CONTROLS_STORAGE_KEY, LEGACY_CONTROLS_STORAGE_KEY, loadControlBindings, saveControlBindings,
} from './controlPreferences'

const bindingsV2 = {
  north: ['KeyI'], east: ['KeyL'], south: ['KeyK'], west: ['KeyJ'],
  bomb: ['Enter'], equipment: ['KeyE'],
}
const valid = JSON.stringify({ version: 2, bindings: bindingsV2 })
const legacy = (bindings: Record<string, string[]>) => JSON.stringify({ version: 1, bindings })
const legacyBindings = {
  north: ['ArrowUp', 'KeyW'], east: ['ArrowRight', 'KeyD'], south: ['ArrowDown', 'KeyS'],
  west: ['ArrowLeft', 'KeyA'], bomb: ['Space'],
}
const storage = (values: Record<string, string | null>) => ({
  getItem: (key: string) => values[key] ?? null,
})

describe('keyboard control preferences', () => {
  it('prefers and freezes a valid six-action v2 preference', () => {
    const bindings = loadControlBindings(storage({
      [CONTROLS_STORAGE_KEY]: valid,
      [LEGACY_CONTROLS_STORAGE_KEY]: legacy(legacyBindings),
    }))
    expect(bindings).toEqual(bindingsV2)
    expect(Object.isFrozen(bindings)).toBe(true)
    expect(Object.isFrozen(bindings.north)).toBe(true)
  })

  it('preserves every v1 binding and assigns E when it is free', () => {
    expect(loadControlBindings(storage({
      [LEGACY_CONTROLS_STORAGE_KEY]: legacy(legacyBindings),
    }))).toEqual({ ...legacyBindings, equipment: ['KeyE'] })
  })

  it('uses the first free migration key when E is already bound', () => {
    const occupiedE = { ...legacyBindings, east: ['ArrowRight', 'KeyE'] }
    expect(loadControlBindings(storage({
      [LEGACY_CONTROLS_STORAGE_KEY]: legacy(occupiedE),
    }))).toEqual({ ...occupiedE, equipment: ['KeyF'] })
  })

  it('falls back from invalid v2 to an exact valid v1 migration', () => {
    expect(loadControlBindings(storage({
      [CONTROLS_STORAGE_KEY]: JSON.stringify({ version: 2, bindings: { ...bindingsV2, extra: ['KeyQ'] } }),
      [LEGACY_CONTROLS_STORAGE_KEY]: legacy(legacyBindings),
    }))).toEqual({ ...legacyBindings, equipment: ['KeyE'] })
  })

  it.each([
    '', '{', JSON.stringify({ version: 1, bindings: {} }),
    JSON.stringify({ version: 2, bindings: { ...bindingsV2, north: [] } }),
    JSON.stringify({ version: 2, bindings: { ...bindingsV2, north: ['Tab'] } }),
    JSON.stringify({ version: 2, bindings: { ...bindingsV2, north: ['KeyI', 'KeyI'] } }),
    JSON.stringify({ version: 2, bindings: { ...bindingsV2, east: ['KeyI'] } }),
    JSON.stringify({ version: 2, bindings: { ...bindingsV2, extra: ['KeyQ'] } }),
    ' '.repeat(2_049),
  ])('uses defaults when both stored versions are invalid', (raw) => {
    expect(loadControlBindings(storage({
      [CONTROLS_STORAGE_KEY]: raw,
      [LEGACY_CONTROLS_STORAGE_KEY]: '{',
    }))).toBe(defaultKeyboardBindings)
  })

  it('falls back when storage is unavailable and ignores write failures', () => {
    expect(loadControlBindings({ getItem: () => { throw new Error('blocked') } })).toBe(defaultKeyboardBindings)
    expect(() => saveControlBindings(defaultKeyboardBindings, {
      setItem: () => { throw new Error('quota') },
    })).not.toThrow()
  })

  it('stores only the versioned six-action map under the v2 key', () => {
    const setItem = vi.fn()
    saveControlBindings(freezeBindings(bindingsV2), { setItem })
    expect(setItem).toHaveBeenCalledWith(CONTROLS_STORAGE_KEY, valid)
  })
})
