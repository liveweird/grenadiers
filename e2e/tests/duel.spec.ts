import { selectClassicSettings } from './support/settings'
import AxeBuilder from '@axe-core/playwright'
import { expect, test } from '@playwright/test'
import { EMPTY_ACTIVE_ACTOR_STATS, expectActorStats } from './support/actor-stats'
import { visibleActor, expectPaintedCanvas, latestSnapshot, observeGame } from './support/duel'

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
    await selectClassicSettings(one, games[0])
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
      for (const player of ['one', 'two'] as const) {
        await expectActorStats(page, player, {
          available: '1', capacity: '1', range: '2', speed: '1',
          ...EMPTY_ACTIVE_ACTOR_STATS,
        })
      }
    }
    for (const page of [one, two]) {
      expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([])
    }
    const arena = one.getByRole('region', { name: 'Grenadiers courtyard' })
    const actor = () => visibleActor(latestSnapshot(games[0]), 'one')
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
    await expect.poll(() => actor()?.position.x, { intervals: [20] }).toBeGreaterThan(2350)
    await one.keyboard.up('ArrowRight')
    await one.keyboard.down('Space')
    await one.keyboard.down('Space')
    await expect.poll(() => games[0].commands.filter((command) => command.type === 'manualAction')).toEqual([{ type: 'manualAction', action: 'bomb', actionId: 1 }])
    await expect.poll(() => latestSnapshot(games[0])?.state.bombs.length).toBe(1)
    for (const page of [one, two]) {
      await expectActorStats(page, 'one', {
        available: '0', capacity: '1', range: '2', speed: '1',
        ...EMPTY_ACTIVE_ACTOR_STATS,
      })
    }
    await one.screenshot({ path: testInfo.outputPath('running-bomb.png'), fullPage: true })
    await expect.poll(() => latestSnapshot(games[0])?.room.phase.type).toBe('result')
    await one.keyboard.up('Space')
    await expect.poll(() => latestSnapshot(games[1])?.room.phase.type).toBe('result')
    // Result holds last three seconds. Assert both clients before expensive
    // compositor/accessibility work; automatic progression must keep running.
    await Promise.all([one, two].map(async (page, index) => {
      const result = latestSnapshot(games[index])!
      expect(result.room.phase.type).toBe('result')
      expect(result.room.lastRound?.result).toEqual({ type: 'winner', seat: 'two' })
      expect(result.room.scores).toEqual({ one: 0, two: 1 })
      expect(result.state.actors.find((candidate) => candidate.seat === 'one')?.alive).toBe(false)
      await expect(page.getByLabel('Phase time remaining')).toHaveText(/^0:0[123]$/)
      await expect(page.getByRole('status')).toContainText('Player two wins the round.')
      await expectActorStats(page, 'one', {
        available: '1', capacity: '1', range: '2', speed: '1',
        ...EMPTY_ACTIVE_ACTOR_STATS,
      })
      await expect(page.locator('.seat-one small')).toHaveText('Eliminated')
      await expect(page.locator('.seat-two small')).toHaveText('Alive')
    }))
    await Promise.all([one, two].map((page, index) =>
      page.screenshot({ path: testInfo.outputPath(`result-${index}.png`), fullPage: true }),
    ))
    await two.setViewportSize({ width: 320, height: 844 })
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    const statBounds = await two.locator('.actor-stats').evaluateAll((elements) => elements.map((element) => {
      const { x, width, height } = element.getBoundingClientRect()
      return { x, width, height }
    }))
    expect(statBounds).toHaveLength(2)
    for (const bounds of statBounds) {
      expect(bounds.width).toBeGreaterThan(0)
      expect(bounds.height).toBeGreaterThan(0)
      expect(bounds.x).toBeGreaterThanOrEqual(0)
      expect(bounds.x + bounds.width).toBeLessThanOrEqual(320)
    }
    await two.screenshot({ path: testInfo.outputPath('result-320.png'), fullPage: true })
    await two.setViewportSize({ width: 390, height: 844 })
    for (const page of [one, two]) {
      expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([])
    }
    const firstSnapshots = games[0].frames.filter((frame) => frame.type === 'snapshot')
    const secondSnapshots = games[1].frames.filter((frame) => frame.type === 'snapshot')
    const shared = firstSnapshots.filter((frame) => secondSnapshots.some((other) => other.room.revision === frame.room.revision))
    expect(shared.length).toBeGreaterThan(20)
    for (const frame of shared) {
      expect(secondSnapshots.find((other) => other.room.revision === frame.room.revision)).toEqual(frame)
    }
    expect(games.map((game) => game.sockets)).toEqual([1, 1])
    expect(games[0].commands.filter((command) => command.type === 'manualAction')).toEqual([{ type: 'manualAction', action: 'bomb', actionId: 1 }])
    expect(pageErrors).toEqual([])
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
