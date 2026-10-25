import { createResponsiveBrowserContexts } from '../tests/support/browser-contexts'
import { selectClassicSettings } from '../tests/support/settings'
import AxeBuilder from '@axe-core/playwright'
import { expect, test, type Page } from '@playwright/test'
import { observeBrowserPolicy } from '../tests/support/browser-policy'
import { visibleActor, latestSnapshot, observeGame, type ObservedGame } from '../tests/support/duel'

async function changeKey(page: Page, action: string, key: string): Promise<void> {
  await page.getByRole('button', { name: `Change ${action}`, exact: true }).click()
  await page.keyboard.press(key)
}

function firstActor(game: ObservedGame) {
  const actor = visibleActor(latestSnapshot(game), 'one')
  if (!actor) throw new Error('First actor is unavailable')
  return actor
}

test('custom controls persist and drive only their assigned native keys', async ({ browser }, testInfo) => {
  const contexts = await createResponsiveBrowserContexts(browser)
  const games: ObservedGame[] = []
  try {
    const policies = await Promise.all(contexts.map(observeBrowserPolicy))
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    games.push(observeGame(one), observeGame(two))
    await Promise.all([one, two].map((page) => page.goto('/')))

    await one.getByRole('button', { name: 'Keyboard controls', exact: true }).click()
    await changeKey(one, 'East', 'KeyL')
    await changeKey(one, 'Bomb', 'Enter')
    await expect(one.getByRole('dialog', { name: 'Keyboard controls' })).toContainText('L')
    expect((await new AxeBuilder({ page: one }).analyze()).violations).toEqual([])
    await one.getByRole('button', { name: 'Done', exact: true }).click()
    await expect(one.getByText(/East: L/)).toBeVisible()
    await one.reload()
    await expect(one.getByText(/East: L/)).toBeVisible()
    await expect(one.getByText(/bomb with Enter/)).toBeVisible()

    await two.getByText('Polski', { exact: true }).click()
    await two.getByRole('button', { name: 'Sterowanie klawiaturą', exact: true }).click()
    expect((await new AxeBuilder({ page: two }).analyze()).violations).toEqual([])
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    const dialogBounds = await two.getByRole('dialog', { name: 'Sterowanie klawiaturą' }).boundingBox()
    expect(dialogBounds!.x).toBeGreaterThanOrEqual(0)
    expect(dialogBounds!.x + dialogBounds!.width).toBeLessThanOrEqual(320)
    await two.screenshot({ path: testInfo.outputPath('controls-polish-320.png'), fullPage: true })
    await two.getByRole('button', { name: 'Gotowe', exact: true }).click()

    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    await selectClassicSettings(one, games[0])
    const invitation = one.getByLabel('Room invitation code', { exact: true })
    await expect(invitation).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await two.getByRole('textbox', { name: 'Kod pokoju', exact: true }).fill(await invitation.innerText())
    await two.getByRole('button', { name: 'Dołącz do pokoju', exact: true }).click()
    await one.getByRole('button', { name: 'Ready', exact: true }).click()
    await two.getByRole('button', { name: 'Gotowy', exact: true }).click()
    await expect.poll(() => latestSnapshot(games[0])?.room.phase.type).toBe('running')
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()

    const beforeOldKey = firstActor(games[0]).position.x
    const beforeRevision = latestSnapshot(games[0])!.room.revision
    await one.keyboard.press('ArrowRight', { delay: 120 })
    await expect.poll(() => latestSnapshot(games[0])!.room.revision).toBeGreaterThan(beforeRevision + 2)
    expect(firstActor(games[0]).position.x).toBe(beforeOldKey)

    await one.keyboard.down('KeyL')
    await expect.poll(() => firstActor(games[0]).position.x, { intervals: [20] }).toBeGreaterThan(beforeOldKey)
    await one.keyboard.up('KeyL')
    await one.keyboard.down('Enter')
    await one.keyboard.down('Enter')
    await expect.poll(() => games[0].commands.filter((command) => command.type === 'manualAction'))
      .toEqual([{ type: 'manualAction', action: 'bomb', actionId: 1 }])
    await one.keyboard.up('Enter')

    await one.getByRole('button', { name: 'Keyboard controls', exact: true }).click()
    await one.getByRole('button', { name: 'Restore defaults', exact: true }).click()
    await one.getByRole('button', { name: 'Done', exact: true }).click()
    await expect(one.getByText(/East: Right arrow \/ D/)).toBeVisible()
    await expect(one.getByText(/bomb with Space/)).toBeVisible()
    await one.reload()
    await expect(one.getByText(/East: Right arrow \/ D/)).toBeVisible()
    expect(await one.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await one.screenshot({ path: testInfo.outputPath('controls-reset-desktop.png'), fullPage: true })
    expect(policies.flatMap((policy) => policy.violations)).toEqual([])
  } catch (error) {
    await testInfo.attach('controls-public-summary', {
      contentType: 'application/json',
      body: JSON.stringify(games.map((game) => ({
        sockets: game.sockets,
        commands: game.commands.slice(-64),
        snapshots: game.frames.filter((frame) => frame.type === 'snapshot').slice(-20),
      }))),
    })
    throw error
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
