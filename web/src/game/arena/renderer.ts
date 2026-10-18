import { Application } from 'pixi.js'
import { GROUND, TILE } from './drawing'
import { ArenaScene } from './scene'
import type { SnapshotMessage } from './types'

export interface ArenaRenderer {
  update: (snapshot: SnapshotMessage) => void
  suspend: () => void
  setReducedMotion: (reduced: boolean) => void
  destroy: () => void
}

export async function createArenaRenderer(host: HTMLElement): Promise<ArenaRenderer> {
  const app = new Application()
  try {
    await app.init({
      width: 15 * TILE, height: 13 * TILE, background: GROUND,
      preference: 'webgl', antialias: false, autoDensity: true,
      autoStart: false,
      resolution: Math.min(window.devicePixelRatio || 1, 2),
      powerPreference: 'low-power',
    })
  } catch (error) {
    if (app.renderer) app.destroy({ removeView: true }, { children: true, context: true })
    throw error
  }
  const scene = new ArenaScene()
  app.stage.addChild(scene.root)
  app.canvas.setAttribute('aria-hidden', 'true')
  app.canvas.style.width = '100%'
  app.canvas.style.height = '100%'
  app.canvas.style.display = 'block'
  app.canvas.style.visibility = 'hidden'
  app.canvas.style.imageRendering = 'pixelated'
  host.appendChild(app.canvas)
  app.ticker.add(() => scene.render(performance.now()))
  let destroyed = false
  let suspended = true
  return {
    update(snapshot) {
      if (destroyed) return
      scene.update(snapshot, performance.now())
      if (!suspended) return
      app.render()
      app.canvas.style.visibility = 'visible'
      app.ticker.start()
      suspended = false
    },
    suspend() {
      if (destroyed || suspended) return
      app.canvas.style.visibility = 'hidden'
      app.ticker.stop()
      scene.reset()
      suspended = true
    },
    setReducedMotion(reduced) { if (!destroyed) scene.setReducedMotion(reduced) },
    destroy() {
      if (destroyed) return
      destroyed = true
      app.canvas.style.visibility = 'hidden'
      app.ticker.stop()
      scene.reset()
      app.destroy({ removeView: true }, { children: true, context: true })
    },
  }
}
