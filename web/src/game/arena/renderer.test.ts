import { beforeEach, describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { createArenaRenderer } from './renderer'

const hooks = vi.hoisted(() => ({
  fail: false,
  partial: true,
  applications: [] as Array<{
    canvas: HTMLCanvasElement
    init: ReturnType<typeof vi.fn>
    destroy: ReturnType<typeof vi.fn>
    render: ReturnType<typeof vi.fn>
    renderer: { resize: ReturnType<typeof vi.fn>; resolution: number } | undefined
    ticker: {
      add: ReturnType<typeof vi.fn>
      start: ReturnType<typeof vi.fn>
      stop: ReturnType<typeof vi.fn>
    }
  }>,
  scenes: [] as Array<{
    root: object
    update: ReturnType<typeof vi.fn>
    render: ReturnType<typeof vi.fn>
    reset: ReturnType<typeof vi.fn>
    restoreContext: ReturnType<typeof vi.fn>
    setReducedMotion: ReturnType<typeof vi.fn>
    resolution: number
  }>,
}))

vi.mock('pixi.js', () => ({
  Application: class {
    canvas = document.createElement('canvas')
    stage = { addChild: vi.fn() }
    renderer: { resize: ReturnType<typeof vi.fn>; resolution: number } | undefined = {
      resize: vi.fn(), resolution: 2,
    }
    ticker = { add: vi.fn(), start: vi.fn(), stop: vi.fn() }
    render = vi.fn()
    init = vi.fn(async () => {
      if (hooks.fail) {
        if (!hooks.partial) this.renderer = undefined
        throw new Error('GPU unavailable')
      }
    })
    destroy = vi.fn(() => this.canvas.remove())
    constructor() { hooks.applications.push(this) }
  },
  Container: class {},
  Graphics: class {},
}))
vi.mock('./scene', () => ({
  ArenaScene: class {
    root = {}
    update = vi.fn()
    render = vi.fn()
    reset = vi.fn()
    restoreContext = vi.fn()
    setReducedMotion = vi.fn()
    resolution: number
    constructor(resolution: number) { this.resolution = resolution; hooks.scenes.push(this) }
  },
}))

beforeEach(() => {
  hooks.fail = false
  hooks.partial = true
  hooks.applications.length = 0
  hooks.scenes.length = 0
  Object.defineProperty(window, 'devicePixelRatio', { value: 3, configurable: true })
})

describe('Pixi application lifetime', () => {
  it('suspends and resumes one hidden canvas before releasing it on final destroy', async () => {
    const host = document.createElement('div')
    const renderer = await createArenaRenderer(host)
    const app = hooks.applications[0]
    const scene = hooks.scenes[0]
    expect(host.querySelectorAll('canvas')).toHaveLength(1)
    expect(app.canvas).toHaveAttribute('aria-hidden', 'true')
    expect(app.init).toHaveBeenCalledWith(expect.objectContaining({
      preference: 'webgl', resolution: 2, width: 720, height: 624, antialias: false,
      autoStart: false,
    }))
    expect(app.canvas.style.imageRendering).toBe('pixelated')
    expect(app.canvas.style.visibility).toBe('hidden')
    expect(scene.resolution).toBe(2)
    expect(app.renderer?.resize).not.toHaveBeenCalled()
    const snapshot = gameSnapshot()
    renderer.update(snapshot)
    expect(scene.update).toHaveBeenCalledWith(snapshot, expect.any(Number))
    expect(app.render).toHaveBeenCalledOnce()
    expect(app.render.mock.invocationCallOrder[0]).toBeLessThan(app.ticker.start.mock.invocationCallOrder[0])
    expect(app.canvas.style.visibility).toBe('visible')
    expect(app.ticker.start).toHaveBeenCalledOnce()
    renderer.setReducedMotion(true)
    const tick: () => void = app.ticker.add.mock.calls[0][0]
    tick()
    expect(scene.render).toHaveBeenCalledOnce()
    expect(scene.setReducedMotion).toHaveBeenCalledWith(true)
    app.canvas.dispatchEvent(new Event('webglcontextrestored'))
    expect(scene.restoreContext).toHaveBeenCalledOnce()
    expect(app.render).toHaveBeenCalledTimes(2)
    renderer.suspend()
    renderer.suspend()
    expect(app.canvas.style.visibility).toBe('hidden')
    expect(app.ticker.stop).toHaveBeenCalledOnce()
    expect(scene.reset).toHaveBeenCalledOnce()
    expect(app.destroy).not.toHaveBeenCalled()
    expect(host.querySelectorAll('canvas')).toHaveLength(1)
    renderer.update(snapshot)
    expect(scene.update).toHaveBeenCalledTimes(2)
    expect(app.render).toHaveBeenCalledTimes(3)
    expect(app.ticker.start).toHaveBeenCalledTimes(2)
    expect(host.querySelectorAll('canvas')).toHaveLength(1)
    app.canvas.dispatchEvent(new Event('webglcontextrestored'))
    expect(scene.restoreContext).toHaveBeenCalledTimes(2)
    expect(app.render).toHaveBeenCalledTimes(4)
    renderer.destroy()
    renderer.destroy()
    renderer.update(snapshot)
    renderer.suspend()
    renderer.setReducedMotion(false)
    expect(host.querySelectorAll('canvas')).toHaveLength(0)
    expect(app.destroy).toHaveBeenCalledExactlyOnceWith(
      { removeView: true }, { children: true, context: true },
    )
    expect(scene.update).toHaveBeenCalledTimes(2)
    expect(scene.setReducedMotion).toHaveBeenCalledOnce()
    expect(scene.reset).toHaveBeenCalledTimes(2)
    app.canvas.dispatchEvent(new Event('webglcontextrestored'))
    expect(scene.restoreContext).toHaveBeenCalledTimes(2)
  })

  it('uses a safe DPR fallback and releases partially initialized GPU resources on failure', async () => {
    Object.defineProperty(window, 'devicePixelRatio', { value: 0, configurable: true })
    const host = document.createElement('div')
    const renderer = await createArenaRenderer(host)
    expect(hooks.applications[0].init).toHaveBeenCalledWith(expect.objectContaining({ resolution: 1 }))
    expect(hooks.applications[0].renderer?.resize).not.toHaveBeenCalled()
    renderer.destroy()
    hooks.fail = true
    await expect(createArenaRenderer(host)).rejects.toThrow('GPU unavailable')
    expect(hooks.applications[1].destroy).toHaveBeenCalledOnce()
    hooks.partial = false
    await expect(createArenaRenderer(host)).rejects.toThrow('GPU unavailable')
    expect(hooks.applications[2].destroy).not.toHaveBeenCalled()
    expect(host.querySelectorAll('canvas')).toHaveLength(0)
  })
})
