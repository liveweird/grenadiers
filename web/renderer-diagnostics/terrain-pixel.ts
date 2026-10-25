import '../src/game/arena/pixiCompatibility'
import { Application, Graphics } from 'pixi.js'
import { GROUND, paintTerrain, TILE } from '../src/game/arena/drawing'
import { ArenaScene } from '../src/game/arena/scene'
import type { SnapshotMessage } from '../src/game/arena/types'
import { diagnosticSnapshot } from './snapshot'

type Pixels = { pixels: Uint8ClampedArray; width: number; height: number }
type Comparison = { stage: string; width: number; height: number; differentBytes: number }
type TerrainPixelResult = { resolution: number; comparisons: Comparison[] }

type TerrainPixelApi = { run: () => Promise<TerrainPixelResult> }

declare global {
  interface Window { grenadiersTerrainPixel: TerrainPixelApi }
}

function terrainOnlySnapshot(): SnapshotMessage {
  const snapshot = diagnosticSnapshot(1)
  snapshot.state.arena.blocks = []
  snapshot.state.actors = []
  snapshot.state.bombs = []
  snapshot.state.flames = []
  snapshot.state.pickups = []
  return snapshot
}

function compare(stage: string, actual: Pixels, expected: Pixels): Comparison {
  let differentBytes = Math.abs(actual.pixels.length - expected.pixels.length)
  const sharedLength = Math.min(actual.pixels.length, expected.pixels.length)
  for (let index = 0; index < sharedLength; index += 1) {
    if (actual.pixels[index] !== expected.pixels[index]) differentBytes += 1
  }
  return {
    stage,
    width: actual.width,
    height: actual.height,
    differentBytes: differentBytes + Number(actual.width !== expected.width)
      + Number(actual.height !== expected.height),
  }
}

function referencePixels(
  app: Application, snapshot: SnapshotMessage, resolution: number,
): Pixels {
  const reference = new Graphics()
  paintTerrain(reference, snapshot.state.arena)
  const pixels = app.renderer.extract.pixels({ target: reference, resolution })
  reference.destroy({ context: true })
  return pixels
}

function once(target: EventTarget, type: string): Promise<void> {
  return new Promise((resolve) => target.addEventListener(type, () => resolve(), { once: true }))
}

async function restoreWebGlContext(app: Application): Promise<void> {
  if (!('context' in app.renderer)) throw new Error('The diagnostic renderer is not WebGL')
  const canvas = app.canvas
  const lost = once(canvas, 'webglcontextlost')
  const restored = once(canvas, 'webglcontextrestored')
  app.renderer.context.forceContextLoss()
  await lost
  await restored
}

async function run(): Promise<TerrainPixelResult> {
  const resolution = Math.min(window.devicePixelRatio || 1, 2)
  const app = new Application()
  await app.init({
    width: 15 * TILE, height: 13 * TILE, background: GROUND,
    preference: 'webgl', antialias: false, autoDensity: true,
    autoStart: false, resolution, powerPreference: 'low-power',
  })
  document.body.appendChild(app.canvas)
  const scene = new ArenaScene(app.renderer.resolution)
  app.stage.addChild(scene.root)
  const restore = (): void => scene.restoreContext()
  app.canvas.addEventListener('webglcontextrestored', restore)
  const comparisons: Comparison[] = []
  try {
    const baseline = terrainOnlySnapshot()
    scene.update(baseline, 0)
    app.render()
    comparisons.push(compare(
      'baseline', app.renderer.extract.pixels({ target: scene.root, resolution }),
      referencePixels(app, baseline, resolution),
    ))

    const repaint = structuredClone(baseline)
    repaint.state.arena.terrain[baseline.state.arena.width + 1] = 'wall'
    scene.update(repaint, 50)
    app.render()
    comparisons.push(compare(
      'repaint', app.renderer.extract.pixels({ target: scene.root, resolution }),
      referencePixels(app, repaint, resolution),
    ))

    scene.reset()
    scene.update(baseline, 100)
    app.render()
    comparisons.push(compare(
      'reset', app.renderer.extract.pixels({ target: scene.root, resolution }),
      referencePixels(app, baseline, resolution),
    ))

    await restoreWebGlContext(app)
    app.render()
    comparisons.push(compare(
      'context-restored', app.renderer.extract.pixels({ target: scene.root, resolution }),
      referencePixels(app, baseline, resolution),
    ))
    return { resolution, comparisons }
  } finally {
    app.canvas.removeEventListener('webglcontextrestored', restore)
    scene.reset()
    app.destroy({ removeView: true }, { children: true, context: true })
  }
}

window.grenadiersTerrainPixel = { run }
