import { expect, test } from '@playwright/test'
import {
  assertCleanAudit, beginAudit, canvasScreenshot, openFixture, pixelDifference,
  type BoardName, type BrowserAudit,
} from './three-arena-api'

const variants = ['zoomIn', 'zoomOut', 'rotateLeft', 'rotateRight'] as const
let audit: BrowserAudit | undefined
test.beforeEach(async ({ context, page }) => { audit = await beginAudit(context, page) })
test.afterEach(() => { if (audit) assertCleanAudit(audit) })

test('round camera openings and endings animate every variant and settle to identical native pixels', async ({ page }, info) => {
  test.skip(info.project.name.includes('reduced'), 'animated desktop proof')
  await openFixture(page)
  for (const boundary of ['start', 'end'] as const) {
    for (const variant of variants) {
      await page.evaluate((edge) => window.grenadiersThreeArena.prepareRoundCamera('large', edge), boundary)
      const baseline = await canvasScreenshot(page, info, `camera-${boundary}-${variant}-baseline`)
      await page.evaluate((kind) => window.grenadiersThreeArena.beginRoundCamera(kind), variant)
      await page.evaluate(() => window.grenadiersThreeArena.sampleRoundCamera(350))
      const moving = await canvasScreenshot(page, info, `camera-${boundary}-${variant}-moving`)
      expect((await pixelDifference(page, baseline, moving)).changed).toBeGreaterThan(100)
      await page.evaluate(() => window.grenadiersThreeArena.sampleRoundCamera(950))
      const settled = await canvasScreenshot(page, info, `camera-${boundary}-${variant}-settled`)
      expect(settled.equals(baseline), `${boundary} ${variant} returns exactly to the canonical camera`).toBe(true)
    }
  }
})

test('round camera openings survive live snapshots and clear on suspension and WebGL recovery', async ({ page }, info) => {
  test.skip(info.project.name.includes('reduced'), 'animated lifecycle proof')
  await openFixture(page)
  await page.evaluate(() => window.grenadiersThreeArena.prepareRoundCamera('large', 'start'))
  const baseline = await canvasScreenshot(page, info, 'camera-lifecycle-baseline')
  await page.evaluate(() => window.grenadiersThreeArena.beginRoundCamera('rotateRight'))
  await page.evaluate(() => window.grenadiersThreeArena.sampleRoundCamera(200))
  const moving = await canvasScreenshot(page, info, 'camera-lifecycle-moving')
  expect(moving.equals(baseline)).toBe(false)
  await page.evaluate(() => window.grenadiersThreeArena.enterRoundCameraRunning())
  expect((await canvasScreenshot(page, info, 'camera-live-running')).equals(moving)).toBe(true)
  await page.evaluate(() => window.grenadiersThreeArena.sampleRoundCamera(950))
  expect((await canvasScreenshot(page, info, 'camera-live-running-later')).equals(baseline)).toBe(true)

  await page.evaluate(() => window.grenadiersThreeArena.prepareRoundCamera('large', 'end'))
  await page.evaluate(() => window.grenadiersThreeArena.beginRoundCamera('zoomIn'))
  await page.evaluate(() => window.grenadiersThreeArena.sampleRoundCamera(450))
  const restoration = await page.evaluate(() => window.grenadiersThreeArena.loseAndRestoreContext())
  expect(restoration.supported).toBe(true)
  expect(restoration.restored).toMatchObject({ canvasCount: 1, canvasVisible: true })
  expect((await canvasScreenshot(page, info, 'camera-restored-neutral')).equals(baseline)).toBe(true)
  await page.evaluate(() => window.grenadiersThreeArena.suspend())
  await page.evaluate(() => window.grenadiersThreeArena.enterRoundCameraRunning())
  expect((await canvasScreenshot(page, info, 'camera-resumed-neutral')).equals(baseline)).toBe(true)
})

test('reduced motion keeps all round camera variants static on every board', async ({ page }, info) => {
  test.skip(!info.project.name.includes('reduced'), 'reduced-motion proof')
  await openFixture(page)
  for (const board of ['small', 'medium', 'large'] as BoardName[]) {
    for (const boundary of ['start', 'end'] as const) {
      for (const variant of variants) {
        await page.evaluate(({ board, boundary }) => window.grenadiersThreeArena.prepareRoundCamera(board, boundary), { board, boundary })
        const baseline = await page.locator('#arena canvas').screenshot()
        await page.evaluate((kind) => window.grenadiersThreeArena.beginRoundCamera(kind), variant)
        await page.evaluate(() => window.grenadiersThreeArena.sampleRoundCamera(450))
        const still = await page.locator('#arena canvas').screenshot()
        expect(still.equals(baseline), `${board} ${boundary} ${variant} reduced motion`).toBe(true)
      }
    }
  }
  await canvasScreenshot(page, info, 'camera-reduced-motion-static')
})
