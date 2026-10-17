import { act, cleanup, fireEvent, render, screen } from '@testing-library/react'
import { StrictMode } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { GameArena } from './GameArena'
import type { ArenaRenderer } from './renderer'
import type { GameArenaProps } from './types'

const hooks = vi.hoisted(() => ({ create: vi.fn() }))
vi.mock('./renderer', () => ({ createArenaRenderer: hooks.create }))

function instance(): ArenaRenderer {
  return { update: vi.fn(), destroy: vi.fn(), setReducedMotion: vi.fn() }
}
function props(): GameArenaProps {
  return {
    label: 'Game arena', waitingText: 'Waiting for the round', failureText: 'Graphics unavailable',
    focusHint: 'Click to play', instructionsId: 'controls-help',
    snapshot: gameSnapshot(), youSeat: 'one', enabled: true,
    onDirection: vi.fn(), onBomb: vi.fn(),
  }
}
async function imports(): Promise<void> {
  await act(async () => { await vi.dynamicImportSettled() })
}

let media: { matches: boolean; addEventListener: ReturnType<typeof vi.fn>; removeEventListener: ReturnType<typeof vi.fn> }
beforeEach(() => {
  vi.useFakeTimers()
  hooks.create.mockReset().mockImplementation(async () => instance())
  media = { matches: false, addEventListener: vi.fn(), removeEventListener: vi.fn() }
  vi.stubGlobal('matchMedia', vi.fn(() => media))
})
afterEach(() => { cleanup(); vi.useRealTimers(); vi.unstubAllGlobals() })

describe('React arena ownership and keyboard scope', () => {
  it('waits for public state, owns one renderer, and forwards reduced-effects changes', async () => {
    const options = props()
    const view = render(<GameArena {...options} snapshot={null} enabled={false} />)
    expect(screen.getByText('Waiting for the round')).toBeVisible()
    expect(screen.getByRole('region')).toHaveAttribute('tabindex', '-1')
    expect(hooks.create).not.toHaveBeenCalled()
    view.rerender(<GameArena {...options} />)
    await imports()
    expect(hooks.create).toHaveBeenCalledOnce()
    const renderer = await hooks.create.mock.results[0].value as ArenaRenderer
    expect(renderer.update).toHaveBeenCalledWith(options.snapshot)
    expect(renderer.setReducedMotion).toHaveBeenCalledWith(false)
    media.matches = true
    const changed: () => void = media.addEventListener.mock.calls[0][1]
    act(changed)
    expect(renderer.setReducedMotion).toHaveBeenLastCalledWith(true)
    view.unmount()
    expect(renderer.destroy).toHaveBeenCalledOnce()
    expect(media.removeEventListener).toHaveBeenCalledWith('change', changed)
  })

  it('sends fresh scoped keys, refreshes at 100ms, and clears on blur, disable and round change', async () => {
    const options = props()
    const view = render(<GameArena {...options} />)
    await imports()
    const region = screen.getByRole('region')
    fireEvent.pointerDown(region)
    expect(region).toHaveFocus()
    fireEvent.keyDown(region, { code: 'ArrowRight' })
    fireEvent.keyDown(region, { code: 'Space' })
    fireEvent.keyDown(region, { code: 'Space', repeat: true })
    expect(options.onDirection).toHaveBeenLastCalledWith('east')
    expect(options.onBomb).toHaveBeenCalledOnce()
    act(() => { vi.advanceTimersByTime(100) })
    expect(options.onDirection).toHaveBeenCalledTimes(2)
    fireEvent.keyUp(region, { code: 'Space' })
    fireEvent(window, new Event('blur'))
    expect(options.onDirection).toHaveBeenLastCalledWith(null)
    const callsAfterBlur = vi.mocked(options.onDirection).mock.calls.length
    act(() => { vi.advanceTimersByTime(100) })
    expect(options.onDirection).toHaveBeenCalledTimes(callsAfterBlur)
    fireEvent.keyDown(region, { code: 'KeyW' })
    view.rerender(<GameArena {...options} enabled={false} />)
    expect(options.onDirection).toHaveBeenLastCalledWith(null)
    fireEvent.keyDown(region, { code: 'Space' })
    expect(options.onBomb).toHaveBeenCalledOnce()
    view.rerender(<GameArena {...options} />)
    fireEvent.keyDown(region, { code: 'KeyA' })
    const next = structuredClone(options.snapshot!)
    next.room.roundId += 1
    view.rerender(<GameArena {...options} snapshot={next} />)
    expect(options.onDirection).toHaveBeenLastCalledWith(null)
    act(() => { vi.advanceTimersByTime(100) })
    expect(options.onDirection).toHaveBeenLastCalledWith(null)
    view.unmount()
  })

  it('keeps descendant editable keys native and clears input when the document hides', async () => {
    const options = props()
    render(<GameArena {...options} />)
    await imports()
    const region = screen.getByRole('region')
    const input = document.createElement('input')
    region.appendChild(input)
    input.focus()
    const space = new KeyboardEvent('keydown', { code: 'Space', bubbles: true, cancelable: true })
    fireEvent(input, space)
    fireEvent.keyUp(input, { code: 'Space' })
    expect(space.defaultPrevented).toBe(false)
    expect(options.onBomb).not.toHaveBeenCalled()
    fireEvent.keyDown(region, { code: 'ArrowUp' })
    Object.defineProperty(document, 'hidden', { configurable: true, value: true })
    fireEvent(document, new Event('visibilitychange'))
    expect(options.onDirection).toHaveBeenLastCalledWith(null)
    Object.defineProperty(document, 'hidden', { configurable: true, value: false })
    fireEvent(document, new Event('visibilitychange'))
  })

  it('destroys a late asynchronous renderer after unmount and uses the latest snapshot', async () => {
    let resolve!: (value: ArenaRenderer) => void
    hooks.create.mockImplementation(() => new Promise<ArenaRenderer>((done) => { resolve = done }))
    const options = props()
    const view = render(<GameArena {...options} />)
    await imports()
    const newer = structuredClone(options.snapshot!)
    newer.state.runningUpdates += 3
    view.rerender(<GameArena {...options} snapshot={newer} />)
    const renderer = instance()
    await act(async () => { resolve(renderer) })
    expect(renderer.update).toHaveBeenCalledWith(newer)
    view.unmount()
    expect(renderer.destroy).toHaveBeenCalledOnce()

    const pending = render(<GameArena {...options} />)
    await imports()
    pending.unmount()
    const late = instance()
    await act(async () => { resolve(late) })
    expect(late.destroy).toHaveBeenCalledOnce()
    expect(late.update).not.toHaveBeenCalled()
  })

  it('reports initialization failure without leaving active controls', async () => {
    hooks.create.mockRejectedValue(new Error('GPU unavailable'))
    const options = props()
    render(<GameArena {...options} />)
    await imports()
    expect(screen.getByRole('alert')).toHaveTextContent('Graphics unavailable')
    const region = screen.getByRole('region')
    expect(region).toHaveAttribute('tabindex', '-1')
    fireEvent.pointerDown(region)
    fireEvent.keyDown(region, { code: 'Space' })
    expect(options.onBomb).not.toHaveBeenCalled()
  })

  it('cleans an initialization rejection after StrictMode disposal without surfacing stale errors', async () => {
    const rejectors: Array<(error: Error) => void> = []
    hooks.create.mockImplementation(() => new Promise<ArenaRenderer>((_, reject) => { rejectors.push(reject) }))
    const view = render(<StrictMode><GameArena {...props()} /></StrictMode>)
    await imports()
    view.unmount()
    await act(async () => { for (const reject of rejectors) reject(new Error('Canceled')) })
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
  })
})
