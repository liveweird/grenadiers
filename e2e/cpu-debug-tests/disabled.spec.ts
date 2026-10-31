import { expect, test } from '@playwright/test'
import { latestSnapshot, observeGame } from '../tests/support/duel'
import { createCpuMatch, debugPath, englishTitle, observeDebug, polishTitle } from './support'

test.beforeAll(() => expect(process.env.E2E_CPU_DEBUG_ENABLED).toBe('false'))

test('operator-disabled CPU diagnostics return no intent and stay disabled across game start', async ({ browser }) => {
  const context = await browser.newContext()
  try {
    const page = await context.newPage()
    const game = observeGame(page)
    const debug = observeDebug(page)
    await page.goto('/?cpuDebug=1')
    await createCpuMatch(page, game, false)
    await page.getByRole('button', { name: englishTitle, exact: true }).click()
    const panel = page.getByRole('region', { name: englishTitle, exact: true })
    await expect(panel.getByText('CPU diagnostics are disabled on this server.', { exact: true })).toBeVisible()
    expect(debug.statuses).toContain(404)
    expect(debug.responses).toEqual([])
    const response = await page.request.post(debugPath, { data: {} })
    expect(response.status()).toBe(404)
    const requests = debug.requestCount()
    await page.getByRole('button', { name: 'Ready', exact: true }).click()
    await expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')
    const tick = latestSnapshot(game)!.room.tick
    await expect.poll(() => latestSnapshot(game)?.room.tick).toBeGreaterThan(tick + 30)
    expect(debug.requestCount()).toBe(requests)
    await expect(panel.getByText('CPU diagnostics are disabled on this server.', { exact: true })).toBeVisible()
    await page.getByText('Polski', { exact: true }).click()
    await expect(page.getByRole('region', { name: polishTitle, exact: true })
      .getByText('Diagnostyka komputera jest wyłączona na tym serwerze.', { exact: true })).toBeVisible()
  } finally { await context.close() }
})
