import { expect, type BrowserContext, type Page, type TestInfo } from '@playwright/test'
import { observeBrowserPolicy, type BrowserPolicyObservation } from '../tests/support/browser-policy'

export type BoardName = 'small' | 'medium' | 'large'
export type SceneName = 'empty' | 'actorBomb' | 'showcase' | 'teleportA' | 'teleportB' | 'newRoom'
export type DeathAppearance = 'one' | 'two' | 'three' | 'four' | 'mob'
export type DeathOverlay = 'none' | 'flame' | 'burningOil'
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

export async function deathPixelRetention(
  page: Page,
  baseline: Buffer,
  deathOnly: Buffer,
  combined: Buffer,
): Promise<{ deathPixels: number; retainedPixels: number; ratio: number }> {
  return page.evaluate(async ({ baselinePng, deathPng, combinedPng }) => {
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
    const [empty, death, overlap] = await Promise.all([
      pixels(baselinePng), pixels(deathPng), pixels(combinedPng),
    ])
    let deathPixels = 0
    let retainedPixels = 0
    for (let offset = 0; offset < empty.length; offset += 4) {
      const deathDelta = Math.abs(death[offset] - empty[offset])
        + Math.abs(death[offset + 1] - empty[offset + 1])
        + Math.abs(death[offset + 2] - empty[offset + 2])
      if (deathDelta <= 12) continue
      deathPixels += 1
      const retainedDelta = Math.abs(death[offset] - overlap[offset])
        + Math.abs(death[offset + 1] - overlap[offset + 1])
        + Math.abs(death[offset + 2] - overlap[offset + 2])
      if (retainedDelta <= 8) retainedPixels += 1
    }
    return { deathPixels, retainedPixels, ratio: retainedPixels / deathPixels }
  }, {
    baselinePng: baseline.toString('base64'),
    deathPng: deathOnly.toString('base64'),
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

export function expectDifferent(left: Buffer, right: Buffer, message: string): void {
  expect(left.equals(right), message).toBe(false)
}
