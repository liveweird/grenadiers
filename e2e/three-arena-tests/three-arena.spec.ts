import { expect, test } from '@playwright/test'
import {
  assertCleanAudit, beginAudit, canvasScreenshot, deathPixelRetention, expectDifferent,
  openFixture, show, showDeath, showDeathOverlap, settleDeathAge,
  type BoardName, type BrowserAudit, type DeathAppearance, type DeathOverlay,
} from './three-arena-api'

const deathAppearances: readonly DeathAppearance[] = ['one', 'two', 'three', 'four', 'mob']

let audit: BrowserAudit | undefined
test.beforeEach(async ({ context, page }) => {
  audit = undefined
  audit = await beginAudit(context, page)
})
test.afterEach(() => { if (audit) assertCleanAudit(audit) })

test('production GPU finish pass preserves sRGB midtones and separates primary colors', async ({ page }, testInfo) => {
  await openFixture(page)
  const samples = await page.evaluate(() => window.grenadiersThreeArena.calibrateColors())
  const screenshotPath = testInfo.outputPath('production-gpu-color-calibration.png')
  await page.locator('[data-calibration="production-finish-pass"]').screenshot({ path: screenshotPath })
  await testInfo.attach('production-gpu-color-calibration', {
    path: screenshotPath, contentType: 'image/png',
  })

  expect(samples.gray.alpha).toBe(255)
  for (const channel of [samples.gray.red, samples.gray.green, samples.gray.blue]) {
    expect(channel, JSON.stringify(samples)).toBeGreaterThanOrEqual(110)
    expect(channel, JSON.stringify(samples)).toBeLessThanOrEqual(155)
  }
  expect(Math.max(samples.gray.red, samples.gray.green, samples.gray.blue)
    - Math.min(samples.gray.red, samples.gray.green, samples.gray.blue)).toBeLessThanOrEqual(16)
  expect(samples.red.red).toBeGreaterThan(samples.red.green * 2)
  expect(samples.red.red).toBeGreaterThan(samples.red.blue * 2)
  expect(samples.green.green).toBeGreaterThan(samples.green.red * 2)
  expect(samples.green.green).toBeGreaterThan(samples.green.blue * 2)
  expect(samples.blue.blue).toBeGreaterThan(samples.blue.red * 2)
  expect(samples.blue.blue).toBeGreaterThan(samples.blue.green * 2)
})

test('native camera frames all board sizes and the complete public entity showcase', async ({ page }, testInfo) => {
  const initial = await openFixture(page)
  expect(initial).toMatchObject({ canvasCount: 1, canvasVisible: false, destroyed: false })

  for (const board of ['small', 'medium', 'large'] satisfies BoardName[]) {
    const baselineState = await show(page, board, 'empty')
    expect(baselineState.reducedMotion).toBe(testInfo.project.name.includes('reduced'))
    const baseline = await canvasScreenshot(page, testInfo, `${board}-baseline`)
    await show(page, board, 'showcase')
    const showcase = await canvasScreenshot(page, testInfo, `${board}-showcase`)
    expectDifferent(showcase, baseline, `${board} showcase must alter production WebGL pixels`)
  }
})

test('projected actor and bomb pixels revert exactly when public rows are removed', async ({ page }, testInfo) => {
  await openFixture(page)
  await show(page, 'large', 'empty')
  const baseline = await canvasScreenshot(page, testInfo, 'entity-removal-baseline')
  const projected = await page.evaluate(() => ({
    actor: window.grenadiersThreeArena.projectCell('large', 1, 1),
    bomb: window.grenadiersThreeArena.projectCell('large', 3, 1),
  }))
  expect(projected.actor.x).toBeGreaterThan(0)
  expect(projected.actor.y).toBeGreaterThan(0)
  expect(projected.bomb.x).toBeGreaterThan(projected.actor.x)

  await show(page, 'large', 'actorBomb')
  const populated = await canvasScreenshot(page, testInfo, 'projected-actor-bomb')
  expectDifferent(populated, baseline, 'canonical actor/bomb atlas cards must change projected pixels')

  await show(page, 'large', 'empty')
  const removed = await canvasScreenshot(page, testInfo, 'entity-removal-restored')
  expect(removed.equals(baseline), 'removed entities must leave no pixels or shadows').toBe(true)
})

test('native death artwork advances every appearance and expires without lingering pixels', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name.includes('reduced'), 'normal-motion production timing')
  await openFixture(page)
  await show(page, 'large', 'empty')
  const baseline = await canvasScreenshot(page, testInfo, 'death-empty-baseline')

  for (const [index, appearance] of deathAppearances.entries()) {
    const bornAt = await showDeath(page, appearance, index)
    await settleDeathAge(page, bornAt, 75)
    const before = await canvasScreenshot(page, testInfo, `${appearance}-death-before-frame-0`)
    await settleDeathAge(page, bornAt, 225)
    const middleOne = await canvasScreenshot(page, testInfo, `${appearance}-death-middle-frame-1`)
    await settleDeathAge(page, bornAt, 375)
    const middleTwo = await canvasScreenshot(page, testInfo, `${appearance}-death-middle-frame-2`)
    await settleDeathAge(page, bornAt, 525)
    const final = await canvasScreenshot(page, testInfo, `${appearance}-death-final-frame-3`)
    await settleDeathAge(page, bornAt, 650)
    const expired = await canvasScreenshot(page, testInfo, `${appearance}-death-expired`)

    expectDifferent(before, baseline, `${appearance} death frame zero must alter native pixels`)
    expectDifferent(middleOne, before, `${appearance} death animation must advance to frame one`)
    expectDifferent(middleTwo, middleOne, `${appearance} death animation must advance to frame two`)
    expectDifferent(final, middleTwo, `${appearance} death animation must advance to frame three`)
    expect(expired.equals(baseline), `${appearance} death must leave no corpse or shadow`).toBe(true)
  }
})

test('reduced motion holds every death on frame three then expires from the same birth time', async ({ page }, testInfo) => {
  test.skip(!testInfo.project.name.includes('reduced'), 'reduced-motion production timing')
  await openFixture(page)
  await show(page, 'large', 'empty')
  const baseline = await canvasScreenshot(page, testInfo, 'reduced-death-empty-baseline')

  for (const [index, appearance] of deathAppearances.entries()) {
    const bornAt = await showDeath(page, appearance, index)
    await settleDeathAge(page, bornAt, 30)
    const staticStart = await canvasScreenshot(page, testInfo, `${appearance}-reduced-frame-3-start`)
    await settleDeathAge(page, bornAt, 90)
    const staticEnd = await canvasScreenshot(page, testInfo, `${appearance}-reduced-frame-3-held`)
    await settleDeathAge(page, bornAt, 225)
    const expired = await canvasScreenshot(page, testInfo, `${appearance}-reduced-expired`)

    expectDifferent(staticStart, baseline, `${appearance} reduced death must show frame three`)
    expect(staticEnd.equals(staticStart), `${appearance} reduced death must remain static`).toBe(true)
    expect(expired.equals(baseline), `${appearance} reduced death must expire without residue`).toBe(true)
  }
})

test('native death body remains readable over ordinary and burning-oil flames', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name.includes('reduced'), 'normal-motion overlap pixels')
  await openFixture(page)
  await show(page, 'large', 'empty')
  const baseline = await canvasScreenshot(page, testInfo, 'death-overlap-empty-baseline')

  for (const [generation, overlay] of (['flame', 'burningOil'] satisfies DeathOverlay[]).entries()) {
    const bornAt = await showDeathOverlap(page, 'none', true, generation, null)
    await settleDeathAge(page, bornAt, 75)
    const deathOnly = await canvasScreenshot(page, testInfo, `${overlay}-death-only`)
    await showDeathOverlap(page, overlay, false, generation, bornAt)
    const flameOnly = await canvasScreenshot(page, testInfo, `${overlay}-only`)
    await showDeathOverlap(page, overlay, true, generation, bornAt)
    await settleDeathAge(page, bornAt, 75)
    const combined = await canvasScreenshot(page, testInfo, `${overlay}-death-combined`)
    const retention = await deathPixelRetention(page, baseline, deathOnly, combined)
    await testInfo.attach(`${overlay}-death-retention`, {
      body: JSON.stringify(retention), contentType: 'application/json',
    })

    expectDifferent(deathOnly, baseline, 'death-only control must expose native body pixels')
    expectDifferent(flameOnly, baseline, `${overlay} control must expose native flame pixels`)
    expectDifferent(combined, flameOnly, `death body must remain visible over ${overlay}`)
    expectDifferent(combined, deathOnly, `${overlay} must remain visible around the death body`)
    expect(retention.deathPixels, 'death-only mask must contain meaningful pixels').toBeGreaterThan(100)
    expect(retention.ratio, `${overlay} retained ${JSON.stringify(retention)}`).toBeGreaterThanOrEqual(0.75)
  }
})

test('teleport, suspend, room replacement, context restoration, and destroy leave no ghosts', async ({ page }, testInfo) => {
  await openFixture(page)
  await show(page, 'large', 'empty')
  const baseline = await canvasScreenshot(page, testInfo, 'lifecycle-baseline')

  await show(page, 'large', 'teleportA')
  const beforeTeleport = await canvasScreenshot(page, testInfo, 'teleport-before')
  await show(page, 'large', 'teleportB')
  const afterTeleport = await canvasScreenshot(page, testInfo, 'teleport-after')
  expectDifferent(beforeTeleport, afterTeleport, 'teleport must reappear at the new projected position')

  const suspended = await page.evaluate(() => window.grenadiersThreeArena.suspend())
  expect(suspended).toMatchObject({ scene: null, canvasCount: 1, canvasVisible: false })
  await show(page, 'large', 'newRoom')
  const newRoom = await canvasScreenshot(page, testInfo, 'new-room-after-suspend')
  expect(newRoom.equals(baseline), 'new room must resume without prior actor history').toBe(true)

  await show(page, 'large', 'showcase')
  const beforeContextLoss = await canvasScreenshot(page, testInfo, 'context-before')
  const context = await page.evaluate(() => window.grenadiersThreeArena.loseAndRestoreContext())
  if (context.supported) {
    expect(context.lost.canvasVisible).toBe(false)
    expect(context.restored).toMatchObject({ canvasCount: 1, canvasVisible: true })
    const afterContextRestore = await canvasScreenshot(page, testInfo, 'context-restored')
    expectDifferent(afterContextRestore, baseline, 'restored context must redraw the nonempty showcase')
    expect(beforeContextLoss.equals(baseline), 'pre-loss showcase must not equal its empty baseline').toBe(false)
  }

  const destroyed = await page.evaluate(() => window.grenadiersThreeArena.destroy())
  expect(destroyed).toMatchObject({ destroyed: true, scene: null, canvasCount: 0 })
  expect(destroyed.pendingAnimationFrames, 'renderer-owned animation frame must be cancelled').toBe(0)
})
