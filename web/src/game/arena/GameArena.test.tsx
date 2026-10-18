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
  return { update: vi.fn(), suspend: vi.fn(), destroy: vi.fn(), setReducedMotion: vi.fn() }
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
  it('lazily reuses one renderer across waiting state and forwards reduced-effects changes', async () => {
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
    view.rerender(<GameArena {...options} snapshot={null} enabled={false} />)
    expect(renderer.suspend).toHaveBeenCalledOnce()
    const newer = structuredClone(options.snapshot!)
    newer.state.runningUpdates += 1
    view.rerender(<GameArena {...options} snapshot={newer} />)
    expect(hooks.create).toHaveBeenCalledOnce()
    expect(renderer.update).toHaveBeenLastCalledWith(newer)
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
    view.rerender(<GameArena {...options} snapshot={null} />)
    expect(options.onDirection).toHaveBeenLastCalledWith(null)
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

})

describe('React arena asynchronous renderer ownership', () => {

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

  it('uses a pending renderer for the latest state across an A-null-B transition', async () => {
    let resolve!: (value: ArenaRenderer) => void
    hooks.create.mockImplementation(() => new Promise<ArenaRenderer>((done) => { resolve = done }))
    const options = props()
    const view = render(<GameArena {...options} />)
    await imports()
    view.rerender(<GameArena {...options} snapshot={null} />)
    const latest = structuredClone(options.snapshot!)
    latest.state.runningUpdates += 7
    view.rerender(<GameArena {...options} snapshot={latest} />)
    const renderer = instance()
    await act(async () => { resolve(renderer) })
    expect(hooks.create).toHaveBeenCalledOnce()
    expect(renderer.update).toHaveBeenCalledOnce()
    expect(renderer.update).toHaveBeenCalledWith(latest)
    expect(renderer.suspend).not.toHaveBeenCalled()
    view.unmount()
  })

  it('suspends a renderer that resolves while empty and reuses it on the next activation', async () => {
    let resolve!: (value: ArenaRenderer) => void
    hooks.create.mockImplementation(() => new Promise<ArenaRenderer>((done) => { resolve = done }))
    const options = props()
    const view = render(<GameArena {...options} />)
    await imports()
    view.rerender(<GameArena {...options} snapshot={null} />)
    const renderer = instance()
    await act(async () => { resolve(renderer) })
    expect(renderer.suspend).toHaveBeenCalledOnce()
    const next = structuredClone(options.snapshot!)
    next.state.runningUpdates += 1
    view.rerender(<GameArena {...options} snapshot={next} />)
    expect(renderer.update).toHaveBeenCalledWith(next)
    expect(hooks.create).toHaveBeenCalledOnce()
    view.unmount()
  })

})

describe('React arena initialization failure recovery', () => {

  it.each(['setReducedMotion', 'update'] as const)(
    'does not publish a renderer whose first %s fails and recovers on a later activation', async (method) => {
      const poisoned = instance()
      vi.mocked(poisoned[method]).mockImplementationOnce(() => { throw new Error('initial paint failed') })
      const recovered = instance()
      hooks.create.mockResolvedValueOnce(poisoned).mockResolvedValueOnce(recovered)
      const options = props()
      const view = render(<GameArena {...options} />)
      await imports()
      expect(poisoned.destroy).toHaveBeenCalledOnce()
      expect(screen.getByRole('alert')).toHaveTextContent('Graphics unavailable')
      const revision = structuredClone(options.snapshot!)
      revision.state.runningUpdates += 1
      view.rerender(<GameArena {...options} snapshot={revision} />)
      await imports()
      expect(hooks.create).toHaveBeenCalledOnce()
      view.rerender(<GameArena {...options} snapshot={null} />)
      view.rerender(<GameArena {...options} snapshot={revision} />)
      await imports()
      expect(hooks.create).toHaveBeenCalledTimes(2)
      expect(recovered.update).toHaveBeenCalledWith(revision)
      expect(screen.queryByRole('alert')).not.toBeInTheDocument()
      expect(screen.getByRole('region')).toHaveAttribute('tabindex', '0')
    },
  )

  it('retries B once when A first paint fails after a pending A-null-B transition', async () => {
    let resolveA!: (value: ArenaRenderer) => void
    const recovered = instance()
    hooks.create
      .mockImplementationOnce(() => new Promise<ArenaRenderer>((resolve) => { resolveA = resolve }))
      .mockResolvedValueOnce(recovered)
    const options = props()
    const view = render(<GameArena {...options} />)
    await imports()
    view.rerender(<GameArena {...options} snapshot={null} />)
    const latest = structuredClone(options.snapshot!)
    latest.state.runningUpdates += 2
    view.rerender(<GameArena {...options} snapshot={latest} />)
    const poisoned = instance()
    vi.mocked(poisoned.update).mockImplementationOnce(() => { throw new Error('A paint failed') })
    await act(async () => { resolveA(poisoned) })
    await imports()
    expect(poisoned.destroy).toHaveBeenCalledOnce()
    expect(hooks.create).toHaveBeenCalledTimes(2)
    expect(recovered.update).toHaveBeenCalledWith(latest)
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
  })

  it('contains a retained renderer resume failure until the next activation', async () => {
    let resolveA!: (value: ArenaRenderer) => void
    const recovered = instance()
    hooks.create
      .mockImplementationOnce(() => new Promise<ArenaRenderer>((resolve) => { resolveA = resolve }))
      .mockResolvedValueOnce(recovered)
    const options = props()
    const view = render(<GameArena {...options} />)
    await imports()
    view.rerender(<GameArena {...options} snapshot={null} />)
    const retained = instance()
    vi.mocked(retained.update).mockImplementationOnce(() => { throw new Error('first render failed') })
    vi.mocked(retained.destroy).mockImplementationOnce(() => { throw new Error('cleanup failed') })
    await act(async () => { resolveA(retained) })
    expect(retained.suspend).toHaveBeenCalledOnce()
    const stateB = structuredClone(options.snapshot!)
    stateB.state.runningUpdates += 2
    expect(() => view.rerender(<GameArena {...options} snapshot={stateB} />)).not.toThrow()
    expect(retained.destroy).toHaveBeenCalledOnce()
    expect(screen.getByRole('alert')).toHaveTextContent('Graphics unavailable')
    expect(screen.getByRole('region')).toHaveAttribute('tabindex', '-1')
    const revisionB = structuredClone(stateB)
    revisionB.state.runningUpdates += 1
    view.rerender(<GameArena {...options} snapshot={revisionB} />)
    await imports()
    expect(hooks.create).toHaveBeenCalledOnce()
    view.rerender(<GameArena {...options} snapshot={null} />)
    view.rerender(<GameArena {...options} snapshot={revisionB} />)
    await imports()
    expect(hooks.create).toHaveBeenCalledTimes(2)
    expect(recovered.update).toHaveBeenCalledWith(revisionB)
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
  })

  it('contains direct suspend failure and replaces the renderer on reactivation', async () => {
    const broken = instance()
    vi.mocked(broken.suspend).mockImplementationOnce(() => { throw new Error('suspend failed') })
    const recovered = instance()
    hooks.create.mockResolvedValueOnce(broken).mockResolvedValueOnce(recovered)
    const options = props()
    const view = render(<GameArena {...options} />)
    await imports()
    expect(() => view.rerender(<GameArena {...options} snapshot={null} />)).not.toThrow()
    expect(broken.destroy).toHaveBeenCalledOnce()
    expect(screen.getByRole('region')).toHaveAttribute('tabindex', '-1')
    view.rerender(<GameArena {...options} />)
    await imports()
    expect(hooks.create).toHaveBeenCalledTimes(2)
    expect(recovered.update).toHaveBeenCalledWith(options.snapshot)
  })

  it('does not retry failed revisions and recovers once on a later activation', async () => {
    const recovered = instance()
    hooks.create.mockRejectedValueOnce(new Error('GPU unavailable')).mockResolvedValueOnce(recovered)
    const options = props()
    const view = render(<GameArena {...options} />)
    await imports()
    expect(screen.getByRole('alert')).toHaveTextContent('Graphics unavailable')
    const region = screen.getByRole('region')
    expect(region).toHaveAttribute('tabindex', '-1')
    fireEvent.pointerDown(region)
    fireEvent.keyDown(region, { code: 'Space' })
    expect(options.onBomb).not.toHaveBeenCalled()
    const revision = structuredClone(options.snapshot!)
    revision.state.runningUpdates += 1
    view.rerender(<GameArena {...options} snapshot={revision} />)
    await imports()
    expect(hooks.create).toHaveBeenCalledOnce()
    view.rerender(<GameArena {...options} snapshot={null} />)
    view.rerender(<GameArena {...options} snapshot={revision} />)
    await imports()
    expect(hooks.create).toHaveBeenCalledTimes(2)
    expect(recovered.update).toHaveBeenCalledWith(revision)
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
    expect(region).toHaveAttribute('tabindex', '0')
  })

  it('starts B once when pending A rejects after a null transition', async () => {
    let rejectA!: (error: Error) => void
    let resolveB!: (value: ArenaRenderer) => void
    hooks.create
      .mockImplementationOnce(() => new Promise<ArenaRenderer>((_, reject) => { rejectA = reject }))
      .mockImplementationOnce(() => new Promise<ArenaRenderer>((resolve) => { resolveB = resolve }))
    const options = props()
    const view = render(<GameArena {...options} />)
    await imports()
    view.rerender(<GameArena {...options} snapshot={null} />)
    const latest = structuredClone(options.snapshot!)
    latest.state.runningUpdates += 1
    view.rerender(<GameArena {...options} snapshot={latest} />)
    await act(async () => { rejectA(new Error('A failed')) })
    expect(hooks.create).toHaveBeenCalledTimes(2)
    const renderer = instance()
    await act(async () => { resolveB(renderer) })
    expect(renderer.update).toHaveBeenCalledWith(latest)
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
  })

  it('ignores a disposed owner rejection after a replacement owner succeeds', async () => {
    let rejectOld!: (error: Error) => void
    const replacement = instance()
    hooks.create
      .mockImplementationOnce(() => new Promise<ArenaRenderer>((_, reject) => { rejectOld = reject }))
      .mockResolvedValueOnce(replacement)
    const options = props()
    const old = render(<GameArena {...options} />)
    await imports()
    old.unmount()
    const current = render(<GameArena {...options} />)
    await imports()
    expect(replacement.update).toHaveBeenCalledWith(options.snapshot)
    await act(async () => { rejectOld(new Error('old owner failed')) })
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
    expect(screen.getByRole('region')).toHaveAttribute('tabindex', '0')
    current.unmount()
  })

  it('skips stale creation after StrictMode disposal without surfacing errors', async () => {
    vi.useRealTimers()
    const rejectors: Array<(error: Error) => void> = []
    hooks.create.mockImplementation(() => new Promise<ArenaRenderer>((_, reject) => { rejectors.push(reject) }))
    const view = render(<StrictMode><GameArena {...props()} /></StrictMode>)
    await imports()
    await act(async () => { await Promise.resolve() })
    expect(media.addEventListener).toHaveBeenCalledTimes(2)
    expect(hooks.create).not.toHaveBeenCalled()
    view.unmount()
    await act(async () => { for (const reject of rejectors) reject(new Error('Canceled')) })
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
  })
})
