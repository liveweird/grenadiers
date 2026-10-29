import { createResponsiveBrowserContexts } from '../tests/support/browser-contexts'
import { expect, test, type Page } from '@playwright/test'
import {
  latestSnapshot, observeGame, visibleActor, type ObservedGame,
} from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'
import { applyItemSettings } from '../tests/support/settings'

test.beforeAll(() => {
  expect(process.env.E2E_HEART_FIXTURE, 'Configure the documented production Heart seed')
    .toBe('arena-v3-heart')
})

async function plant(page: Page, game: ObservedGame): Promise<void> {
  const previous = latestSnapshot(game)!.state.bombs.map((bomb) => bomb.id)
  await page.keyboard.press('Space')
  await expect.poll(() => latestSnapshot(game)?.state.bombs
    .some((bomb) => !previous.includes(bomb.id)), { intervals: [15] }).toBe(true)
}

function protectionFrame(game: ObservedGame) {
  return game.frames.find((frame) => frame.type === 'snapshot' && frame.events.some((event) =>
    event.kind.type === 'damagePrevented' && event.kind.seat === 'one' && event.kind.source === 'heart'))
}

test('two browsers reveal a generated Heart and observe one charge protect its collector', async ({ browser }, testInfo) => {
  const contexts = await createResponsiveBrowserContexts(browser)
  const games: ObservedGame[] = []
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    games.push(observeGame(one), observeGame(two))
    await Promise.all([one, two].map((page) => page.goto('/')))
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await applyItemSettings(one, games[0], [{ id: 'heart', label: 'Heart' }])
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await Promise.all([one, two].map((page) =>
      page.getByRole('button', { name: 'Ready', exact: true }).click()))
    await Promise.all(games.map((game) =>
      expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
    const initial = latestSnapshot(games[0])!
    expect(initial.room).toMatchObject({
      rulesVersion: 7, mapId: 'generated_v3', availableItems: ['kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'oilBarrel', 'teleporter'],
      settings: { enabledItems: ['heart'], environment: { stormCloud: false } },
    })
    expect(initial.state.arena.blocks).toContainEqual({ column: 3, row: 1 })
    expect(visibleActor(initial, 'one')?.inventory.equipment).toBeNull()
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()

    await moveTo(one, games[0], { column: 2, row: 1 })
    await plant(one, games[0])
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => {
      const state = latestSnapshot(games[0])!.state
      return state.bombs.length + state.flames.length
    }, { intervals: [30] }).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])?.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'heart',
    })
    await one.screenshot({ path: testInfo.outputPath('heart-exposed-desktop.png'), fullPage: true })
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 3, row: 1 })
    await Promise.all(games.map((game) => expect.poll(() =>
      visibleActor(latestSnapshot(game), 'one')?.inventory.equipment).toBe('heart')))
    await expect(one.getByRole('region', { name: 'Your items' })).toContainText('GearHeart')
    await expect(one.getByText('Picked up Heart.', { exact: true })).toBeVisible()
    await two.getByText('Polski', { exact: true }).click()
    const polishOwnItems = two.getByRole('region', { name: 'Twoje przedmioty' })
    await expect(polishOwnItems).toContainText('WyposażenieBrak')
    await expect(polishOwnItems).not.toContainText('Serce')
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({ path: testInfo.outputPath('heart-equipped-polish-320.png'), fullPage: true })

    await one.setViewportSize({ width: 320, height: 844 })
    await one.getByText('Polski', { exact: true }).click()
    const collectorOwnItems = one.getByRole('region', { name: 'Twoje przedmioty' })
    await expect(collectorOwnItems).toContainText('WyposażenieSerce')
    expect(await one.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await one.screenshot({
      path: testInfo.outputPath('heart-collector-equipped-polish-320.png'), fullPage: true,
    })
    await one.getByRole('region', { name: 'Dziedziniec Grenadiers' }).click()
    await plant(one, games[0])
    await Promise.all(games.map((game) => expect.poll(() => Boolean(protectionFrame(game))).toBe(true)))
    for (const game of games) {
      const frame = protectionFrame(game)!
      if (frame.type !== 'snapshot') throw new Error('Protection requires a state publication')
      const protectedActor = visibleActor(frame, 'one')!
      expect(protectedActor.alive).toBe(true)
      expect(protectedActor.inventory.equipment).toBeNull()
      expect(protectedActor.damageGraceRemainingTicks).toBeGreaterThan(0)
      expect(frame.room.phase.type).toBe('running')
    }
    await expect(collectorOwnItems).toContainText('WyposażenieBrak')
    await expect(collectorOwnItems.getByLabel(/^Ochrona przed obrażeniami aktywna, pozostało 0:0[1-9]$/))
      .not.toHaveText('0:00')
    await expect.poll(() => visibleActor(latestSnapshot(games[0]), 'one')
      ?.damageGraceRemainingTicks, { intervals: [15] }).toBe(0)
    await expect(collectorOwnItems).toContainText('Ochrona przed obrażeniami0:00')
    expect(visibleActor(latestSnapshot(games[0]), 'one')?.alive).toBe(true)
    await plant(one, games[0])
    await Promise.all(games.map((game) => expect.poll(() =>
      visibleActor(latestSnapshot(game), 'one')?.alive).toBe(false)))
    const prevented = games[0].frames.flatMap((frame) => frame.type === 'snapshot'
      ? frame.events.filter((event) => event.kind.type === 'damagePrevented') : [])
    expect(prevented).toHaveLength(1)
    expect(latestSnapshot(games[0])?.room.lastRound?.result).toEqual({ type: 'winner', seat: 'two' })
  } finally {
    await testInfo.attach('heart-public-summary', {
      contentType: 'application/json',
      body: JSON.stringify(games.map((game) => ({
        commands: game.commands.slice(-64),
        snapshots: game.frames.filter((frame) => frame.type === 'snapshot').slice(-160),
      }))),
    })
    await Promise.all(contexts.map((context) => context.close()))
  }
})
