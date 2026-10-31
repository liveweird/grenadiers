import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ReactElement } from 'react'
import { gameSnapshot } from '../../../test/game-snapshot'
import { createThreeArenaRenderer } from './renderer'

const hooks = vi.hoisted(() => ({
  configureError: null as Error | null,
  atlasError: null as Error | null,
  atlas: { dispose: vi.fn(), texture: { needsUpdate: false } },
  store: {
    read: vi.fn(), subscribe: vi.fn(), update: vi.fn(), clear: vi.fn(), setReducedMotion: vi.fn(),
  },
  roundCamera: {
    update: vi.fn(), apply: vi.fn(), reapply: vi.fn(), clear: vi.fn(), setReducedMotion: vi.fn(),
  },
  createRoundCamera: vi.fn(),
  advance: vi.fn(),
  setSize: vi.fn(),
  resetGlState: vi.fn(),
  setClearColor: vi.fn(),
  shadowMap: { needsUpdate: false },
  configure: vi.fn(),
  render: vi.fn(),
  unmount: vi.fn(),
  frames: new Map<number, FrameRequestCallback>(),
  nextFrame: 1,
}))

vi.mock('./atlas', () => ({ createThreeAtlas: vi.fn(async () => {
  if (hooks.atlasError) throw hooks.atlasError
  return hooks.atlas
}) }))
vi.mock('./sceneStore', () => ({ createArenaSceneStore: vi.fn(() => hooks.store) }))
vi.mock('./roundCamera', () => ({
  createRoundCameraController: hooks.createRoundCamera,
}))
vi.mock('@react-three/fiber', () => ({
  extend: vi.fn(),
  createRoot: vi.fn(() => ({
    configure: hooks.configure,
    render: hooks.render,
  })),
  unmountComponentAtNode: hooks.unmount,
}))

beforeEach(() => {
  Object.defineProperty(window, 'devicePixelRatio', { value: 1, configurable: true })
  hooks.configureError = null
  hooks.atlasError = null
  hooks.atlas.dispose.mockReset()
  hooks.atlas.texture.needsUpdate = false
  for (const method of Object.values(hooks.store)) method.mockReset()
  for (const method of Object.values(hooks.roundCamera)) method.mockReset()
  hooks.createRoundCamera.mockReset().mockReturnValue(hooks.roundCamera)
  hooks.advance.mockReset()
  hooks.setSize.mockReset()
  hooks.resetGlState.mockReset()
  hooks.setClearColor.mockReset()
  hooks.shadowMap.needsUpdate = false
  hooks.configure.mockReset().mockImplementation(async () => {
    if (hooks.configureError) throw hooks.configureError
  })
  hooks.render.mockReset().mockImplementation((element: ReactElement<{ resolve: () => void }>) => {
    element.props.resolve()
    return { getState: () => ({
      advance: hooks.advance, setSize: hooks.setSize,
      gl: {
        resetState: hooks.resetGlState,
        setClearColor: hooks.setClearColor,
        shadowMap: hooks.shadowMap,
      },
    }) }
  })
  hooks.unmount.mockReset().mockImplementation((_canvas: HTMLCanvasElement, done: () => void) => done())
  hooks.frames.clear()
  hooks.nextFrame = 1
  vi.stubGlobal('requestAnimationFrame', vi.fn((callback: FrameRequestCallback) => {
    const id = hooks.nextFrame
    hooks.nextFrame += 1
    hooks.frames.set(id, callback)
    return id
  }))
  vi.stubGlobal('cancelAnimationFrame', vi.fn((id: number) => hooks.frames.delete(id)))
  vi.stubGlobal('ResizeObserver', class {
    observe = vi.fn()
    disconnect = vi.fn()
  })
})

function runFrame(timestamp = 100): void {
  const first = hooks.frames.entries().next().value as [number, FrameRequestCallback] | undefined
  if (!first) throw new Error('No animation frame is pending')
  hooks.frames.delete(first[0])
  first[1](timestamp)
}

describe('manual Three renderer lifetime', () => {
  it('reveals only after a frame and synchronously suspends private presentation state', async () => {
    const host = document.createElement('div')
    const renderer = await createThreeArenaRenderer(host)
    const canvas = host.querySelector('canvas')
    expect(canvas).not.toBeNull()
    expect(canvas?.style.visibility).toBe('hidden')
    expect(hooks.configure).toHaveBeenCalledWith(expect.objectContaining({
      frameloop: 'never', dpr: [1, 1], shadows: 'soft',
      gl: expect.objectContaining({ antialias: true }),
    }))

    const snapshot = gameSnapshot()
    renderer.update(snapshot)
    expect(hooks.roundCamera.update).toHaveBeenCalledWith(snapshot, expect.any(Number))
    expect(hooks.roundCamera.update.mock.invocationCallOrder[0])
      .toBeLessThan(hooks.store.update.mock.invocationCallOrder[0]!)
    expect(hooks.store.update).toHaveBeenCalledWith(snapshot, expect.any(Number))
    expect(canvas?.style.visibility).toBe('hidden')
    runFrame(123)
    expect(hooks.roundCamera.apply.mock.invocationCallOrder.at(-1))
      .toBeLessThan(hooks.advance.mock.invocationCallOrder[0]!)
    expect(hooks.advance).toHaveBeenCalledWith(0.123, true)
    expect(canvas?.style.visibility).toBe('visible')

    const contextLoss = new Event('webglcontextlost', { cancelable: true })
    canvas?.dispatchEvent(contextLoss)
    expect(contextLoss.defaultPrevented).toBe(true)
    expect(canvas?.style.visibility).toBe('hidden')
    expect(hooks.frames.size).toBe(0)
    expect(hooks.roundCamera.clear).toHaveBeenCalled()
    canvas?.dispatchEvent(new Event('webglcontextrestored'))
    runFrame(234)
    expect(canvas?.style.visibility).toBe('visible')
    expect(hooks.resetGlState).toHaveBeenCalledOnce()
    expect(hooks.setClearColor).toHaveBeenCalledWith(0x172127, 1)
    expect(hooks.shadowMap.needsUpdate).toBe(true)
    expect(hooks.atlas.texture.needsUpdate).toBe(true)

    renderer.setReducedMotion(true)
    expect(hooks.roundCamera.setReducedMotion).toHaveBeenCalledWith(
      true, expect.anything(), expect.objectContaining({ width: 15, height: 13 }),
    )
    expect(hooks.store.setReducedMotion).toHaveBeenCalledWith(true)
    renderer.suspend()
    expect(canvas?.style.visibility).toBe('hidden')
    expect(hooks.store.clear).toHaveBeenCalledOnce()
    expect(hooks.frames.size).toBe(0)

    renderer.update(snapshot)
    runFrame(456)
    expect(canvas?.style.visibility).toBe('visible')
    renderer.destroy()
    renderer.destroy()
    renderer.update(snapshot)
    renderer.suspend()
    renderer.setReducedMotion(false)
    expect(host.querySelector('canvas')).toBeNull()
    expect(hooks.unmount).toHaveBeenCalledOnce()
    expect(hooks.atlas.dispose).toHaveBeenCalledOnce()
    expect(hooks.store.update).toHaveBeenCalledTimes(2)
    expect(hooks.store.setReducedMotion).toHaveBeenCalledOnce()
  })

  it('retains an asynchronous frame failure for the next authoritative update', async () => {
    const renderer = await createThreeArenaRenderer(document.createElement('div'))
    const snapshot = gameSnapshot()
    hooks.advance.mockImplementationOnce(() => { throw new Error('GPU frame failed') })
    renderer.update(snapshot)
    runFrame()
    expect(hooks.roundCamera.clear).toHaveBeenCalled()
    expect(() => renderer.update(snapshot)).toThrow('GPU frame failed')
    expect(hooks.frames.size).toBe(0)
    renderer.destroy()
  })

  it('releases atlas and canvas when Fiber initialization fails', async () => {
    hooks.configureError = new Error('WebGL unavailable')
    const host = document.createElement('div')
    await expect(createThreeArenaRenderer(host)).rejects.toThrow('WebGL unavailable')
    expect(host.querySelector('canvas')).toBeNull()
    expect(hooks.unmount).toHaveBeenCalledOnce()
    expect(hooks.atlas.dispose).toHaveBeenCalledOnce()
  })

  it('supports the no-ResizeObserver fallback and clamps a sub-one DPR', async () => {
    vi.stubGlobal('ResizeObserver', undefined)
    Object.defineProperty(window, 'devicePixelRatio', { value: 0.5, configurable: true })
    const renderer = await createThreeArenaRenderer(document.createElement('div'))
    expect(hooks.configure).toHaveBeenCalledWith(expect.objectContaining({ dpr: [1, 1] }))
    const snapshot = gameSnapshot()
    snapshot.state.arena.width = 13
    snapshot.state.arena.height = 11
    renderer.update(snapshot)
    expect(hooks.setSize).toHaveBeenCalled()
    expect(hooks.roundCamera.reapply).toHaveBeenCalled()
    renderer.destroy()
  })

  it('passes an injected random source to the per-renderer camera controller', async () => {
    const random = vi.fn(() => 0.5)
    const renderer = await createThreeArenaRenderer(document.createElement('div'), random)
    expect(hooks.createRoundCamera).toHaveBeenCalledWith(random)
    renderer.destroy()
  })

  it('does not allocate a canvas when atlas loading fails', async () => {
    hooks.atlasError = new Error('atlas decode failed')
    const host = document.createElement('div')
    await expect(createThreeArenaRenderer(host)).rejects.toThrow('atlas decode failed')
    expect(host.querySelector('canvas')).toBeNull()
    expect(hooks.configure).not.toHaveBeenCalled()
    expect(hooks.atlas.dispose).not.toHaveBeenCalled()
  })

  it('releases the atlas after a late host attachment failure', async () => {
    const host = document.createElement('div')
    vi.spyOn(host, 'appendChild').mockImplementation(() => { throw new Error('detached host') })
    await expect(createThreeArenaRenderer(host)).rejects.toThrow('detached host')
    expect(hooks.atlas.dispose).toHaveBeenCalledOnce()
    expect(hooks.unmount).not.toHaveBeenCalled()
    expect(hooks.configure).not.toHaveBeenCalled()
  })
})
