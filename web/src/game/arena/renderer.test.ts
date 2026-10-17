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
    renderer: { resize: ReturnType<typeof vi.fn> } | undefined
    ticker: { add: ReturnType<typeof vi.fn> }
  }>,
  scenes: [] as Array<{
    root: { scale: { set: ReturnType<typeof vi.fn> } }
    update: ReturnType<typeof vi.fn>
    render: ReturnType<typeof vi.fn>
    setReducedMotion: ReturnType<typeof vi.fn>
  }>,
  resized: [] as Array<() => void>,
  disconnected: vi.fn(),
}))

vi.mock('pixi.js', () => ({
  Application: class {
    canvas = document.createElement('canvas')
    stage = { addChild: vi.fn() }
    renderer: { resize: ReturnType<typeof vi.fn> } | undefined = { resize: vi.fn() }
    ticker = { add: vi.fn() }
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
    root = { scale: { set: vi.fn() } }
    update = vi.fn()
    render = vi.fn()
    setReducedMotion = vi.fn()
    constructor() { hooks.scenes.push(this) }
  },
}))

beforeEach(() => {
  hooks.fail = false
  hooks.partial = true
  hooks.applications.length = 0
  hooks.scenes.length = 0
  hooks.resized.length = 0
  hooks.disconnected.mockClear()
  vi.stubGlobal('ResizeObserver', class {
    constructor(callback: () => void) { hooks.resized.push(callback) }
    observe(): void { /* Browser resize delivery is triggered explicitly below. */ }
    disconnect = hooks.disconnected
  })
  Object.defineProperty(window, 'devicePixelRatio', { value: 3, configurable: true })
})

describe('Pixi application lifetime', () => {
  it('sizes one canvas, updates the scene, and stops all work on idempotent destroy', async () => {
    const host = document.createElement('div')
    const width = vi.spyOn(host, 'clientWidth', 'get').mockReturnValue(720)
    const renderer = await createArenaRenderer(host)
    const app = hooks.applications[0]
    const scene = hooks.scenes[0]
    expect(host.querySelectorAll('canvas')).toHaveLength(1)
    expect(app.canvas).toHaveAttribute('aria-hidden', 'true')
    expect(app.init).toHaveBeenCalledWith(expect.objectContaining({ preference: 'webgl', resolution: 2 }))
    expect(app.renderer?.resize).toHaveBeenLastCalledWith(720, 624)
    width.mockReturnValue(360)
    hooks.resized[0]()
    expect(app.renderer?.resize).toHaveBeenLastCalledWith(360, 312)
    expect(scene.root.scale.set).toHaveBeenLastCalledWith(0.5)
    const snapshot = gameSnapshot()
    renderer.update(snapshot)
    renderer.setReducedMotion(true)
    const tick: () => void = app.ticker.add.mock.calls[0][0]
    tick()
    expect(scene.update).toHaveBeenCalledWith(snapshot, expect.any(Number))
    expect(scene.render).toHaveBeenCalledOnce()
    expect(scene.setReducedMotion).toHaveBeenCalledWith(true)
    renderer.destroy()
    renderer.destroy()
    renderer.update(snapshot)
    renderer.setReducedMotion(false)
    expect(host.querySelectorAll('canvas')).toHaveLength(0)
    expect(app.destroy).toHaveBeenCalledExactlyOnceWith(
      { removeView: true }, { children: true, context: true },
    )
    expect(hooks.disconnected).toHaveBeenCalledOnce()
    expect(scene.update).toHaveBeenCalledOnce()
    expect(scene.setReducedMotion).toHaveBeenCalledOnce()
  })

  it('uses a safe minimum surface and releases partially initialized GPU resources on failure', async () => {
    Object.defineProperty(window, 'devicePixelRatio', { value: 0, configurable: true })
    const host = document.createElement('div')
    const renderer = await createArenaRenderer(host)
    expect(hooks.applications[0].init).toHaveBeenCalledWith(expect.objectContaining({ resolution: 1 }))
    expect(hooks.applications[0].renderer?.resize).toHaveBeenLastCalledWith(1, 13 / 15)
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
