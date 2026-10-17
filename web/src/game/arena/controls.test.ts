import { describe, expect, it, vi } from 'vitest'
import { ArenaControls } from './controls'

function setup(enabled = true) {
  const onDirection = vi.fn()
  const onBomb = vi.fn()
  const controls = new ArenaControls({ enabled, onDirection, onBomb })
  controls.setFocused(true)
  return { controls, onDirection, onBomb }
}

function key(code: string, repeat = false): KeyboardEvent {
  return new KeyboardEvent('keydown', { code, repeat, cancelable: true })
}

describe('arena keyboard ownership', () => {
  it('uses fresh key recency, ignores repeats, and falls back on release', () => {
    const { controls, onDirection } = setup()
    controls.keyDown(key('ArrowUp'))
    controls.keyDown(key('ArrowRight'))
    controls.keyDown(key('ArrowUp', true))
    controls.keyUp(key('ArrowRight'))
    controls.keyUp(key('ArrowUp'))
    expect(onDirection.mock.calls.map(([direction]) => direction)).toEqual(['north', 'east', 'north', null])
  })

  it('keeps aliases independent without duplicate direction changes', () => {
    const { controls, onDirection } = setup()
    controls.keyDown(key('KeyW'))
    controls.keyDown(key('ArrowUp'))
    controls.keyDown(key('ArrowUp'))
    controls.keyUp(key('KeyW'))
    controls.refresh()
    controls.keyUp(key('ArrowUp'))
    expect(onDirection.mock.calls.map(([direction]) => direction)).toEqual(['north', 'north', null])
  })

  it('refreshes a held direction without resending an idle stop', () => {
    const { controls, onDirection } = setup()
    controls.refresh()
    expect(onDirection).not.toHaveBeenCalled()
    controls.keyDown(key('ArrowRight'))
    controls.refresh()
    controls.keyUp(key('ArrowRight'))
    controls.refresh()
    expect(onDirection.mock.calls.map(([direction]) => direction)).toEqual(['east', 'east', null])
  })

  it('accepts a fresh Space once and keeps unrelated keys native', () => {
    const { controls, onBomb } = setup()
    const space = key('Space')
    controls.keyDown(space)
    controls.keyDown(key('Space'))
    controls.keyDown(key('Space', true))
    expect(space.defaultPrevented).toBe(true)
    expect(onBomb).toHaveBeenCalledTimes(1)
    controls.keyUp(key('Space'))
    controls.keyDown(key('Space'))
    expect(onBomb).toHaveBeenCalledTimes(2)
    const unrelated = key('Enter')
    controls.keyDown(unrelated)
    controls.keyUp(unrelated)
    expect(unrelated.defaultPrevented).toBe(false)
  })

  it('clears on focus loss and never buffers disabled or unfocused keys', () => {
    const { controls, onDirection, onBomb } = setup()
    controls.keyDown(key('KeyA'))
    controls.setFocused(false)
    controls.keyDown(key('KeyD'))
    controls.keyDown(key('Space'))
    controls.keyUp(key('KeyA'))
    controls.refresh()
    expect(onDirection.mock.calls.map(([direction]) => direction)).toEqual(['west', null])
    expect(onBomb).not.toHaveBeenCalled()
    controls.setFocused(true)
    controls.refresh()
    expect(onDirection).toHaveBeenLastCalledWith(null)
    controls.keyDown(key('KeyS'))
    controls.configure({ enabled: false, onDirection, onBomb })
    controls.keyDown(key('KeyW'))
    controls.keyUp(key('KeyS'))
    controls.refresh()
    expect(onDirection).toHaveBeenLastCalledWith(null)
    controls.configure({ enabled: true, onDirection, onBomb })
    controls.refresh()
    expect(onDirection).toHaveBeenLastCalledWith(null)
    controls.clear()
  })

  it('supports browsers without a physical code and updates callbacks without clearing held keys', () => {
    const { controls, onDirection, onBomb } = setup(false)
    controls.keyDown(key('ArrowLeft'))
    controls.keyUp(key('ArrowLeft'))
    expect(onDirection).not.toHaveBeenCalled()
    const replacement = vi.fn()
    controls.configure({ enabled: true, onDirection: replacement, onBomb })
    controls.keyDown(new KeyboardEvent('keydown', { key: 'D' }))
    controls.keyUp(new KeyboardEvent('keyup', { key: 'D' }))
    controls.keyDown(new KeyboardEvent('keydown', { key: ' ' }))
    controls.keyUp(new KeyboardEvent('keyup', { key: ' ' }))
    expect(replacement.mock.calls.map(([direction]) => direction)).toEqual(['east', null])
    expect(onBomb).toHaveBeenCalledOnce()
  })
})
