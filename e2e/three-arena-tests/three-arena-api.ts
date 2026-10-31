import { expect, type BrowserContext, type Page, type TestInfo } from '@playwright/test'
import { observeBrowserPolicy, type BrowserPolicyObservation } from '../tests/support/browser-policy'

export type BoardName = 'small' | 'medium' | 'large'
export type SceneName = 'empty' | 'actorBomb' | 'showcase' | 'teleportA' | 'teleportB' | 'newRoom'
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
export type ColorSample = { red: number; green: number; blue: number; alpha: number }
export type CalibrationSamples = {
  red: ColorSample
  green: ColorSample
  blue: ColorSample
  gray: ColorSample
}
type FixtureApi = {
  initialize(): Promise<FixtureState>
  show(board: BoardName, scene: SceneName): Promise<FixtureState>
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

export async function canvasScreenshot(
  page: Page,
  testInfo: TestInfo,
  name: string,
): Promise<Buffer> {
  const image = await page.locator('#arena canvas').screenshot({ animations: 'disabled' })
  await testInfo.attach(name, { body: image, contentType: 'image/png' })
  return image
}

export function expectDifferent(left: Buffer, right: Buffer, message: string): void {
  expect(left.equals(right), message).toBe(false)
}
