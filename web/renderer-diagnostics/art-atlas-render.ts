import { Application, Container, Rectangle, Texture } from 'pixi.js'
import {
  createArtCatalog, createAtlasRegions, type ArtManifest, type AtlasRegions,
  type PixelBuffer, type RuntimeArtManifest,
} from '../src/game/arena/artAtlas'
import { pixelCanvas } from './art-atlas-pixels'

export type PreviewController = {
  app: Application
  regions: AtlasRegions
  renderMs: number
  spriteCount: number
  pixels: PixelBuffer
  destroy(): void
}

const previewResources = { created: 0, destroyed: 0, live: 0 }

export function readPreviewResourceState(): Readonly<typeof previewResources> {
  return { ...previewResources }
}

function extractedPixels(app: Application, width: number, height: number): PixelBuffer {
  const result = app.renderer.extract.pixels({
    target: app.stage, frame: new Rectangle(0, 0, width, height), resolution: 1,
  })
  return { width: result.width, height: result.height, data: result.pixels }
}

export async function renderBaselineSource(manifest: ArtManifest): Promise<PixelBuffer> {
  const app = new Application()
  await app.init({
    width: manifest.sheet.width, height: manifest.sheet.height,
    backgroundAlpha: 0, antialias: false, autoDensity: false,
    autoStart: false, resolution: 1, preference: 'webgl', powerPreference: 'low-power',
  })
  const catalog = createArtCatalog()
  const leases = catalog.entries.map((entry) => {
    const lease = entry.createDrawing()
    const { sourceRect, localBounds } = entry.frame
    lease.drawing.position.set(sourceRect.x - localBounds.x, sourceRect.y - localBounds.y)
    app.stage.addChild(lease.drawing)
    return lease
  })
  try {
    app.render()
    return extractedPixels(app, manifest.sheet.width, manifest.sheet.height)
  } finally {
    app.stage.removeChildren()
    for (const lease of leases) lease.destroy()
    app.destroy({ removeView: true }, { children: true, context: true })
  }
}

function guideCanvas(manifest: ArtManifest): HTMLCanvasElement {
  const canvas = document.createElement('canvas')
  canvas.width = manifest.sheet.width
  canvas.height = manifest.sheet.height
  const context = canvas.getContext('2d')
  if (!context) throw new Error('Canvas 2D is unavailable')
  context.lineWidth = 1
  context.font = '8px ui-monospace, monospace'
  context.textBaseline = 'top'
  for (const frame of manifest.frames) {
    const slotX = Math.floor(frame.sourceRect.x / manifest.sheet.slotSize) * manifest.sheet.slotSize
    const slotY = Math.floor(frame.sourceRect.y / manifest.sheet.slotSize) * manifest.sheet.slotSize
    context.strokeStyle = frame.status === 'reserved' ? '#9b443c' : '#a9782c'
    context.strokeRect(slotX + 0.5, slotY + 0.5, manifest.sheet.slotSize - 1, manifest.sheet.slotSize - 1)
    context.strokeStyle = '#172a3a'
    context.strokeRect(
      frame.sourceRect.x + 0.5, frame.sourceRect.y + 0.5,
      frame.sourceRect.width - 1, frame.sourceRect.height - 1,
    )
    const lines = labelLines(frame.id)
    context.fillStyle = '#172a3a'
    context.fillRect(slotX + 2, slotY + manifest.sheet.slotSize - 23, manifest.sheet.slotSize - 4, 21)
    context.fillStyle = '#eef2f3'
    lines.forEach((line, index) => context.fillText(
      line, slotX + 5, slotY + manifest.sheet.slotSize - 21 + index * 9,
    ))
  }
  return canvas
}

function labelLines(id: string): readonly string[] {
  if (id.length <= 18) return [id]
  const punctuation = id.lastIndexOf('.', 18)
  const split = punctuation >= 9 ? punctuation + 1 : 18
  return [id.slice(0, split), id.slice(split)]
}

function xml(value: string): string {
  return value.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('"', '&quot;')
}

export function guideSvg(manifest: ArtManifest): string {
  const elements = manifest.frames.map((frame) => {
    const slotX = Math.floor(frame.sourceRect.x / manifest.sheet.slotSize) * manifest.sheet.slotSize
    const slotY = Math.floor(frame.sourceRect.y / manifest.sheet.slotSize) * manifest.sheet.slotSize
    const color = frame.status === 'reserved' ? '#9b443c' : '#a9782c'
    const lines = labelLines(frame.id)
    const labels = lines.map((line, index) => `<tspan x="${slotX + 5}" y="${slotY + 98 + index * 9}">${xml(line)}</tspan>`).join('')
    return `<g><rect x="${slotX + 0.5}" y="${slotY + 0.5}" width="111" height="111" fill="none" stroke="${color}"/><rect x="${frame.sourceRect.x + 0.5}" y="${frame.sourceRect.y + 0.5}" width="${frame.sourceRect.width - 1}" height="${frame.sourceRect.height - 1}" fill="none" stroke="#172a3a"/><rect x="${slotX + 2}" y="${slotY + 89}" width="108" height="21" fill="#172a3a"/><text fill="#eef2f3" font-family="monospace" font-size="8">${labels}</text></g>`
  }).join('')
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${manifest.sheet.width}" height="${manifest.sheet.height}" viewBox="0 0 ${manifest.sheet.width} ${manifest.sheet.height}">${elements}</svg>`
}

export function renderGuides(manifest: ArtManifest): PixelBuffer {
  const canvas = guideCanvas(manifest)
  const context = canvas.getContext('2d')
  if (!context) throw new Error('Canvas 2D is unavailable')
  return { width: canvas.width, height: canvas.height, data: context.getImageData(0, 0, canvas.width, canvas.height).data }
}

export function renderOverview(source: PixelBuffer, manifest: ArtManifest): PixelBuffer {
  const canvas = document.createElement('canvas')
  canvas.width = source.width
  canvas.height = source.height
  const context = canvas.getContext('2d')
  if (!context) throw new Error('Canvas 2D is unavailable')
  context.fillStyle = '#d9e2e6'
  context.fillRect(0, 0, canvas.width, canvas.height)
  context.fillStyle = '#c2cdd1'
  for (let y = 0; y < canvas.height; y += 16) {
    for (let x = (y / 16) % 2 === 0 ? 0 : 16; x < canvas.width; x += 32) context.fillRect(x, y, 16, 16)
  }
  context.drawImage(pixelCanvas(source), 0, 0)
  context.drawImage(guideCanvas(manifest), 0, 0)
  return { width: canvas.width, height: canvas.height, data: context.getImageData(0, 0, canvas.width, canvas.height).data }
}

function spriteAt(regions: AtlasRegions, id: string, x: number, y: number): ReturnType<AtlasRegions['createSprite']> {
  const sprite = regions.createSprite(id)
  sprite.position.set(x, y)
  return sprite
}

function addTerrain(stage: Container, regions: AtlasRegions): void {
  for (let row = 0; row < 13; row += 1) {
    for (let column = 0; column < 15; column += 1) {
      const edge = row === 0 || row === 12 || column === 0 || column === 14
      const pillar = row % 2 === 0 && column % 2 === 0
      const id = edge || pillar ? `terrain.wall.${(row + column) % 2}` : `terrain.floor.${(row * 3 + column) % 8}`
      stage.addChild(spriteAt(regions, id, column * 48, row * 48))
    }
  }
  for (const [column, row] of [[3, 2], [5, 2], [9, 8], [11, 10]]) {
    stage.addChild(spriteAt(regions, 'terrain.block.brick', column * 48, row * 48))
  }
}

function addPickups(stage: Container, regions: AtlasRegions, manifest: RuntimeArtManifest): void {
  const ids = manifest.frames.filter(({ id }) => id.startsWith('pickup.')).map(({ id }) => id)
  ids.forEach((id, index) => {
    const column = 1 + (index % 12)
    const row = 3 + Math.floor(index / 12) * 2
    stage.addChild(spriteAt(regions, id, column * 48, row * 48))
  })
}

function addActorsAndCues(stage: Container, regions: AtlasRegions): void {
  const first = { x: 120, y: 504 }
  const second = { x: 600, y: 120 }
  for (const id of [
    'actor.shadow', 'actor.one.east.1', 'actor.cue.shield', 'actor.cue.goldenFlame',
    'actor.cue.slowBoot', 'actor.cue.slowMarker.1', 'actor.cue.kick',
  ]) stage.addChild(spriteAt(regions, id, first.x, first.y))
  for (const id of [
    'actor.shadow', 'actor.two.west.0', 'actor.cue.protection', 'actor.cue.stopwatch',
    'actor.cue.reverseControls', 'actor.cue.piercing', 'actor.cue.bracelet',
  ]) stage.addChild(spriteAt(regions, id, second.x, second.y))
}

function addBombsAndEffects(stage: Container, regions: AtlasRegions): void {
  const bombs: ReadonlyArray<[string, number, number, readonly string[]]> = [
    ['bomb.shell.timed.standard', 264, 504, ['bomb.shadow', 'bomb.spark']],
    ['bomb.shell.remote.piercing', 360, 504, ['bomb.shadow']],
    ['bomb.shell.timed.power', 456, 504, ['bomb.shadow', 'bomb.pause']],
    ['bomb.shell.contact.standard', 552, 504, ['bomb.shadow']],
  ]
  for (const [shell, x, y, parts] of bombs) {
    for (const id of parts.filter((part) => part === 'bomb.shadow')) {
      stage.addChild(spriteAt(regions, id, x, y))
    }
    stage.addChild(spriteAt(regions, shell, x, y))
    for (const id of parts.filter((part) => part !== 'bomb.shadow')) {
      stage.addChild(spriteAt(regions, id, x, y))
    }
  }
  for (const [id, column, row] of [
    ['effect.flame.0', 7, 9], ['effect.flame.1', 8, 9], ['effect.flame.0', 9, 9],
  ] as const) stage.addChild(spriteAt(regions, id, column * 48, row * 48))
  stage.addChild(spriteAt(regions, 'oil.unlit', 48, 336))
  stage.addChild(spriteAt(regions, 'oil.charred', 624, 336))
  stage.addChild(spriteAt(regions, 'effect.flame.1', 624, 336))
}

export async function renderRuntimePreview(
  runtimePixels: PixelBuffer,
  manifest: RuntimeArtManifest,
  host?: HTMLElement,
): Promise<PreviewController> {
  const app = new Application()
  await app.init({
    width: 720, height: 624, background: 0x879181,
    antialias: false, autoDensity: false, autoStart: false, resolution: 1,
    preference: 'webgl', powerPreference: 'low-power',
  })
  let baseTexture: Texture | undefined
  let regions: AtlasRegions | undefined
  try {
    baseTexture = Texture.from(pixelCanvas(runtimePixels))
    regions = createAtlasRegions(baseTexture.source, manifest)
    addTerrain(app.stage, regions)
    addPickups(app.stage, regions, manifest)
    addActorsAndCues(app.stage, regions)
    addBombsAndEffects(app.stage, regions)
    const started = performance.now()
    app.render()
    const renderMs = performance.now() - started
    const pixels = extractedPixels(app, 720, 624)
    host?.appendChild(app.canvas)
    let destroyed = false
    previewResources.created += 1
    previewResources.live += 1
    const ownedTexture = baseTexture
    const ownedRegions = regions
    return {
      app, regions, renderMs, pixels, spriteCount: app.stage.children.length,
      destroy() {
        if (destroyed) return
        destroyed = true
        app.stage.removeChildren().forEach((child) => child.destroy())
        ownedTexture.destroy(false)
        ownedRegions.destroy()
        app.destroy({ removeView: true }, { children: true, context: true })
        previewResources.destroyed += 1
        previewResources.live -= 1
      },
    }
  } catch (error) {
    app.stage.removeChildren().forEach((child) => child.destroy())
    baseTexture?.destroy(false)
    regions?.destroy()
    app.destroy({ removeView: true }, { children: true, context: true })
    throw error
  }
}

export async function inspectImportedFrame(
  runtimePixels: PixelBuffer,
  manifest: RuntimeArtManifest,
  id: string,
): Promise<{ pixels: PixelBuffer; bounds: { x: number; y: number; width: number; height: number } }> {
  const frame = manifest.frames.find((candidate) => candidate.id === id)
  if (!frame) throw new RangeError(`Unknown frame ${id}`)
  const app = new Application()
  await app.init({
    width: frame.sourceRect.width, height: frame.sourceRect.height,
    backgroundAlpha: 0, antialias: false, autoDensity: false, autoStart: false,
    resolution: 1, preference: 'webgl', powerPreference: 'low-power',
  })
  const baseTexture = Texture.from(pixelCanvas(runtimePixels))
  const regions = createAtlasRegions(baseTexture.source, manifest)
  const sprite = regions.createSprite(id)
  sprite.position.set(frame.pivot.x, frame.pivot.y)
  app.stage.addChild(sprite)
  try {
    app.render()
    const bounds = sprite.getBounds()
    return {
      pixels: extractedPixels(app, frame.sourceRect.width, frame.sourceRect.height),
      bounds: { x: bounds.x, y: bounds.y, width: bounds.width, height: bounds.height },
    }
  } finally {
    app.stage.removeChild(sprite)
    sprite.destroy()
    baseTexture.destroy(false)
    regions.destroy()
    app.destroy({ removeView: true }, { children: true, context: true })
  }
}
