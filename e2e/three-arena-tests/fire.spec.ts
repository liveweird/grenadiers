import { expect, test } from '@playwright/test'
import {
  assertCleanAudit, beginAudit, beginFireTransition, canvasRegionScreenshot, canvasScreenshot,
  measureFireCost, openFixture, pixelDifference, resetFire, settleFireAge, showFire,
  type BrowserAudit, type FireReset,
} from './three-arena-api'

let audit: BrowserAudit | undefined
test.beforeEach(async ({ context, page }) => {
  audit = undefined
  audit = await beginAudit(context, page)
})
test.afterEach(() => { if (audit) assertCleanAudit(audit) })

async function point(
  page: Parameters<typeof showFire>[0], column: number, row: number, elevation: number,
): Promise<{ x: number; y: number }> {
  return page.evaluate(
    (input) => window.grenadiersThreeArena
      .projectPoint('large', input.column, input.row, input.elevation),
    { column, row, elevation },
  )
}

test('native public fire warms nearby floor walls and actors without bomb inference', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name.includes('reduced'), 'desktop illumination pixel proof')
  await openFixture(page)
  const floor = await point(page, 6, 5, 0.02)
  const wall = await point(page, 6, 4, 0.49)
  const actor = await point(page, 6, 5, 0.42)

  await showFire(page, 'empty')
  const emptyCanvas = await canvasScreenshot(page, testInfo, 'fire-light-empty-arena')
  const emptyFloor = await canvasRegionScreenshot(page, testInfo, 'fire-light-empty-floor', floor, 9)
  const emptyWall = await canvasRegionScreenshot(page, testInfo, 'fire-light-empty-wall', wall, 9)
  await showFire(page, 'single')
  const litFloor = await canvasRegionScreenshot(page, testInfo, 'fire-light-warm-floor', floor, 9)
  const litWall = await canvasRegionScreenshot(page, testInfo, 'fire-light-warm-wall', wall, 9)
  await canvasRegionScreenshot(page, testInfo, 'fire-light-closeup', floor, 115)

  const floorDelta = await pixelDifference(page, emptyFloor, litFloor)
  const wallDelta = await pixelDifference(page, emptyWall, litWall)
  expect(floorDelta.changed, JSON.stringify(floorDelta)).toBeGreaterThan(2)
  expect(wallDelta.red - wallDelta.blue, JSON.stringify(wallDelta)).toBeGreaterThan(100)
  expect(floorDelta.red - floorDelta.blue, JSON.stringify(floorDelta)).toBeGreaterThan(0)

  await showFire(page, 'bombOnly')
  const bombFloor = await canvasRegionScreenshot(page, testInfo, 'fire-light-bomb-only-floor', floor, 9)
  const bombWall = await canvasRegionScreenshot(page, testInfo, 'fire-light-bomb-only-wall', wall, 9)
  expect(bombFloor.equals(emptyFloor), 'bomb facts must not illuminate nearby floor').toBe(true)
  expect(bombWall.equals(emptyWall), 'bomb facts must not illuminate nearby wall').toBe(true)
  await showFire(page, 'concealed')
  const concealed = await canvasScreenshot(page, testInfo, 'fire-light-concealed-public-view')
  expect(concealed.equals(emptyCanvas), 'concealed facts have no public lighting row to infer from').toBe(true)

  await showFire(page, 'actorOnly')
  const plainActor = await canvasRegionScreenshot(page, testInfo, 'fire-tint-actor-unlit', actor, 18)
  await showFire(page, 'actorNear')
  const warmActor = await canvasRegionScreenshot(page, testInfo, 'fire-tint-actor-warm', actor, 18)
  const actorDelta = await pixelDifference(page, plainActor, warmActor)
  expect(actorDelta.changed, JSON.stringify(actorDelta)).toBeGreaterThan(5)
  expect(actorDelta.red - actorDelta.blue, JSON.stringify(actorDelta)).toBeGreaterThan(0)
})

test('native fire advances six poses with bounded ignition smoke and embers', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name.includes('reduced'), 'normal-motion cosmetic fire proof')
  await openFixture(page)
  const center = await point(page, 5, 5, 0.5)
  const bornAt = await beginFireTransition(page)
  const frames: Buffer[] = []
  for (const [index, age] of [0, 80, 160, 240, 320, 400].entries()) {
    await settleFireAge(page, bornAt, age)
    frames.push(await canvasRegionScreenshot(
      page, testInfo, `fire-pose-${index}-age-${age}`, center, 105,
    ))
  }
  expect(new Set(frames.map((frame) => frame.toString('base64'))).size,
    'all six production flame poses must alter native pixels').toBe(6)

  await settleFireAge(page, bornAt, 0)
  const ignition = await canvasRegionScreenshot(page, testInfo, 'fire-ignition-start', center, 65)
  await settleFireAge(page, bornAt, 480)
  const settled = await canvasRegionScreenshot(page, testInfo, 'fire-ignition-expired', center, 65)
  expect(ignition.equals(settled), 'short ignition and drifting particles must change the closeup').toBe(false)
})

test('reduced motion keeps fire and illumination static without ignition smoke or embers', async ({ page }, testInfo) => {
  test.skip(!testInfo.project.name.includes('reduced'), 'reduced-motion native pixel proof')
  await openFixture(page)
  const center = await point(page, 5, 5, 0.5)
  const bornAt = await beginFireTransition(page)
  const frames: Buffer[] = []
  for (const age of [0, 160, 480]) {
    await settleFireAge(page, bornAt, age)
    frames.push(await canvasRegionScreenshot(page, testInfo, `reduced-fire-static-${age}`, center, 90))
  }
  expect(frames[1]!.equals(frames[0]!), 'reduced fire must not advance or spawn particles').toBe(true)
  expect(frames[2]!.equals(frames[0]!), 'reduced fire lighting must not flicker').toBe(true)
})

test('fire state resets and context restoration leave bounded native resources without ghosts', async ({ page }, testInfo) => {
  await openFixture(page)
  await showFire(page, 'empty')
  const baseline = await canvasScreenshot(page, testInfo, 'fire-reset-empty-baseline')
  for (const reason of ['clear', 'round', 'admission', 'backward', 'preferences'] satisfies FireReset[]) {
    await beginFireTransition(page)
    await resetFire(page, reason)
    const reset = await canvasScreenshot(page, testInfo, `fire-reset-${reason}`)
    expect(reset.equals(baseline), `${reason} reset must leave no fire pixels or light`).toBe(true)
  }

  await showFire(page, 'single')
  const before = await canvasScreenshot(page, testInfo, 'fire-context-before')
  const context = await page.evaluate(() => window.grenadiersThreeArena.loseAndRestoreContext())
  if (context.supported) {
    expect(context.restored).toMatchObject({ canvasCount: 1, canvasVisible: true, destroyed: false })
    const restored = await canvasScreenshot(page, testInfo, 'fire-context-restored')
    expect(restored.equals(before), 'restored context must reproduce held terrain, fire, and lighting').toBe(true)
    await resetFire(page, 'clear')
    const restoredEmpty = await canvasScreenshot(page, testInfo, 'fire-context-restored-empty')
    expect(restored.equals(restoredEmpty), 'restored context must retain public fire pixels').toBe(false)
    expect(before.equals(baseline), 'pre-loss public fire control must differ from empty').toBe(false)
  }
  const destroyed = await page.evaluate(() => window.grenadiersThreeArena.destroy())
  expect(destroyed).toMatchObject({ canvasCount: 0, pendingAnimationFrames: 0, destroyed: true })
})

test('worst-case public fire keeps warmed native frame work bounded after warmup', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name.includes('reduced'), 'single isolated desktop frame benchmark')
  await openFixture(page)
  const empty = await measureFireCost(page, 'empty')
  const fire = await measureFireCost(page, 'worstCase')
  await testInfo.attach('fire-frame-cost', {
    body: JSON.stringify({ empty, fire }), contentType: 'application/json',
  })
  console.log(`FIRE_FRAME_COST ${JSON.stringify({ empty, fire })}`)
  expect(empty.samples).toBeGreaterThanOrEqual(40)
  expect(fire.samples).toBeGreaterThanOrEqual(40)
  expect(fire.p95Ms, JSON.stringify({ empty, fire })).toBeLessThan(50)
  expect(fire.medianMs, JSON.stringify({ empty, fire }))
    .toBeLessThanOrEqual(empty.medianMs * 3 + 4)
})
