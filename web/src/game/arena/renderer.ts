import './pixiCompatibility'
import { Application } from 'pixi.js'
import { createAtlasArtwork } from './artAtlas/artwork'
import { loadArenaAtlas } from './artAtlas/load'
import type { AtlasRegions } from './artAtlas/regions'
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
  let regions: AtlasRegions
  try {
    await app.init({
      width: 15 * TILE, height: 13 * TILE, background: GROUND,
      preference: 'webgl', antialias: false, autoDensity: true,
      autoStart: false,
      resolution: Math.min(window.devicePixelRatio || 1, 2),
      powerPreference: 'low-power',
    })
    regions = await loadArenaAtlas()
  } catch (error) {
    try {
      if (app.renderer) app.destroy({ removeView: true }, { children: true, context: true })
    } catch { /* Retain the initialization error. */ }
    throw error
  }
  let scene: ArenaScene
  try {
    scene = new ArenaScene(app.renderer.resolution, createAtlasArtwork(regions))
  } catch (error) {
    try { app.destroy({ removeView: true }, { children: true, context: true }) } catch { /* Retain cause. */ }
    try { regions.destroy() } catch { /* Retain cause. */ }
    throw error
  }
  let destroyed = false
  let suspended = true
  let released = false
  let dimensions = '15x13'
  const restoreContext = (): void => {
    scene.restoreContext()
    if (!destroyed && !suspended) app.render()
  }
  const release = (): void => {
    if (released) return
    released = true
    try {
      app.canvas.style.visibility = 'hidden'
      app.ticker.stop()
      app.canvas.removeEventListener('webglcontextrestored', restoreContext)
      scene.reset()
    } finally {
      try {
        app.destroy({ removeView: true }, { children: true, context: true })
      } finally {
        regions.destroy()
      }
    }
  }
  try {
    app.stage.addChild(scene.root)
    app.canvas.setAttribute('aria-hidden', 'true')
    app.canvas.style.width = '100%'
    app.canvas.style.height = '100%'
    app.canvas.style.display = 'block'
    app.canvas.style.visibility = 'hidden'
    app.canvas.style.imageRendering = 'pixelated'
    host.appendChild(app.canvas)
    app.canvas.addEventListener('webglcontextrestored', restoreContext)
    app.ticker.add(() => scene.render(performance.now()))
  } catch (error) {
    destroyed = true
    try { release() } catch { /* Retain the setup error. */ }
    throw error
  }
  return {
    update(snapshot) {
      if (destroyed) return
      const nextDimensions = `${snapshot.state.arena.width}x${snapshot.state.arena.height}`
      if (nextDimensions !== dimensions) {
        app.renderer.resize(snapshot.state.arena.width * TILE, snapshot.state.arena.height * TILE)
        dimensions = nextDimensions
      }
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
      release()
    },
  }
}
