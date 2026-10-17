import AxeBuilder from '@axe-core/playwright'
import { expect, test } from '@playwright/test'
import { expectPaintedCanvas, latestSnapshot, observeGame } from './support/duel'

test('two players complete a rendered keyboard duel with shared authoritative results', async ({ browser }, testInfo) => {
  test.setTimeout(60_000)
  const contexts = await Promise.all([
    browser.newContext({ viewport: { width: 1440, height: 1050 } }),
    browser.newContext({ viewport: { width: 390, height: 844 }, reducedMotion: 'reduce' }),
  ])
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    const games = [observeGame(one), observeGame(two)]
    const pageErrors: string[] = []
    for (const page of [one, two]) page.on('pageerror', (error) => pageErrors.push(error.message))
    await Promise.all([one, two].map((page) => page.goto('/')))
    expect((await new AxeBuilder({ page: one }).analyze()).violations).toEqual([])
    await one.screenshot({ path: testInfo.outputPath('lobby-desktop.png'), fullPage: true })

    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const invitation = one.getByLabel('Room invitation code', { exact: true })
    await expect(invitation).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    const code = await invitation.innerText()
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(code)
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await expect(two.getByLabel('Room invitation code', { exact: true })).toHaveText(code)
    await expect(one.getByRole('list', { name: 'Players' })).toContainText('Player two')
    await two.getByText('Polski', { exact: true }).click()
    await expect(two.getByRole('button', { name: 'Opuść pokój', exact: true })).toBeVisible()
    await two.getByText('English', { exact: true }).click()
    await expect(two.getByRole('radio', { name: 'English', exact: true })).toBeChecked()
    await expect(two.getByLabel('Room invitation code', { exact: true })).toHaveText(code)
    expect(games.map((game) => game.sockets)).toEqual([1, 1])
    for (const [index, page] of [one, two].entries()) {
      expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([])
      await page.screenshot({ path: testInfo.outputPath(`waiting-${index}.png`), fullPage: true })
      expect(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
      if (index === 1) {
        const arenaBounds = await page.locator('.game-arena').boundingBox()
        const controlBounds = await page.locator('.control-column').boundingBox()
        expect(arenaBounds!.width).toBeGreaterThan(340)
        expect(controlBounds!.y).toBeGreaterThan(arenaBounds!.y + arenaBounds!.height)
      }
    }

    await one.getByRole('button', { name: 'Ready', exact: true }).click()
    await two.getByRole('button', { name: 'Ready', exact: true }).click()
    await expect.poll(() => latestSnapshot(games[0])?.room.phase.type).toBe('running')
    await expect.poll(() => latestSnapshot(games[1])?.room.phase.type).toBe('running')
    await Promise.all([one, two].map(expectPaintedCanvas))
    for (const page of [one, two]) {
      for (const player of ['one', 'two']) {
        await expect(page.getByLabel(`Player ${player} upgrades`).locator('dd')).toHaveText(['1', '1', '2', '1'])
      }
    }
    for (const page of [one, two]) {
      expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([])
    }
    const arena = one.getByRole('region', { name: 'Grenadiers courtyard' })
    const actor = () => latestSnapshot(games[0])?.state.actors.find((candidate) => candidate.seat === 'one')
    await arena.click()
    await one.keyboard.down('ArrowRight')
    await expect.poll(() => actor()?.position.x, { intervals: [20] }).toBeGreaterThan(1900)
    await one.getByRole('button', { name: 'Leave room', exact: true }).focus()
    await one.keyboard.up('ArrowRight')
    const beforeBlurRevision = latestSnapshot(games[0])!.room.revision
    await expect.poll(() => latestSnapshot(games[0])!.room.revision).toBeGreaterThan(beforeBlurRevision + 2)
    const stopped = actor()!.position.x
    await one.keyboard.down('ArrowRight')
    const stoppedRevision = latestSnapshot(games[0])!.room.revision
    await expect.poll(() => latestSnapshot(games[0])!.room.revision).toBeGreaterThan(stoppedRevision + 3)
    await one.keyboard.up('ArrowRight')
    expect(actor()!.position.x).toBe(stopped)

    await arena.click()
    await one.keyboard.down('ArrowRight')
    await expect.poll(() => actor()?.position.x, { intervals: [20] }).toBeGreaterThan(3450)
    await one.keyboard.up('ArrowRight')
    expect(latestSnapshot(games[0])!.state.arena.blocks).toContainEqual({ column: 5, row: 1 })
    await one.keyboard.down('Space')
    await one.keyboard.down('Space')
    await expect.poll(() => games[0].commands.filter((command) => command.type === 'placeBomb')).toEqual([{ type: 'placeBomb', actionId: 1 }])
    await expect.poll(() => latestSnapshot(games[0])?.state.bombs.length).toBe(1)
    for (const page of [one, two]) {
      await expect(page.getByLabel('Player one upgrades').locator('dd')).toHaveText(['0', '1', '2', '1'])
    }
    await one.screenshot({ path: testInfo.outputPath('running-bomb.png'), fullPage: true })
    await expect.poll(() => latestSnapshot(games[0])?.room.phase.type).toBe('result')
    await one.keyboard.up('Space')
    await expect.poll(() => latestSnapshot(games[1])?.room.phase.type).toBe('result')
    for (const [index, page] of [one, two].entries()) {
      const result = latestSnapshot(games[index])!
      expect(result.room.phase).toMatchObject({ type: 'result', result: { type: 'winner', seat: 'two' } })
      expect(result.state.arena.blocks).not.toContainEqual({ column: 5, row: 1 })
      expect(result.state.actors.find((candidate) => candidate.seat === 'one')?.alive).toBe(false)
      const seconds = Math.ceil(result.state.remainingTicks / 60)
      await expect(page.getByLabel('Round time remaining')).toHaveText(`${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`)
      await expect(page.getByRole('status')).toContainText('Player two wins the round.')
      await expect(page.getByLabel('Player one upgrades').locator('dd')).toHaveText(['1', '1', '2', '1'])
      await expect(page.locator('.seat-one small')).toHaveText('Eliminated')
      await expect(page.locator('.seat-two small')).toHaveText('Alive')
      expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([])
      await page.screenshot({ path: testInfo.outputPath(`result-${index}.png`), fullPage: true })
    }
    await two.setViewportSize({ width: 320, height: 844 })
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    for (const stats of await two.locator('.actor-stats').all()) {
      const bounds = await stats.boundingBox()
      expect(bounds!.x).toBeGreaterThanOrEqual(0)
      expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(320)
    }
    await two.screenshot({ path: testInfo.outputPath('result-320.png'), fullPage: true })
    const firstSnapshots = games[0].frames.filter((frame) => frame.type === 'snapshot')
    const secondSnapshots = games[1].frames.filter((frame) => frame.type === 'snapshot')
    const shared = firstSnapshots.filter((frame) => secondSnapshots.some((other) => other.room.revision === frame.room.revision))
    expect(shared.length).toBeGreaterThan(20)
    for (const frame of shared) {
      expect(secondSnapshots.find((other) => other.room.revision === frame.room.revision)).toEqual(frame)
    }
    expect(games.map((game) => game.sockets)).toEqual([1, 1])
    expect(games[0].commands.filter((command) => command.type === 'placeBomb')).toEqual([{ type: 'placeBomb', actionId: 1 }])
    expect(pageErrors).toEqual([])
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
