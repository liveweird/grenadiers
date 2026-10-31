import { expect, type BrowserContext, type Page, type TestInfo } from '@playwright/test'
import { observeBrowserPolicy, type BrowserPolicyObservation } from '../tests/support/browser-policy'

export type BoardName = 'small' | 'medium' | 'large'
export type SceneName =
  | 'empty' | 'actorBomb' | 'showcase' | 'teleportA' | 'teleportB' | 'newRoom'
  | 'actorOnly' | 'remoteActor' | 'lairOnly'
  | 'spikeIdle' | 'spikeWarning' | 'spikeRaised'
export type DeathAppearance = 'one' | 'two' | 'three' | 'four' | 'mob'
export type DeathOverlay = 'none' | 'flame' | 'burningOil'
export type BloodLayerMode = 'subjectOnly' | 'poolOnly' | 'combined'
export type FireScene =
  | 'empty' | 'single' | 'burningOil' | 'bombOnly' | 'concealed'
  | 'actorOnly' | 'actorNear' | 'transitionBase' | 'transitionAdded' | 'worstCase'
export type FireReset = 'clear' | 'round' | 'admission' | 'backward' | 'preferences'
export type FrameCost = { samples: number; medianMs: number; p95Ms: number; maxMs: number }
export type FixtureState = {
  board: BoardName
  scene: SceneName | null
  destroyed: boolean
  reducedMotion: boolean
  canvasCount: number
  canvasVisible: boolean
  pendingAnimationFrames: number
  contextLossSupported: boolean
}
type ColorSample = { red: number; green: number; blue: number; alpha: number }
type CalibrationSamples = {
  red: ColorSample
  green: ColorSample
  blue: ColorSample
  gray: ColorSample
}
type FixtureApi = {
  initialize(): Promise<FixtureState>
  show(board: BoardName, scene: SceneName): Promise<FixtureState>
  showDeath(board: BoardName, appearance: DeathAppearance, index: number): Promise<{
    state: FixtureState
    bornAt: number
  }>
  settleDeathAge(bornAt: number, ageMs: number): Promise<number>
  showPoolOnly(board: BoardName, appearance: DeathAppearance, index: number): Promise<{
    state: FixtureState
    bornAt: number
  }>
  replayDeath(
    board: BoardName,
    appearance: DeathAppearance,
    index: number,
    bornAt: number,
  ): Promise<FixtureState>
  resetDeathPresentation(board: BoardName, index: number): Promise<FixtureState>
  showBloodLayers(
    mode: BloodLayerMode,
    generation: number,
    bornAt: number | null,
  ): Promise<{ state: FixtureState; bornAt: number }>
  showDeathOverlap(
    overlay: DeathOverlay,
    includeDeath: boolean,
    generation: number,
    bornAt: number | null,
  ): Promise<{ state: FixtureState; bornAt: number }>
  suspend(): Promise<FixtureState>
  destroy(): Promise<FixtureState>
  readState(): FixtureState
  projectCell(board: BoardName, column: number, row: number): { x: number; y: number }
  loseAndRestoreContext(): Promise<{ supported: boolean; lost: FixtureState; restored: FixtureState }>
  calibrateColors(): Promise<CalibrationSamples>
  showFire(scene: FireScene): Promise<FixtureState>
  beginFireTransition(): Promise<{ state: FixtureState; bornAt: number }>
  settleFireAge(bornAt: number, ageMs: number): Promise<number>
  resetFire(reason: FireReset): Promise<FixtureState>
  measureFireCost(scene: 'empty' | 'worstCase'): Promise<FrameCost>
  projectPoint(
    board: BoardName, column: number, row: number, elevation: number,
  ): { x: number; y: number }
}

declare global {
  interface Window { grenadiersThreeArena: FixtureApi }
}

export type BrowserAudit = {
  policy: BrowserPolicyObservation
  faults: string[]
}

export async function beginAudit(context: BrowserContext, page: Page): Promise<BrowserAudit> {
  const policy = await observeBrowserPolicy(context)
  const faults: string[] = []
  page.on('pageerror', (error) => faults.push(`pageerror: ${error.message}`))
  page.on('console', (message) => {
    if (message.type() === 'error') faults.push(`console: ${message.text()}`)
  })
  return { policy, faults }
}

export function assertCleanAudit(audit: BrowserAudit): void {
  expect(audit.policy.violations, 'CSP violations').toEqual([])
  expect(audit.faults, 'page, WebGL, or shader errors').toEqual([])
}

export async function openFixture(page: Page): Promise<FixtureState> {
  await page.goto('/three-arena.html')
  await expect(page.locator('[data-build-target]'))
    .toHaveAttribute('data-build-target', 'grenadiers-three-arena-diagnostic')
  await expect.poll(() => page.evaluate(() => Boolean(window.grenadiersThreeArena))).toBe(true)
  return page.evaluate(() => window.grenadiersThreeArena.initialize())
}

export async function show(
  page: Page,
  board: BoardName,
  scene: SceneName,
): Promise<FixtureState> {
  const state = await page.evaluate(
    ({ board: nextBoard, scene: nextScene }) => window.grenadiersThreeArena.show(nextBoard, nextScene),
    { board, scene },
  )
  expect(state).toMatchObject({ board, scene, canvasCount: 1, canvasVisible: true, destroyed: false })
  return state
}

export async function showDeath(
  page: Page,
  appearance: DeathAppearance,
  index: number,
): Promise<number> {
  const result = await page.evaluate(
    ({ deathAppearance, deathIndex }) => window.grenadiersThreeArena
      .showDeath('large', deathAppearance, deathIndex),
    { deathAppearance: appearance, deathIndex: index },
  )
  expect(result.state).toMatchObject({
    board: 'large', scene: null, canvasCount: 1, canvasVisible: true, destroyed: false,
  })
  return result.bornAt
}

export async function settleDeathAge(page: Page, bornAt: number, ageMs: number): Promise<void> {
  const age = await page.evaluate(
    ({ cueBornAt, targetAge }) => window.grenadiersThreeArena
      .settleDeathAge(cueBornAt, targetAge),
    { cueBornAt: bornAt, targetAge: ageMs },
  )
  expect(age).toBeGreaterThanOrEqual(ageMs)
}

export async function showPoolOnly(
  page: Page,
  appearance: DeathAppearance,
  index: number,
): Promise<number> {
  const result = await page.evaluate(
    ({ cueAppearance, cueIndex }) => window.grenadiersThreeArena
      .showPoolOnly('large', cueAppearance, cueIndex),
    { cueAppearance: appearance, cueIndex: index },
  )
  expect(result.state).toMatchObject({ canvasCount: 1, canvasVisible: true, destroyed: false })
  return result.bornAt
}

export async function replayDeath(
  page: Page,
  appearance: DeathAppearance,
  index: number,
  bornAt: number,
): Promise<void> {
  await page.evaluate(
    (input) => window.grenadiersThreeArena
      .replayDeath('large', input.appearance, input.index, input.bornAt),
    { appearance, index, bornAt },
  )
}

export async function resetDeathPresentation(page: Page, index: number): Promise<void> {
  await page.evaluate(
    (cueIndex) => window.grenadiersThreeArena.resetDeathPresentation('large', cueIndex),
    index,
  )
}

export async function showBloodLayers(
  page: Page,
  mode: BloodLayerMode,
  generation: number,
  bornAt: number | null,
): Promise<number> {
  const result = await page.evaluate(
    (input) => window.grenadiersThreeArena
      .showBloodLayers(input.mode, input.generation, input.bornAt),
    { mode, generation, bornAt },
  )
  expect(result.state).toMatchObject({ canvasCount: 1, canvasVisible: true, destroyed: false })
  return result.bornAt
}

export async function showDeathOverlap(
  page: Page,
  overlay: DeathOverlay,
  includeDeath: boolean,
  generation: number,
  bornAt: number | null,
): Promise<number> {
  const result = await page.evaluate(
    (input) => window.grenadiersThreeArena.showDeathOverlap(
      input.overlay, input.includeDeath, input.generation, input.bornAt,
    ),
    { overlay, includeDeath, generation, bornAt },
  )
  expect(result.state).toMatchObject({
    board: 'large', scene: null, canvasCount: 1, canvasVisible: true, destroyed: false,
  })
  return result.bornAt
}

export async function showFire(page: Page, scene: FireScene): Promise<FixtureState> {
  const state = await page.evaluate((nextScene) => window.grenadiersThreeArena.showFire(nextScene), scene)
  expect(state).toMatchObject({ board: 'large', canvasCount: 1, canvasVisible: true, destroyed: false })
  return state
}

export async function beginFireTransition(page: Page): Promise<number> {
  const result = await page.evaluate(() => window.grenadiersThreeArena.beginFireTransition())
  expect(result.state).toMatchObject({ canvasCount: 1, canvasVisible: true, destroyed: false })
  return result.bornAt
}

export async function settleFireAge(page: Page, bornAt: number, ageMs: number): Promise<void> {
  const age = await page.evaluate(
    (input) => window.grenadiersThreeArena.settleFireAge(input.bornAt, input.ageMs),
    { bornAt, ageMs },
  )
  expect(age).toBe(ageMs)
}

export async function resetFire(page: Page, reason: FireReset): Promise<FixtureState> {
  return page.evaluate((nextReason) => window.grenadiersThreeArena.resetFire(nextReason), reason)
}

export async function measureFireCost(
  page: Page, scene: 'empty' | 'worstCase',
): Promise<FrameCost> {
  return page.evaluate((nextScene) => window.grenadiersThreeArena.measureFireCost(nextScene), scene)
}

export async function pixelRetention(
  page: Page,
  baseline: Buffer,
  subjectOnly: Buffer,
  combined: Buffer,
): Promise<{ subjectPixels: number; retainedPixels: number; addedPixels: number; ratio: number }> {
  return page.evaluate(async ({ baselinePng, subjectPng, combinedPng }) => {
    const pixels = async (base64: string): Promise<Uint8ClampedArray> => {
      const image = new Image()
      image.src = `data:image/png;base64,${base64}`
      await image.decode()
      const canvas = document.createElement('canvas')
      canvas.width = image.naturalWidth
      canvas.height = image.naturalHeight
      const context = canvas.getContext('2d')
      if (!context) throw new Error('2D screenshot comparison context is unavailable')
      context.drawImage(image, 0, 0)
      return context.getImageData(0, 0, canvas.width, canvas.height).data
    }
    const [empty, subject, overlap] = await Promise.all([
      pixels(baselinePng), pixels(subjectPng), pixels(combinedPng),
    ])
    let subjectPixels = 0
    let retainedPixels = 0
    let addedPixels = 0
    for (let offset = 0; offset < empty.length; offset += 4) {
      const subjectDelta = Math.abs(subject[offset] - empty[offset])
        + Math.abs(subject[offset + 1] - empty[offset + 1])
        + Math.abs(subject[offset + 2] - empty[offset + 2])
      const overlapDelta = Math.abs(overlap[offset] - empty[offset])
        + Math.abs(overlap[offset + 1] - empty[offset + 1])
        + Math.abs(overlap[offset + 2] - empty[offset + 2])
      if (subjectDelta > 12) {
        subjectPixels += 1
        const retainedDelta = Math.abs(subject[offset] - overlap[offset])
          + Math.abs(subject[offset + 1] - overlap[offset + 1])
          + Math.abs(subject[offset + 2] - overlap[offset + 2])
        if (retainedDelta <= 8) retainedPixels += 1
      } else if (overlapDelta > 12) addedPixels += 1
    }
    return { subjectPixels, retainedPixels, addedPixels, ratio: retainedPixels / subjectPixels }
  }, {
    baselinePng: baseline.toString('base64'),
    subjectPng: subjectOnly.toString('base64'),
    combinedPng: combined.toString('base64'),
  })
}

export async function canvasScreenshot(
  page: Page,
  testInfo: TestInfo,
  name: string,
): Promise<Buffer> {
  const screenshotPath = testInfo.outputPath(`${name}.png`)
  const image = await page.locator('#arena canvas').screenshot({
    animations: 'disabled', path: screenshotPath,
  })
  await testInfo.attach(name, { path: screenshotPath, contentType: 'image/png' })
  return image
}

export async function canvasRegionScreenshot(
  page: Page,
  testInfo: TestInfo,
  name: string,
  point: { x: number; y: number },
  radius: number,
): Promise<Buffer> {
  const bounds = await page.locator('#arena canvas').boundingBox()
  if (!bounds) throw new Error('Three arena canvas bounds are unavailable')
  const clip = {
    x: Math.max(bounds.x, bounds.x + point.x - radius),
    y: Math.max(bounds.y, bounds.y + point.y - radius),
    width: Math.min(radius * 2, bounds.width),
    height: Math.min(radius * 2, bounds.height),
  }
  const path = testInfo.outputPath(`${name}.png`)
  const image = await page.screenshot({ animations: 'disabled', clip, path })
  await testInfo.attach(name, { path, contentType: 'image/png' })
  return image
}

export async function pixelDifference(
  page: Page,
  before: Buffer,
  after: Buffer,
): Promise<{ changed: number; red: number; green: number; blue: number }> {
  return page.evaluate(async ({ beforePng, afterPng }) => {
    const pixels = async (encoded: string): Promise<Uint8ClampedArray> => {
      const image = new Image()
      image.src = `data:image/png;base64,${encoded}`
      await image.decode()
      const canvas = document.createElement('canvas')
      canvas.width = image.naturalWidth
      canvas.height = image.naturalHeight
      const context = canvas.getContext('2d')
      if (!context) throw new Error('Fire pixel comparison context is unavailable')
      context.drawImage(image, 0, 0)
      return context.getImageData(0, 0, canvas.width, canvas.height).data
    }
    const [left, right] = await Promise.all([pixels(beforePng), pixels(afterPng)])
    let changed = 0; let red = 0; let green = 0; let blue = 0
    for (let offset = 0; offset < left.length; offset += 4) {
      const dr = right[offset] - left[offset]
      const dg = right[offset + 1] - left[offset + 1]
      const db = right[offset + 2] - left[offset + 2]
      if (Math.abs(dr) + Math.abs(dg) + Math.abs(db) > 9) changed += 1
      red += dr; green += dg; blue += db
    }
    return { changed, red, green, blue }
  }, { beforePng: before.toString('base64'), afterPng: after.toString('base64') })
}

export function expectDifferent(left: Buffer, right: Buffer, message: string): void {
  expect(left.equals(right), message).toBe(false)
}
