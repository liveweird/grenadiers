import { describe, expect, it, vi } from 'vitest'
import { defaultKeyboardBindings, freezeBindings } from '../arena/bindings'
import {
  CONTROLS_STORAGE_KEY, LEGACY_CONTROLS_STORAGE_KEY, loadControlBindings, saveControlBindings,
  V2_CONTROLS_STORAGE_KEY,
} from './controlPreferences'

const bindings = {
  north: ['KeyI'], east: ['KeyL'], south: ['KeyK'], west: ['KeyJ'], bomb: ['Enter'],
}
const stored = (version: number, value: Record<string, string[]>) => JSON.stringify({ version, bindings: value })
const v2Bindings = { ...bindings, equipment: ['KeyE'] }
const v1Bindings = {
  north: ['ArrowUp', 'KeyW'], east: ['ArrowRight', 'KeyD'], south: ['ArrowDown', 'KeyS'],
  west: ['ArrowLeft', 'KeyA'], bomb: ['Space'],
}
const storage = (values: Record<string, string | null>) => ({
  getItem: (key: string) => values[key] ?? null,
})

describe('keyboard control preferences', () => {
  it('prefers and freezes a valid five-action v3 preference', () => {
    const result = loadControlBindings(storage({
      [CONTROLS_STORAGE_KEY]: stored(3, bindings),
      [V2_CONTROLS_STORAGE_KEY]: stored(2, v2Bindings),
    }))
    expect(result).toEqual(bindings)
    expect(Object.isFrozen(result)).toBe(true)
    expect(Object.isFrozen(result.north)).toBe(true)
  })

  it('migrates custom v2 keys by dropping only Equipment', () => {
    expect(loadControlBindings(storage({
      [V2_CONTROLS_STORAGE_KEY]: stored(2, v2Bindings),
    }))).toEqual(bindings)
  })

  it('keeps E when v2 assigned it to movement and frees the retired Equipment key', () => {
    const occupiedE = { ...v2Bindings, east: ['KeyE'], equipment: ['KeyF'] }
    expect(loadControlBindings(storage({
      [V2_CONTROLS_STORAGE_KEY]: stored(2, occupiedE),
    }))).toEqual({ ...bindings, east: ['KeyE'] })
  })

  it('falls through malformed v2 to an exact valid v1 migration', () => {
    expect(loadControlBindings(storage({
      [CONTROLS_STORAGE_KEY]: '{',
      [V2_CONTROLS_STORAGE_KEY]: stored(2, { ...v2Bindings, equipment: ['Tab'] }),
      [LEGACY_CONTROLS_STORAGE_KEY]: stored(1, v1Bindings),
    }))).toEqual(v1Bindings)
  })

  it.each([
    '', '{', JSON.stringify({ version: 3, bindings: {} }),
    stored(3, { ...bindings, north: [] }), stored(3, { ...bindings, north: ['Tab'] }),
    stored(3, { ...bindings, north: ['KeyI', 'KeyI'] }), stored(3, { ...bindings, east: ['KeyI'] }),
    stored(3, { ...bindings, extra: ['KeyQ'] }), ' '.repeat(2_049),
  ])('uses defaults when all stored versions are invalid', (raw) => {
    expect(loadControlBindings(storage({
      [CONTROLS_STORAGE_KEY]: raw,
      [V2_CONTROLS_STORAGE_KEY]: '{',
      [LEGACY_CONTROLS_STORAGE_KEY]: '{',
    }))).toBe(defaultKeyboardBindings)
  })

  it('falls back when storage is unavailable and ignores write failures', () => {
    expect(loadControlBindings({ getItem: () => { throw new Error('blocked') } })).toBe(defaultKeyboardBindings)
    expect(() => saveControlBindings(defaultKeyboardBindings, {
      setItem: () => { throw new Error('quota') },
    })).not.toThrow()
  })

  it('stores only the versioned five-action map under the v3 key', () => {
    const setItem = vi.fn()
    saveControlBindings(freezeBindings(bindings), { setItem })
    expect(setItem).toHaveBeenCalledWith(CONTROLS_STORAGE_KEY, stored(3, bindings))
  })
})
