import { expect, test, type Locator, type Page } from '@playwright/test'
import { expectPaintedCanvas, latestSnapshot, observeGame } from './support/duel'
import { openSettings } from './support/settings'

const SIZES = [
  { width: 1440, height: 900 }, { width: 1024, height: 768 },
  { width: 390, height: 844 }, { width: 320, height: 568 },
] as const

async function expectReachable(locator: Locator, page: Page): Promise<void> {
  await locator.scrollIntoViewIfNeeded()
  await expect(locator).toBeVisible()
  const box = await locator.boundingBox()
  const viewport = page.viewportSize()
  expect(box).not.toBeNull()
  expect(viewport).not.toBeNull()
  expect(box!.x).toBeGreaterThanOrEqual(0)
  expect(box!.y).toBeGreaterThanOrEqual(0)
  expect(box!.x + box!.width).toBeLessThanOrEqual(viewport!.width)
  expect(box!.y + box!.height).toBeLessThanOrEqual(viewport!.height)
}

async function expectInitiallyVisible(locator: Locator, page: Page): Promise<void> {
  await expect(locator).toBeVisible()
  expect(await locator.evaluate((element) => {
    const offsets: number[][] = []
    for (let parent = element.parentElement; parent; parent = parent.parentElement) {
      offsets.push([parent.scrollLeft, parent.scrollTop])
    }
    return offsets.every(([left, top]) => left === 0 && top === 0)
  })).toBe(true)
  const box = await locator.boundingBox()
  const viewport = page.viewportSize()!
  expect(box).not.toBeNull()
  expect(box!.x).toBeGreaterThanOrEqual(0)
  expect(box!.y).toBeGreaterThanOrEqual(0)
  expect(box!.x + box!.width).toBeLessThanOrEqual(viewport.width)
  expect(box!.y + box!.height).toBeLessThanOrEqual(viewport.height)
}

test('the game shell contains play and settings at every supported viewport', async ({ page }, testInfo) => {
  for (const size of SIZES) {
    await page.setViewportSize(size)
    await page.goto('/')
    await expect(page.locator('.game-shell')).toBeVisible()
    const geometry = await page.evaluate(() => {
      const arena = document.querySelector('.game-arena')!.getBoundingClientRect()
      return {
        bodyX: document.body.scrollWidth - document.body.clientWidth,
        bodyY: document.body.scrollHeight - document.body.clientHeight,
        rootX: document.documentElement.scrollWidth - document.documentElement.clientWidth,
        rootY: document.documentElement.scrollHeight - document.documentElement.clientHeight,
        arena: { left: arena.left, top: arena.top, right: arena.right, bottom: arena.bottom },
      }
    })
    expect(geometry).toMatchObject({ bodyX: 0, bodyY: 0, rootX: 0, rootY: 0 })
    expect(geometry.arena.left).toBeGreaterThanOrEqual(0)
    expect(geometry.arena.top).toBeGreaterThanOrEqual(0)
    expect(geometry.arena.right).toBeLessThanOrEqual(size.width)
    expect(geometry.arena.bottom).toBeLessThanOrEqual(size.height)

    for (const name of ['Create room', 'Play solo against CPU', 'Join room']) {
      await expectInitiallyVisible(page.getByRole('button', { name, exact: true }), page)
    }
    const toggle = page.getByRole('button', { name: 'Open settings', exact: true })
    await expectInitiallyVisible(toggle, page)
    await toggle.click()
    const pane = page.locator('#game-settings-pane')
    await expect(pane).toBeVisible()
    await expectReachable(page.getByRole('combobox', { name: 'Board size', exact: true }), page)
    await page.getByRole('button', { name: 'Close settings', exact: true }).click()
    await expect(toggle).toBeFocused()
    await expect(pane).toBeHidden()
    await page.evaluate(() => {
      window.scrollTo(0, 0)
      document.querySelector('.control-column')!.scrollTop = 0
    })
    expect(await page.evaluate(() => window.scrollY)).toBe(0)
    await page.screenshot({ path: testInfo.outputPath(`viewport-${size.width}x${size.height}.png`) })
  }
})

test('a running four-player HUD remains contained at every supported viewport', async ({ browser }, testInfo) => {
  test.setTimeout(60_000)
  const contexts = await Promise.all(SIZES.map((size) => browser.newContext({ viewport: size })))
  try {
    const pages = await Promise.all(contexts.map((context) => context.newPage()))
    const game = observeGame(pages[0])
    await Promise.all(pages.map((page) => page.goto('/')))
    await openSettings(pages[0])
    await pages[0].getByRole('combobox', { name: 'Players', exact: true }).selectOption('4')
    await pages[0].getByRole('button', { name: 'Create custom match', exact: true }).click()
    const invitation = pages[0].getByLabel('Room invitation code', { exact: true })
    await expect(invitation).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    const code = await invitation.innerText()
    for (const page of pages.slice(1)) {
      await page.getByRole('textbox', { name: 'Room code', exact: true }).fill(code)
      await page.getByRole('button', { name: 'Join room', exact: true }).click()
    }
    await Promise.all(pages.map((page) => page.getByRole('button', { name: 'Ready', exact: true }).click()))
    await expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')
    await Promise.all(pages.map(expectPaintedCanvas))

    for (const [index, page] of pages.entries()) {
      const column = (await page.locator('.arena-column').boundingBox())!
      for (const selector of ['.game-arena', '.own-status-bar', '.local-controls']) {
        await expectInitiallyVisible(page.locator(selector), page)
        const box = (await page.locator(selector).boundingBox())!
        expect(box.y).toBeGreaterThanOrEqual(column.y)
        expect(box.y + box.height).toBeLessThanOrEqual(column.y + column.height)
      }
      const arena = (await page.locator('.game-arena').boundingBox())!
      if (SIZES[index].width === 320) expect(arena.width).toBeGreaterThanOrEqual(240)
      expect(await page.evaluate(() => ({
        bodyX: document.body.scrollWidth - document.body.clientWidth,
        bodyY: document.body.scrollHeight - document.body.clientHeight,
      }))).toEqual({ bodyX: 0, bodyY: 0 })
      await page.screenshot({ path: testInfo.outputPath(`running-${SIZES[index].width}x${SIZES[index].height}.png`) })
    }
    const narrow = pages.at(-1)!
    const statusList = narrow.locator('.own-status-list')
    await statusList.evaluate((list) => {
      const items = [...list.children]
      for (let index = 0; index < 12; index++) list.append(items[index % items.length].cloneNode(true))
    })
    await narrow.locator('.own-pickup-notice').evaluate((notice) => {
      notice.innerHTML = '<span>Golden flame collected · 0:10</span>'
    })
    expect(await statusList.evaluate((list) => list.scrollWidth > list.clientWidth)).toBe(true)
    const lastStatus = statusList.locator('.own-status-item').last()
    await lastStatus.scrollIntoViewIfNeeded()
    await expect(lastStatus).toBeVisible()
    for (const selector of ['.game-arena', '.own-status-bar', '.local-controls']) {
      const box = await narrow.locator(selector).boundingBox()
      const column = (await narrow.locator('.arena-column').boundingBox())!
      expect(box).not.toBeNull()
      expect(box!.y).toBeGreaterThanOrEqual(column.y)
      expect(box!.y + box!.height).toBeLessThanOrEqual(column.y + column.height)
    }
    await narrow.screenshot({ path: testInfo.outputPath('running-320x568-dense-hud.png') })
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
