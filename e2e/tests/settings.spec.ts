import { openSettings, selectClassicSettings } from './support/settings'
import { expect, test, type Page } from '@playwright/test'
import { expectPaintedCanvas, latestSnapshot, observeGame, type ObservedGame } from './support/duel'
import { admissionBaseline, disconnectTransport, installTransportProbe } from './support/reconnect'

function latestRoom(game: ObservedGame) {
  return game.frames.filter((frame) =>
    frame.type === 'roomState' || frame.type === 'snapshot' || frame.type === 'admitted').at(-1)?.room
}

async function expectPolishDropWeight(page: Page) {
  await page.getByText('Polski', { exact: true }).click()
  await openSettings(page)
  await expect(page.getByRole('button', { name: 'Włącz wszystkie', exact: true })).toBeVisible()
  await expect(page.getByRole('button', { name: 'Wyłącz wszystkie', exact: true })).toBeVisible()
  await expect(page.getByRole('checkbox', { name: 'Chmura burzowa', exact: true })).toBeChecked()
  await expect(page.getByRole('checkbox', { name: 'Legowiska potworów', exact: true })).toBeChecked()
  await expect(page.getByRole('combobox', { name: 'Waga losowania dodatkowej bomby', exact: true }))
    .toHaveValue('5')
  await page.getByRole('button', { name: 'Zamknij ustawienia', exact: true }).click()
  await expect(page.locator('#game-settings-pane')).toBeHidden()
  await page.getByText('English', { exact: true }).click()
}

test('two browsers share configured rules that drive speed, fuse and match target', async ({ browser }) => {
  const contexts = await Promise.all([browser.newContext(), browser.newContext()])
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    const games = [observeGame(one), observeGame(two)]
    await installTransportProbe(two)
    await Promise.all([one, two].map((page) => page.goto('/')))
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    await expect.poll(() => latestRoom(games[0])).toBeTruthy()
    await openSettings(one)
    expect(latestRoom(games[0])?.settings.enabledItems).toEqual(latestRoom(games[0])?.availableItems)
    await expect(one.getByRole('button', { name: 'All on', exact: true })).toBeVisible()
    await expect(one.getByRole('button', { name: 'All off', exact: true })).toBeVisible()
    const stormCloud = one.getByRole('checkbox', { name: 'Storm cloud', exact: true })
    const lairs = one.getByRole('checkbox', { name: 'Monster lairs', exact: true })
    const spawnSeconds = one.getByRole('textbox', { name: 'Respawn delay (seconds)', exact: true })
    const maxLivingMobs = one.getByRole('textbox', { name: 'Living monsters per lair', exact: true })
    const dropWeight = one.getByRole('combobox', { name: 'Extra-bomb drop weight', exact: true })
    await expect(stormCloud).toBeChecked()
    await expect(lairs).toBeChecked()
    await expect(spawnSeconds).toHaveValue('10')
    await expect(maxLivingMobs).toHaveCount(0)
    await expect(dropWeight).toHaveValue('5')
    const defaultEnvironment = {
      stormCloud: true,
      lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 },
      spikePits: { enabled: true, warningTicks: 60, raisedTicks: 300 },
    }
    expect(latestRoom(games[0])?.settings.environment).toEqual(defaultEnvironment)
    expect(latestRoom(games[0])?.configuration.environment).toEqual(defaultEnvironment)
    await spawnSeconds.fill('11')
    await one.getByRole('button', { name: 'All off', exact: true }).click()
    await expect(dropWeight).toHaveValue('5')
    await expect(stormCloud).not.toBeChecked()
    await expect(lairs).not.toBeChecked()
    await expect(spawnSeconds).toHaveValue('11')
    await expect(maxLivingMobs).toHaveCount(0)
    await one.getByRole('button', { name: 'All on', exact: true }).click()
    await expect(dropWeight).toHaveValue('5')
    await expect(stormCloud).toBeChecked()
    await expect(lairs).toBeChecked()
    await expect(spawnSeconds).toHaveValue('11')
    await expect(maxLivingMobs).toHaveCount(0)
    await one.getByRole('button', { name: 'Classic settings', exact: true }).click()
    await expect(dropWeight).toHaveValue('1')
    await expect(stormCloud).not.toBeChecked()
    await expect(lairs).not.toBeChecked()
    await expect(spawnSeconds).toHaveValue('10')
    await expect(maxLivingMobs).toHaveCount(0)
    await one.getByRole('button', { name: 'Restore match defaults', exact: true }).click()
    await expect(dropWeight).toHaveValue('5')
    await expect(stormCloud).toBeChecked()
    await expect(lairs).toBeChecked()
    await expect(spawnSeconds).toHaveValue('10')
    await expect(maxLivingMobs).toHaveCount(0)
    await one.getByRole('button', { name: 'Close settings', exact: true }).click()
    await expect(one.locator('#game-settings-pane')).toBeHidden()
    await expectPolishDropWeight(one)
    await selectClassicSettings(one, games[0])
    await expect(one.locator('#game-settings-pane')).toBeHidden()
    const invitation = one.getByLabel('Room invitation code', { exact: true })
    await expect(invitation).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await invitation.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()

    await two.getByRole('button', { name: 'Ready', exact: true }).click()
    await expect.poll(() => latestRoom(games[0])?.seats[1]?.ready).toBe(true)

    await openSettings(one)
    await one.getByLabel('Starting speed').selectOption('4')
    await one.getByLabel('Turning tolerance').selectOption('64')
    await one.getByLabel('Round wins needed').selectOption('1')
    await one.getByLabel('Bomb fuse').selectOption('60')
    await one.getByLabel('Extra-bomb drop weight').selectOption('10')
    await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await expect(one.locator('#game-settings-pane')).toBeHidden()
    const expected = {
      startingSpeedLevel: 4, turnAssistUnits: 64, winsToFinish: 1, bombFuseTicks: 60,
      bombCapacityDropMultiplier: 10, enabledItems: [],
      environment: {
        stormCloud: false,
        lairs: { enabled: false, spawnSeconds: 10, maxLivingMobs: 1 },
        spikePits: { enabled: false, warningTicks: 60, raisedTicks: 300 },
      },
    }
    await Promise.all(games.map((game) => expect.poll(() => latestRoom(game)?.settings).toEqual(expected)))
    await Promise.all(games.map((game) => expect.poll(() => latestRoom(game)?.configuration.environment)
      .toEqual({
        stormCloud: false,
        lairs: { enabled: false, spawnSeconds: 10, maxLivingMobs: 1 },
        spikePits: { enabled: false, warningTicks: 60, raisedTicks: 300 },
      })))
    await Promise.all(games.map((game) => expect.poll(() => latestRoom(game)?.seats[1]?.ready).toBe(false)))
    await openSettings(two)
    await expect(two.getByLabel('Starting speed')).toHaveValue('4')
    await expect(two.getByLabel('Bomb fuse')).toHaveValue('60')
    await expect(two.getByLabel('Extra-bomb drop weight')).toHaveValue('10')
    await two.getByRole('button', { name: 'Close settings', exact: true }).click()
    await expect(two.locator('#game-settings-pane')).toBeHidden()

    await openSettings(one)
    await one.getByRole('button', { name: 'Restore match defaults', exact: true }).click()
    await one.getByLabel('Starting speed').selectOption('4')
    await one.getByLabel('Turning tolerance').selectOption('64')
    await one.getByLabel('Round wins needed').selectOption('1')
    await one.getByLabel('Bomb fuse').selectOption('60')
    await one.getByRole('button', { name: 'All off', exact: true }).click()
    await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
    const restored = { ...expected, bombCapacityDropMultiplier: 5 }
    await Promise.all(games.map((game) => expect.poll(() => latestRoom(game)?.settings).toEqual(restored)))

    await Promise.all([one, two].map((page) => page.getByRole('button', { name: 'Ready', exact: true }).click()))
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
    for (const game of games) {
      expect(latestSnapshot(game)!.room).toMatchObject({ rulesVersion: 10, settings: restored })
      expect(latestSnapshot(game)!.state.actors.map((actor) => actor.type === 'visible' ? actor.speedLevel : null))
        .toEqual([4, 4])
      expect(latestSnapshot(game)!.state.stormCloud).toBeNull()
      expect(latestSnapshot(game)!.state.lairs).toEqual([])
      expect(latestSnapshot(game)!.state.mobs).toEqual([])
    }
    await Promise.all([one, two].map(expectPaintedCanvas))
    const beforeReconnect = await admissionBaseline(two)
    await disconnectTransport(two)
    await expect.poll(async () => (await admissionBaseline(two))?.index ?? 0)
      .toBeGreaterThan(beforeReconnect?.index ?? 0)
    await expect.poll(() => latestRoom(games[1])?.settings.bombCapacityDropMultiplier).toBe(5)
    await openSettings(one)
    await expect(stormCloud).toBeDisabled()
    await one.getByRole('button', { name: 'Close settings', exact: true }).click()
    await expect(one.locator('#game-settings-pane')).toBeHidden()

    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await one.keyboard.press('Space')
    await expect.poll(() => latestSnapshot(games[0])?.state.bombs.at(0)).toBeTruthy()
    const planted = latestSnapshot(games[0])!
    const bomb = planted.state.bombs[0]
    expect(bomb.trigger).toMatchObject({ type: 'timed', paused: false })
    if (bomb.trigger.type !== 'timed') throw new Error('Configured Classic bomb must have a timed fuse')
    expect(planted.state.physicalTick - bomb.placedAt + bomb.trigger.remainingTicks).toBe(60)
    await Promise.all(games.map((game) => expect.poll(() => latestRoom(game)?.phase.type).toBe('matchResult')))
    expect(latestRoom(games[0])?.scores).toEqual([
      { seat: 'one', wins: 0 }, { seat: 'two', wins: 1 },
    ])
    expect(latestRoom(games[0])?.settings).toEqual(restored)
    await Promise.all([one, two].map((page) => expect(page.getByRole('status'))
      .toHaveText('Player two wins the match with one round win.')))

    await Promise.all([one, two].map((page) =>
      expect(page.getByRole('button', { name: 'Play again', exact: true })).toBeEnabled({ timeout: 10_000 })))
    await Promise.all([one, two].map((page) => page.getByRole('button', { name: 'Play again', exact: true }).click()))
    await Promise.all(games.map((game) => expect.poll(() => latestRoom(game)?.phase.type).toBe('waiting')))
    expect(latestRoom(games[0])?.settings).toEqual(restored)
    await openSettings(one)
    await expect(stormCloud).toBeDisabled()
    await one.getByRole('button', { name: 'Close settings', exact: true }).click()
    await expect(one.locator('#game-settings-pane')).toBeHidden()
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
