import { Application, Container, Sprite } from 'pixi.js'
import type { ActorDrawing } from '../src/game/arena/actorArt'
import { createAtlasArtwork } from '../src/game/arena/artAtlas/artwork'
import { loadArenaAtlas } from '../src/game/arena/artAtlas/load'
import type { ArenaArtwork } from '../src/game/arena/artwork'
import { GROUND, TILE } from '../src/game/arena/drawing'
import { graphicsArtwork } from '../src/game/arena/graphicsArtwork'
import { createArenaRenderer, type ArenaRenderer } from '../src/game/arena/renderer'
import { ArenaScene } from '../src/game/arena/scene'
import type { SnapshotMessage } from '../src/game/arena/types'
import { diagnosticSnapshot } from './snapshot'

type Pixels = { pixels: Uint8Array | Uint8ClampedArray; width: number; height: number }
type Comparison = {
  stage: string; differentBytes: number; differentPixels: number
  maxRgbaDelta: [number, number, number, number]; width: number; height: number
  bounds: { left: number; top: number; right: number; bottom: number } | null
  samples: Array<{ x: number; y: number; atlas: number[]; graphics: number[] }>
}
type Lifecycle = {
  canvasStable: boolean; hiddenBeforeUpdate: boolean; visibleAfterUpdate: boolean
  hiddenAfterSuspend: boolean; visibleAfterResume: boolean; removedAfterDestroy: boolean
  contextRestored: boolean
}
type Performance = { coldStartupMs: number; retainedFrames: number; retainedFramesMs: number }
type AtlasGameplayResult = {
  resolution: number
  comparisons: Comparison[]
  historicalDeltas: Comparison[]
  lifecycle: Lifecycle
  performance: Performance
  resources: {
    atlasNodes: number; atlasSprites: number; textureSources: number
    textureWidth: number; textureHeight: number; rgbaBytes: number
  }
}

const pulsedCues = [
  'protection', 'shield', 'stopwatch', 'goldenFlame', 'shortFuse', 'shortFlame',
  'longFuse', 'reverseControls', 'diarrhea', 'cloak',
] as const satisfies ReadonlyArray<keyof ActorDrawing>

function normalizedGraphicsArtwork(): ArenaArtwork {
  return {
    ...graphicsArtwork,
    makeActor(seat) {
      const actor = graphicsArtwork.makeActor(seat)
      for (const cue of pulsedCues) {
        const drawing = actor[cue]
        const index = actor.root.getChildIndex(drawing)
        const wrapper = new Container()
        wrapper.visible = drawing.visible
        drawing.visible = true
        drawing.alpha = 1
        actor.root.removeChild(drawing)
        wrapper.addChild(drawing)
        actor.root.addChildAt(wrapper, index)
        wrapper.cacheAsTexture({ resolution: 1, antialias: false, scaleMode: 'nearest' })
        actor[cue] = wrapper
      }
      return actor
    },
  }
}

declare global {
  interface Window {
    grenadiersAtlasGameplay: { run: () => Promise<AtlasGameplayResult> }
  }
}

function representativeSnapshot(): SnapshotMessage {
  const snapshot = diagnosticSnapshot(1)
  snapshot.state.pickups = [
    { cell: { column: 2, row: 3 }, kind: 'shield' },
    { cell: { column: 4, row: 3 }, kind: 'remoteBomb' },
    { cell: { column: 6, row: 3 }, kind: 'slowFoot' },
  ]
  snapshot.state.bombs = [{
    id: 31, owner: 10, range: 4, placedAt: 1, cell: { column: 5, row: 5 },
    blast: 'power', trigger: { type: 'timed', remainingTicks: 100, paused: false },
  }]
  snapshot.state.flames = [{ cell: { column: 6, row: 5 }, expiresAt: 20 }]
  snapshot.state.oilBarrels = [{ cell: { column: 8, row: 5 }, state: 'burning' }]
  Object.assign(snapshot.state.actors[0], {
    damageGraceRemainingTicks: 30, shieldRemainingTicks: 40,
    goldenFlameRemainingTicks: 50, shortFuseRemainingTicks: 60,
    slowFootRemainingTicks: [70, 80], kickEquipped: true,
  })
  return snapshot
}

const cueCases = [
  ['protection', 'damageGraceRemainingTicks', 120],
  ['shield', 'shieldRemainingTicks', 160],
  ['stopwatch', 'stopwatchRemainingTicks', 120],
  ['goldenFlame', 'goldenFlameRemainingTicks', 100],
  ['shortFuse', 'shortFuseRemainingTicks', 75],
  ['shortFlame', 'shortFlameRemainingTicks', 130],
  ['longFuse', 'longFuseRemainingTicks', 150],
  ['reverseControls', 'reverseControlsRemainingTicks', 140],
  ['diarrhea', 'diarrheaRemainingTicks', 105],
  ['cloak', 'invisibilityCloakRemainingTicks', 140],
] as const satisfies ReadonlyArray<readonly [string, keyof SnapshotMessage['state']['actors'][number], number]>

function isolatedCueSnapshot(
  name: typeof cueCases[number][1], roundId: number,
): SnapshotMessage {
  const snapshot = diagnosticSnapshot(1)
  snapshot.room.roundId = roundId
  snapshot.state.actors[0][name] = 60 as never
  return snapshot
}

function compare(stage: string, atlas: Pixels, graphics: Pixels): Comparison {
  let differentBytes = Math.abs(atlas.pixels.length - graphics.pixels.length)
  const differentPixels = new Set<number>()
  const maxRgbaDelta: [number, number, number, number] = [0, 0, 0, 0]
  const length = Math.min(atlas.pixels.length, graphics.pixels.length)
  for (let index = 0; index < length; index += 1) {
    const delta = Math.abs(atlas.pixels[index] - graphics.pixels[index])
    if (delta !== 0) { differentBytes += 1; differentPixels.add(Math.floor(index / 4)) }
    const channel = index % 4
    maxRgbaDelta[channel] = Math.max(maxRgbaDelta[channel], delta)
  }
  differentBytes += Number(atlas.width !== graphics.width) + Number(atlas.height !== graphics.height)
  const samples = [...differentPixels].slice(0, 8).map((pixel) => {
    const offset = pixel * 4
    return {
      x: pixel % atlas.width, y: Math.floor(pixel / atlas.width),
      atlas: Array.from(atlas.pixels.slice(offset, offset + 4)),
      graphics: Array.from(graphics.pixels.slice(offset, offset + 4)),
    }
  })
  const changed = [...differentPixels]
  const xs = changed.map((pixel) => pixel % atlas.width)
  const ys = changed.map((pixel) => Math.floor(pixel / atlas.width))
  const bounds = changed.length === 0 ? null : {
    left: Math.min(...xs), top: Math.min(...ys), right: Math.max(...xs), bottom: Math.max(...ys),
  }
  return {
    stage, differentBytes, differentPixels: differentPixels.size,
    maxRgbaDelta, width: atlas.width, height: atlas.height, bounds, samples,
  }
}

function descendants(root: ArenaScene['root']): Array<ArenaScene['root']> {
  return [root, ...root.children.flatMap((child) => descendants(child as ArenaScene['root']))]
}

function once(target: EventTarget, type: string): Promise<void> {
  return new Promise((resolve) => target.addEventListener(type, () => resolve(), { once: true }))
}

async function restoreContext(canvas: HTMLCanvasElement, force?: () => void): Promise<void> {
  const context = canvas.getContext('webgl2') ?? canvas.getContext('webgl')
  const extension = context?.getExtension('WEBGL_lose_context')
  if (!extension) throw new Error('WEBGL_lose_context is unavailable')
  const lost = once(canvas, 'webglcontextlost')
  const restored = once(canvas, 'webglcontextrestored')
  if (force) force()
  else extension.loseContext()
  await lost
  if (!force) {
    await new Promise((resolve) => setTimeout(resolve, 50))
    extension.restoreContext()
  }
  await Promise.race([
    restored,
    new Promise<never>((_resolve, reject) => setTimeout(
      () => reject(new Error('WebGL context restoration timed out')), 5_000,
    )),
  ])
}

async function frames(count: number): Promise<number> {
  const startedAt = performance.now()
  for (let index = 0; index < count; index += 1) {
    await new Promise(requestAnimationFrame)
  }
  return performance.now() - startedAt
}

async function run(): Promise<AtlasGameplayResult> {
  const resolution = Math.min(devicePixelRatio || 1, 2)
  const app = new Application()
  await app.init({
    width: 15 * TILE, height: 13 * TILE, background: GROUND,
    preference: 'webgl', antialias: false, autoDensity: true,
    autoStart: false, resolution, powerPreference: 'low-power',
  })
  document.body.appendChild(app.canvas)
  const regions = await loadArenaAtlas()
  const atlas = new ArenaScene(resolution, createAtlasArtwork(regions))
  const graphics = new ArenaScene(resolution, normalizedGraphicsArtwork())
  const historicalGraphics = new ArenaScene(resolution, graphicsArtwork)
  const comparisons: Comparison[] = []
  const historicalDeltas: Comparison[] = []
  let renderer: ArenaRenderer | undefined
  let regionsDestroyed = false
  let appDestroyed = false
  const host = document.querySelector<HTMLElement>('#diagnostic-host')!

  const pixels = (scene: ArenaScene): Pixels => {
    app.stage.removeChildren()
    app.stage.addChild(scene.root)
    app.render()
    return app.renderer.extract.pixels({ target: scene.root, resolution })
  }
  const primeNormalizedCaches = (): void => {
    const cached = descendants(graphics.root).filter((node) => node.isCachedAsTexture)
    const alphas = cached.map((node) => node.alpha)
    cached.forEach((node) => { node.alpha = 1; node.updateCacheTexture() })
    app.stage.removeChildren(); app.stage.addChild(graphics.root); app.render()
    cached.forEach((node, index) => { node.alpha = alphas[index] })
  }
  const capture = (stage: string): void => {
    const atlasPixels = pixels(atlas)
    const graphicsPixels = pixels(graphics)
    comparisons.push(compare(stage, atlasPixels, graphicsPixels))
    historicalDeltas.push(compare(stage, atlasPixels, pixels(historicalGraphics)))
  }

  try {
    const baseline = representativeSnapshot()
    atlas.update(baseline, 0); graphics.update(baseline, 0)
    historicalGraphics.update(baseline, 0); primeNormalizedCaches(); capture('running')
    const moving = structuredClone(baseline)
    moving.state.physicalTick = 4
    moving.state.actors[0].position.x += 1024
    atlas.update(moving, 100); graphics.update(moving, 100); historicalGraphics.update(moving, 100)
    atlas.render(117); graphics.render(117); historicalGraphics.render(117)
    capture('interpolated-fractional')
    atlas.render(180); graphics.render(180); historicalGraphics.render(180); capture('animation-80ms')
    atlas.render(190); graphics.render(190); historicalGraphics.render(190); capture('animation-90ms')
    atlas.render(210); graphics.render(210); historicalGraphics.render(210); capture('animation-110ms')
    atlas.setReducedMotion(true); graphics.setReducedMotion(true); historicalGraphics.setReducedMotion(true)
    atlas.render(180); graphics.render(180); historicalGraphics.render(180); capture('reduced-motion')
    const result = structuredClone(moving)
    result.room.phase = { type: 'result' }
    result.state.actors[1].alive = false
    atlas.update(result, 200); graphics.update(result, 200); historicalGraphics.update(result, 200)
    capture('result-hidden-actor')
    atlas.reset(); graphics.reset(); historicalGraphics.reset()
    atlas.update(baseline, 250); graphics.update(baseline, 250)
    historicalGraphics.update(baseline, 250); primeNormalizedCaches(); capture('reset')
    const changedTerrain = structuredClone(baseline)
    changedTerrain.state.physicalTick = 2
    changedTerrain.state.arena.terrain[changedTerrain.state.arena.width + 1] = 'wall'
    atlas.update(changedTerrain, 300); graphics.update(changedTerrain, 300)
    historicalGraphics.update(changedTerrain, 300)
    capture('changed-terrain')

    app.stage.removeChildren(); app.stage.addChild(atlas.root)
    const restore = (): void => atlas.restoreContext()
    app.canvas.addEventListener('webglcontextrestored', restore)
    const comparisonRenderer = app.renderer
    if (!('context' in comparisonRenderer)) throw new Error('Comparison renderer is not WebGL')
    await restoreContext(app.canvas, () => comparisonRenderer.context.forceContextLoss())
    app.canvas.removeEventListener('webglcontextrestored', restore)
    atlas.restoreContext(); graphics.restoreContext(); historicalGraphics.restoreContext()
    primeNormalizedCaches()
    capture('context-restored')

    atlas.setReducedMotion(false); graphics.setReducedMotion(false)
    historicalGraphics.setReducedMotion(false)
    cueCases.forEach(([name, field, period], index) => {
      const cue = isolatedCueSnapshot(field, index + 2)
      atlas.update(cue, 0); graphics.update(cue, 0); historicalGraphics.update(cue, 0)
      primeNormalizedCaches(); capture(`cue-${name}-low`)
      atlas.render(period); graphics.render(period); historicalGraphics.render(period)
      capture(`cue-${name}-high`)
    })

    const nodes = descendants(atlas.root)
    const sprites = nodes.filter((node) => node instanceof Sprite) as Sprite[]
    const sources = new Set(sprites.map((sprite) => sprite.texture.source))
    const source = sprites[0]?.texture.source
    const startupAt = performance.now()
    renderer = await createArenaRenderer(host)
    const coldStartupMs = performance.now() - startupAt
    const canvas = host.querySelector('canvas')!
    const hiddenBeforeUpdate = getComputedStyle(canvas).visibility === 'hidden'
    renderer.update(baseline)
    const visibleAfterUpdate = getComputedStyle(canvas).visibility === 'visible'
    renderer.setReducedMotion(true)
    renderer.suspend()
    const hiddenAfterSuspend = getComputedStyle(canvas).visibility === 'hidden'
    renderer.update(result)
    const visibleAfterResume = getComputedStyle(canvas).visibility === 'visible'
    await restoreContext(canvas)
    const retainedFrames = 30
    const retainedFramesMs = await frames(retainedFrames)
    const canvasStable = host.querySelector('canvas') === canvas

    atlas.reset(); graphics.reset(); historicalGraphics.reset()
    app.stage.removeChildren()
    atlas.root.destroy({ children: true, context: true })
    graphics.root.destroy({ children: true, context: true })
    historicalGraphics.root.destroy({ children: true, context: true })
    app.destroy({ removeView: true }, { children: true, context: true })
    appDestroyed = true
    regions.destroy()
    regionsDestroyed = true
    renderer.destroy()
    renderer = undefined
    return {
      resolution, comparisons, historicalDeltas,
      lifecycle: {
        canvasStable, hiddenBeforeUpdate, visibleAfterUpdate, hiddenAfterSuspend,
        visibleAfterResume, removedAfterDestroy: host.querySelector('canvas') === null,
        contextRestored: true,
      },
      performance: { coldStartupMs, retainedFrames, retainedFramesMs },
      resources: {
        atlasNodes: nodes.length,
        atlasSprites: sprites.length,
        textureSources: sources.size,
        textureWidth: source?.pixelWidth ?? 0,
        textureHeight: source?.pixelHeight ?? 0,
        rgbaBytes: (source?.pixelWidth ?? 0) * (source?.pixelHeight ?? 0) * 4,
      },
    }
  } finally {
    if (renderer) renderer.destroy()
    if (!appDestroyed) {
      atlas.reset(); graphics.reset(); historicalGraphics.reset()
      app.destroy({ removeView: true }, { children: true, context: true })
    }
    if (!regionsDestroyed) regions.destroy()
  }
}

window.grenadiersAtlasGameplay = { run }
