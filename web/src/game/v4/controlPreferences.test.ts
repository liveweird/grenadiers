import { describe, expect, it, vi } from 'vitest'
import { defaultKeyboardBindings, freezeBindings } from '../arena/bindings'
import { CONTROLS_STORAGE_KEY, loadControlBindings, saveControlBindings } from './controlPreferences'

const valid = JSON.stringify({
  version: 1,
  bindings: { north: ['KeyI'], east: ['KeyL'], south: ['KeyK'], west: ['KeyJ'], bomb: ['Enter'] },
})

describe('keyboard control preferences', () => {
  it('loads and freezes a valid bounded preference', () => {
    const bindings = loadControlBindings({ getItem: () => valid })
    expect(bindings).toEqual({ north: ['KeyI'], east: ['KeyL'], south: ['KeyK'], west: ['KeyJ'], bomb: ['Enter'] })
    expect(Object.isFrozen(bindings)).toBe(true)
    expect(Object.isFrozen(bindings.north)).toBe(true)
  })

  it.each([
    '', '{', JSON.stringify({ version: 2, bindings: {} }),
    JSON.stringify({ version: 1, bindings: { north: [], east: ['KeyL'], south: ['KeyK'], west: ['KeyJ'], bomb: ['Enter'] } }),
    JSON.stringify({ version: 1, bindings: { north: ['Tab'], east: ['KeyL'], south: ['KeyK'], west: ['KeyJ'], bomb: ['Enter'] } }),
    JSON.stringify({ version: 1, bindings: { north: ['KeyW', 'KeyW'], east: ['KeyL'], south: ['KeyK'], west: ['KeyJ'], bomb: ['Enter'] } }),
    JSON.stringify({ version: 1, bindings: { north: ['KeyI'], east: ['KeyI'], south: ['KeyK'], west: ['KeyJ'], bomb: ['Enter'] } }),
    JSON.stringify({ version: 1, bindings: { north: ['KeyI'], east: ['KeyL'], south: ['KeyK'], west: ['KeyJ'], bomb: ['Enter'], extra: ['KeyQ'] } }),
    ' '.repeat(2_049),
  ])('falls back for malformed, duplicate, unsupported, or oversized storage', (raw) => {
    expect(loadControlBindings({ getItem: () => raw })).toBe(defaultKeyboardBindings)
  })

  it('falls back when storage is unavailable and ignores write failures', () => {
    expect(loadControlBindings({ getItem: () => { throw new Error('blocked') } })).toBe(defaultKeyboardBindings)
    expect(() => saveControlBindings(defaultKeyboardBindings, {
      setItem: () => { throw new Error('quota') },
    })).not.toThrow()
  })

  it('stores only the versioned control map', () => {
    const setItem = vi.fn()
    const bindings = freezeBindings({
      north: ['KeyI'], east: ['KeyL'], south: ['KeyK'], west: ['KeyJ'], bomb: ['Enter'],
    })
    saveControlBindings(bindings, { setItem })
    expect(setItem).toHaveBeenCalledWith(CONTROLS_STORAGE_KEY, valid)
  })
})
