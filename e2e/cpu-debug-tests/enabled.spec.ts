import { expect, test } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { latestSnapshot, observeGame, visibleActor } from '../tests/support/duel'
import { createCpuMatch, englishTitle, observeDebug, polishTitle } from './support'

test.beforeAll(() => expect(process.env.E2E_CPU_DEBUG_ENABLED).toBe('true'))

test('normal gameplay makes no CPU diagnostic requests without URL opt-in', async ({ browser }) => {
  const context = await browser.newContext()
  try {
    const page = await context.newPage()
    const game = observeGame(page)
    const debug = observeDebug(page)
    await page.goto('/')
    await createCpuMatch(page, game, true)
    const tick = latestSnapshot(game)!.room.tick
    await expect.poll(() => latestSnapshot(game)?.room.tick).toBeGreaterThan(tick + 60)
    await expect(page.getByRole('region', { name: englishTitle, exact: true })).toHaveCount(0)
    expect(debug.requestCount()).toBe(0)
  } finally { await context.close() }
})

test('opted-in CPU intents transition from lobby to play and render in English and Polish without blocking controls', async ({ browser }, testInfo) => {
  const context = await browser.newContext()
  const debugHistory: { statuses: number[]; responses: unknown[] } = { statuses: [], responses: [] }
  try {
    const page = await context.newPage()
    const game = observeGame(page)
    const debug = observeDebug(page)
    debugHistory.statuses = debug.statuses
    debugHistory.responses = debug.responses
    await page.goto('/?cpuDebug=1')
    await createCpuMatch(page, game, false)
    await page.getByRole('button', { name: englishTitle, exact: true }).click()
    const panel = page.getByRole('region', { name: englishTitle, exact: true })
    await expect(panel).toBeVisible()
    await expect.poll(() => debug.responses.some((response) => response.cpus.length === 3
      && response.cpus.every((row) => row.status === 'unavailable'))).toBe(true)
    expect(debug.statuses).not.toContain(404)
    await page.getByRole('button', { name: 'Ready', exact: true }).click()
    await expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')
    await expect.poll(() => debug.responses.some((response) => response.cpus.some(
      (row) => row.status === 'available'))).toBe(true)
    await expect(panel.getByText('Objective', { exact: true }).first()).toBeVisible()
    await expect(panel.getByText('Latest switch reason', { exact: true }).first()).toBeVisible()
    const result = debug.responses.at(-1)!
    expect(result.roomId).toBe(latestSnapshot(game)!.room.roomId)
    expect(result.roundId).toBe(latestSnapshot(game)!.room.roundId)
    expect(result.youSeat).toBe('one')
    expect(JSON.stringify(result)).not.toMatch(/[0-9a-f]{64}/)
    const arena = page.getByRole('region', { name: 'Grenadiers courtyard' })
    await arena.click()
    const before = visibleActor(latestSnapshot(game), 'one')!.position.y
    await page.keyboard.down('ArrowDown')
    try {
      await expect.poll(() => visibleActor(latestSnapshot(game), 'one')?.position.y).toBeGreaterThan(before)
    } finally { await page.keyboard.up('ArrowDown') }
    await expect(panel.getByText('Objective', { exact: true }).first()).toBeVisible()
    await testInfo.attach('cpu-intent-en', { body: await panel.screenshot(), contentType: 'image/png' })
    expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([])
    await page.getByText('Polski', { exact: true }).click()
    const polish = page.getByRole('region', { name: polishTitle, exact: true })
    await expect(polish.getByText('Ostatni powód zmiany', { exact: true }).first()).toBeVisible()
    await testInfo.attach('cpu-intent-pl', { body: await polish.screenshot(), contentType: 'image/png' })
    await page.getByRole('button', { name: polishTitle, exact: true }).click()
    await expect(page.getByRole('button', { name: polishTitle, exact: true })).toHaveAttribute('aria-expanded', 'false')
  } finally {
    await testInfo.attach('cpu-intent-public-responses', {
      body: JSON.stringify({ statuses: debugHistory.statuses, responses: debugHistory.responses.slice(-4) }),
      contentType: 'application/json',
    })
    await context.close()
  }
})
