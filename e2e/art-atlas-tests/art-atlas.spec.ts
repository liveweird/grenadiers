import { readFile } from 'node:fs/promises'
import { expect, test, type Download, type Page } from '@playwright/test'
import { observeBrowserPolicy, type BrowserPolicyObservation } from '../tests/support/browser-policy'
import { inspectActorInventory } from './actor-animation-inventory'

let browserPolicy: BrowserPolicyObservation
test.beforeEach(async ({ context }) => { browserPolicy = await observeBrowserPolicy(context) })
test.afterEach(() => { expect(browserPolicy.violations).toEqual([]) })

type DiagnosticState = {
  initialized: boolean
  rebuildCount: number
  canvasCount: number
  spriteCount: number
  frameCount: number
  texture: { width: number; height: number; rgbaBytes: number } | null
  renderMs: number | null
  resources: { created: number; destroyed: number; live: number }
}
type ExportRequest = { pngBase64: string; manifestJson: string; slices?: boolean }
type TemplateFile = { name: string; base64?: string; text?: string }
type ExportResult = {
  files: TemplateFile[]
  runtimeManifest: { frames: Array<{ id: string }> }
  rgbaBytes: number
}
type RegionComparison = {
  differentBytes: number
  bounds: { x: number; y: number; width: number; height: number }
}
type TestApi = {
  readState(): DiagnosticState
  createTemplate(): Promise<{ files: TemplateFile[] }>
  createEditedTemplate(edit: {
    x: number; y: number; rgba: readonly [number, number, number, number]
  }): Promise<ExportRequest>
  exportSource(request: ExportRequest): Promise<ExportResult>
  compareImportedFrame(id: string): Promise<RegionComparison>
  resetPreview(): Promise<DiagnosticState>
  restorePreviewContext(): Promise<DiagnosticState>
  measureRenders(count: number): { count: number; elapsedMs: number }
}

declare global {
  interface Window { grenadiersArtAtlas: TestApi }
}

async function openWorkbench(page: Page): Promise<DiagnosticState> {
  await page.goto('/art-atlas.html')
  await expect(page.locator('[data-build-target]'))
    .toHaveAttribute('data-build-target', 'grenadiers-art-atlas-diagnostic')
  await expect.poll(() => page.evaluate(() => window.grenadiersArtAtlas?.readState().initialized))
    .toBe(true)
  return page.evaluate(() => window.grenadiersArtAtlas.readState())
}

async function committedSource(): Promise<ExportRequest> {
  const [png, manifestJson] = await Promise.all([
    readFile(new URL('../../assets/art/arena-source.png', import.meta.url)),
    readFile(new URL('../../assets/art/arena-source.json', import.meta.url), 'utf8'),
  ])
  return { pngBase64: png.toString('base64'), manifestJson }
}

async function compareDecodedPng(
  page: Page,
  firstBase64: string,
  secondBase64: string,
  region?: { x: number; y: number; width: number; height: number },
): Promise<{ width: number; height: number; differentBytes: number }> {
  return page.evaluate(async ({ firstBase64: first, secondBase64: second, region: requested }) => {
    const decode = async (base64: string): Promise<{ width: number; height: number; pixels: Uint8ClampedArray }> => {
      const binary = atob(base64)
      const bytes = Uint8Array.from(binary, (value) => value.charCodeAt(0))
      const bitmap = await createImageBitmap(new Blob([bytes.buffer], { type: 'image/png' }))
      try {
        const canvas = document.createElement('canvas')
        canvas.width = bitmap.width
        canvas.height = bitmap.height
        const context = canvas.getContext('2d')
        if (!context) throw new Error('Canvas 2D unavailable while comparing PNG pixels')
        context.drawImage(bitmap, 0, 0)
        return {
          width: canvas.width, height: canvas.height,
          pixels: context.getImageData(0, 0, canvas.width, canvas.height).data,
        }
      } finally { bitmap.close() }
    }
    const [left, right] = await Promise.all([decode(first), decode(second)])
    if (left.width !== right.width || left.height !== right.height) {
      return { width: left.width, height: left.height, differentBytes: Number.POSITIVE_INFINITY }
    }
    const area = requested ?? { x: 0, y: 0, width: left.width, height: left.height }
    let differentBytes = 0
    for (let y = area.y; y < area.y + area.height; y += 1) {
      const start = (y * left.width + area.x) * 4
      const end = start + area.width * 4
      for (let index = start; index < end; index += 1) {
        if (left.pixels[index] !== right.pixels[index]) differentBytes += 1
      }
    }
    return { width: left.width, height: left.height, differentBytes }
  }, { firstBase64, secondBase64, region })
}

function canonicalJson(value: unknown): string {
  const canonical = (entry: unknown): unknown => {
    if (Array.isArray(entry)) return entry.map(canonical)
    if (entry && typeof entry === 'object') {
      return Object.fromEntries(Object.entries(entry).sort(([left], [right]) => left.localeCompare(right))
        .map(([key, child]) => [key, canonical(child)]))
    }
    return entry
  }
  return `${JSON.stringify(canonical(value), null, 2)}\n`
}

test('baseline regions match source pixels and one edited sprite keeps its import position', async ({ page }) => {
  await openWorkbench(page)
  const baseline = await page.evaluate(() => window.grenadiersArtAtlas.compareImportedFrame('terrain.floor.0'))
  expect(baseline).toEqual({ differentBytes: 0, bounds: { x: 0, y: 0, width: 48, height: 48 } })

  const edited = await page.evaluate(async () => {
    const source = await window.grenadiersArtAtlas.createEditedTemplate({
      x: 1, y: 1, rgba: [255, 0, 255, 255],
    })
    await window.grenadiersArtAtlas.exportSource(source)
    return window.grenadiersArtAtlas.compareImportedFrame('terrain.floor.0')
  })
  expect(edited).toEqual({ differentBytes: 0, bounds: { x: 0, y: 0, width: 48, height: 48 } })
})

test('source validation rejects path-like IDs and painting in authoring gutters', async ({ page }) => {
  await page.goto('/art-atlas.html?cli=1')
  const result = await page.evaluate(async () => {
    const template = await window.grenadiersArtAtlas.createTemplate()
    const png = template.files.find(({ name }) => name === 'arena-source.png')?.base64
    const manifestFile = template.files.find(({ name }) => name === 'arena-source.json')?.text
    if (!png || !manifestFile) throw new Error('Template files are incomplete')
    const parsed = JSON.parse(manifestFile) as { frames: Array<{ id: string }> }
    const inventory = {
      frameCount: parsed.frames.length,
      pickupCount: parsed.frames.filter(({ id }) => id.startsWith('pickup.')).length,
    }
    const unsafe = structuredClone(parsed)
    unsafe.frames[0].id = '../terrain'
    let unsafeMessage = ''
    try {
      await window.grenadiersArtAtlas.exportSource({ pngBase64: png, manifestJson: JSON.stringify(unsafe) })
    } catch (error) { unsafeMessage = String(error) }
    const gutter = await window.grenadiersArtAtlas.createEditedTemplate({
      x: 100, y: 100, rgba: [255, 0, 255, 255],
    })
    let gutterMessage = ''
    try {
      await window.grenadiersArtAtlas.exportSource(gutter)
    } catch (error) { gutterMessage = String(error) }
    return { unsafeMessage, gutterMessage, inventory }
  })
  expect(result.unsafeMessage).toContain('invalid stable ID ../terrain')
  expect(result.gutterMessage).toContain('paint exists in a gutter or reserved slot')
  expect(result.inventory).toEqual({ frameCount: 149, pickupCount: 24 })
})

test('repeated preview reset and WebGL context restoration do not accumulate canvases or sprites', async ({ page }) => {
  const initial = await openWorkbench(page)
  const resets = await page.evaluate(async () => {
    const api = window.grenadiersArtAtlas
    return [await api.resetPreview(), await api.resetPreview(), await api.resetPreview()]
  })
  const canvas = page.locator('#preview-host canvas')
  const beforeRestore = await canvas.screenshot()
  const restored = await page.evaluate(() => window.grenadiersArtAtlas.restorePreviewContext())
  const afterRestore = await canvas.screenshot()
  const regionPixels = await compareDecodedPng(
    page, beforeRestore.toString('base64'), afterRestore.toString('base64'),
    { x: 48, y: 48, width: 48, height: 48 },
  )
  const imported = await page.evaluate(() => window.grenadiersArtAtlas.compareImportedFrame('terrain.floor.0'))
  const states = [...resets, restored]
  expect(states.every(({ canvasCount }) => canvasCount === 1)).toBe(true)
  expect(states.every(({ spriteCount }) => spriteCount === initial.spriteCount)).toBe(true)
  expect(states.at(-1)?.rebuildCount).toBe(initial.rebuildCount + 3)
  expect(states.at(-1)?.resources).toEqual({
    created: initial.resources.created + 3,
    destroyed: initial.resources.destroyed + 3,
    live: 1,
  })
  expect(regionPixels.width).toBe(720)
  expect(regionPixels.height).toBeGreaterThanOrEqual(624)
  expect(regionPixels.differentBytes).toBe(0)
  expect(imported.differentBytes).toBe(0)
})

test('independent exports match decoded runtime pixels and canonical JSON', async ({ page }) => {
  await page.goto('/art-atlas.html?cli=1')
  const source = await committedSource()
  const [first, second] = await page.evaluate(async (request) => {
    const api = window.grenadiersArtAtlas
    return [await api.exportSource(request), await api.exportSource(request)]
  }, source)
  const output = (result: ExportResult, name: string): TemplateFile => {
    const file = result.files.find((candidate) => candidate.name === name)
    if (!file) throw new Error(`Export omitted ${name}`)
    return file
  }
  const firstPng = output(first, 'arena-atlas.png').base64
  const secondPng = output(second, 'arena-atlas.png').base64
  const firstJson = output(first, 'arena-atlas.json').text
  const secondJson = output(second, 'arena-atlas.json').text
  if (!firstPng || !secondPng || !firstJson || !secondJson) throw new Error('Runtime export contents are incomplete')
  expect(await compareDecodedPng(page, firstPng, secondPng))
    .toEqual({ width: 2040, height: 528, differentBytes: 0 })
  expect(secondJson).toBe(firstJson)
  expect(firstJson).toBe(canonicalJson(JSON.parse(firstJson)))
})

test('committed artist source validates and exports through the browser pipeline', async ({ page }) => {
  await page.goto('/art-atlas.html?cli=1')
  const source = await committedSource()
  const sourceManifest = JSON.parse(source.manifestJson) as {
    sheet: { width: number; height: number }
    frames: Array<{
      id: string; status: string
      footAnchor?: { x: number; y: number }
    }>
  }
  const result = await page.evaluate(
    (request) => window.grenadiersArtAtlas.exportSource(request), source,
  )
  expect(sourceManifest.sheet).toMatchObject({ width: 1120, height: 1680 })
  expect(sourceManifest.frames).toHaveLength(149)
  expect(sourceManifest.frames.filter(({ status }) => status === 'baseline')).toHaveLength(145)
  expect(sourceManifest.frames.filter(({ status }) => status === 'reserved')).toHaveLength(4)
  expect(result.runtimeManifest.frames).toHaveLength(145)
  expect(result.rgbaBytes).toBe(4_308_480)
  const appendedWalkingIds = (['one', 'two', 'three', 'four'] as const).flatMap((appearance) =>
    (['north', 'east', 'south', 'west'] as const).flatMap((direction) =>
      [2, 3].map((pose) => `actor.${appearance}.${direction}.${pose}`)))
  expect(sourceManifest.frames.slice(111, 143).map(({ id }) => id)).toEqual(appendedWalkingIds)
  expect(sourceManifest.frames.slice(143).map(({ id }) => id)).toEqual(
    (['standard', 'piercing', 'power'] as const).flatMap((blast) =>
      [1, 2].map((pose) => `bomb.shell.timed.${blast}.${pose}`)),
  )
  const actorFrames = sourceManifest.frames.filter(({ id }) => id.startsWith('actor.')
    && /^actor\.(one|two|three|four)\.(north|east|south|west)\.[0-3]$/.test(id))
  expect(actorFrames).toHaveLength(64)
  expect(actorFrames.every(({ footAnchor }) => footAnchor?.x === 40 && footAnchor.y === 65)).toBe(true)
  expect(result.files.map(({ name }) => name)).toEqual([
    'arena-atlas.png', 'arena-atlas.json', 'arena-overview.png', 'arena-preview.png',
  ])
})

test('all adopted directional actors retain binary alpha and bounded opaque palettes', async ({ page }) => {
  await page.goto('/art-atlas.html?cli=1')
  const metrics = await inspectActorInventory(page, await committedSource())
  expect(metrics.map(({ appearance, frameCount, invalidAlpha }) =>
    ({ appearance, frameCount, invalidAlpha })), JSON.stringify(metrics)).toEqual([
    { appearance: 'one', frameCount: 16, invalidAlpha: 0 },
    { appearance: 'two', frameCount: 16, invalidAlpha: 0 },
    { appearance: 'three', frameCount: 16, invalidAlpha: 0 },
    { appearance: 'four', frameCount: 16, invalidAlpha: 0 },
  ])
  for (const appearance of metrics) {
    expect(appearance.opaquePixels).toHaveLength(16)
    expect(appearance.opaquePixels.every((pixels) => pixels > 0), JSON.stringify(appearance)).toBe(true)
    expect(appearance.distinctFamilies.every(({ unique }) => unique),
      JSON.stringify(appearance.distinctFamilies)).toBe(true)
    expect(appearance.colors, JSON.stringify(appearance)).toBeGreaterThanOrEqual(8)
    expect(appearance.colors, JSON.stringify(appearance)).toBeLessThanOrEqual(48)
    if (appearance.appearance === 'three' || appearance.appearance === 'four') {
      expect(appearance.coatPixels, JSON.stringify(appearance)).toBeGreaterThan(100)
    }
  }
})

test('workbench Export button downloads the validated runtime pack', async ({ page }) => {
  await openWorkbench(page)
  const downloads: Download[] = []
  page.on('download', (download) => downloads.push(download))
  await page.locator('#export').click()
  await expect(page.locator('#status')).toHaveText('Runtime files prepared for download.')
  await expect.poll(() => downloads.length).toBe(4)
  expect(downloads.map((download) => download.suggestedFilename()).sort()).toEqual([
    'arena-atlas.json', 'arena-atlas.png', 'arena-overview.png', 'arena-preview.png',
  ])
  for (const download of downloads) expect(await download.failure()).toBeNull()
})

test('concurrent exports and reset serialize different sources without mixing files or preview ownership', async ({ page }) => {
  const initial = await openWorkbench(page)
  const result = await page.evaluate(async () => {
    const api = window.grenadiersArtAtlas
    const [firstSource, secondSource] = await Promise.all([
      api.createEditedTemplate({ x: 1, y: 1, rgba: [255, 0, 255, 255] }),
      api.createEditedTemplate({ x: 113, y: 1, rgba: [0, 255, 255, 255] }),
    ])
    const [first, second, reset] = await Promise.all([
      api.exportSource(firstSource), api.exportSource(secondSource), api.resetPreview(),
    ])
    const atlas = (exported: ExportResult): string | undefined =>
      exported.files.find(({ name }) => name === 'arena-atlas.png')?.base64
    return { firstAtlas: atlas(first), secondAtlas: atlas(second), reset, final: api.readState() }
  })
  expect(result.firstAtlas).toBeTruthy()
  expect(result.secondAtlas).toBeTruthy()
  expect(result.firstAtlas).not.toBe(result.secondAtlas)
  expect(result.reset.canvasCount).toBe(1)
  expect(result.final).toEqual(result.reset)
  expect(result.final.resources).toEqual({
    created: initial.resources.created + 3,
    destroyed: initial.resources.destroyed + 3,
    live: 1,
  })
})

test('invalid upload remains visible while language and selected asset details switch to Polish', async ({ page }) => {
  await openWorkbench(page)
  await page.locator('#inventory button').first().click()
  const template = await page.evaluate(() => window.grenadiersArtAtlas.createTemplate())
  const png = template.files.find(({ name }) => name === 'arena-source.png')?.base64
  const manifestText = template.files.find(({ name }) => name === 'arena-source.json')?.text
  if (!png || !manifestText) throw new Error('Template files are incomplete')
  const manifest = JSON.parse(manifestText) as { frames: Array<{ id: string }> }
  manifest.frames[0].id = '../invalid'
  await page.locator('#source-file').setInputFiles({
    name: 'arena-source.png', mimeType: 'image/png', buffer: Buffer.from(png, 'base64'),
  })
  await page.locator('#manifest-file').setInputFiles({
    name: 'arena-source.json', mimeType: 'application/json', buffer: Buffer.from(JSON.stringify(manifest)),
  })
  await page.locator('#load-files').click()
  await expect(page.locator('#status')).toContainText('invalid stable ID ../invalid')
  const englishError = await page.locator('#status').textContent()
  await page.selectOption('#language', 'pl')
  await expect(page.locator('#status')).toHaveText(englishError ?? '')
  await expect(page.locator('#selection-detail')).toContainText('bazowy')
  await expect(page.locator('#status')).not.toHaveText('Szablon bazowy jest gotowy.')
})

test('workbench captures desktop and 320px reduced-motion proofs with one bounded RGBA texture',
  async ({ browser, page }, testInfo) => {
    const desktop = await openWorkbench(page)
    expect(desktop.canvasCount).toBe(1)
    expect(desktop.texture).not.toBeNull()
    expect(desktop.texture?.rgbaBytes)
      .toBe((desktop.texture?.width ?? 0) * (desktop.texture?.height ?? 0) * 4)
    expect(desktop.texture?.rgbaBytes ?? Number.POSITIVE_INFINITY).toBeLessThanOrEqual(16 * 1024 * 1024)
    const timing = await page.evaluate(() => window.grenadiersArtAtlas.measureRenders(30))
    expect(timing.count).toBe(30)
    expect(timing.elapsedMs).toBeLessThan(2_000)
    await page.screenshot({ path: testInfo.outputPath('art-atlas-desktop.png'), fullPage: true })

    const context = await browser.newContext({
      viewport: { width: 320, height: 844 }, reducedMotion: 'reduce', serviceWorkers: 'block',
    })
    try {
      const mobilePolicy = await observeBrowserPolicy(context)
      const mobile = await context.newPage()
      await openWorkbench(mobile)
      expect(await mobile.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true)
      await mobile.selectOption('#language', 'pl')
      await expect(mobile.locator('h1')).toHaveText('Stół rysunkowy dziedzińca')
      await expect(mobile.locator('#source-canvas')).not.toHaveAttribute('tabindex')
      await expect(mobile.locator('#source-canvas'))
        .toHaveAttribute('aria-label', 'Edytowalny przezroczysty arkusz źródłowy grafiki')
      await mobile.screenshot({ path: testInfo.outputPath('art-atlas-mobile-pl-reduced.png'), fullPage: true })
      expect(mobilePolicy.violations).toEqual([])
    } finally {
      await context.close()
    }
  })
