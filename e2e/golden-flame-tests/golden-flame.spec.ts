import { expect, test, type Page } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'
import type { GameServerMessage } from '../../web/src/game/v8/schema'

test.beforeAll(() => {
  expect(process.env.E2E_GOLDEN_FLAME_FIXTURE, 'Configure the documented production Golden Flame seed')
    .toBe('arena-v2-golden-flame')
})

async function plant(page: Page, game: ObservedGame): Promise<void> {
  const previous = latestSnapshot(game)!.state.bombs.map((bomb) => bomb.id)
  await page.keyboard.press('Space')
  await expect.poll(() => latestSnapshot(game)?.state.bombs
    .some((bomb) => !previous.includes(bomb.id)), { intervals: [15] }).toBe(true)
}

const goldenTicks = (game: ObservedGame) => visibleActor(latestSnapshot(game), 'one')?.effects
  .find((effect) => effect.kind === 'goldenFlame')?.remainingTicks ?? 0

const publicRoom = (game: ObservedGame) => game.frames.filter((frame) =>
  frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room

function collectionTick(game: ObservedGame): number {
  const event = game.frames.flatMap((frame) => frame.type === 'snapshot' ? frame.events : [])
    .find(({ kind }) => kind.type === 'pickupCollected' && kind.seat === 'one' && kind.kind === 'goldenFlame')
  expect(event, 'The real pickup event establishes the effect deadline').toBeDefined()
  return event!.tick
}

function expectExactGoldenDeadline(game: ObservedGame, collectedAt: number): void {
  const expiresAt = collectedAt + 600
  const frames = game.frames.filter((frame): frame is Extract<GameServerMessage, { type: 'snapshot' }> => frame.type === 'snapshot'
    && frame.state.physicalTick >= collectedAt)
  expect(frames.length).toBeGreaterThan(1)
  for (const frame of frames) {
    const effect = visibleActor(frame, 'one')?.effects.find((value) => value.kind === 'goldenFlame')
    expect(effect?.remainingTicks ?? 0, `Golden Flame at physical tick ${frame.state.physicalTick}`)
      .toBe(Math.max(0, expiresAt - frame.state.physicalTick))
  }
  const firstExpired = frames.find((frame) => frame.state.physicalTick >= expiresAt)
  expect(firstExpired, 'A publication must cross the exact effect deadline').toBeDefined()
  expect(frames.some((frame) => frame.state.physicalTick < expiresAt)).toBe(true)
}

test('two browsers collect Golden Flame and restore base bomb range after its exact deadline', async ({ browser }, testInfo) => {
  const contexts = await Promise.all([
    browser.newContext({ viewport: { width: 1440, height: 1050 } }),
    browser.newContext({ viewport: { width: 320, height: 844 }, reducedMotion: 'reduce' }),
  ])
  const games: ObservedGame[] = []
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    await installTransportProbe(one)
    games.push(observeGame(one), observeGame(two))
    await Promise.all([one, two].map((page) => page.goto('/')))
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await expect(one.getByRole('checkbox', { name: 'Heart', exact: true })).toBeChecked()
    await expect(one.getByRole('checkbox', { name: 'Golden flame', exact: true })).not.toBeChecked()
    await one.getByRole('checkbox', { name: 'Heart', exact: true }).uncheck()
    await one.getByRole('checkbox', { name: 'Golden flame', exact: true }).check()
    await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await expect.poll(() => publicRoom(games[0])?.settings.enabledItems).toEqual(['goldenFlame'])
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await Promise.all([one, two].map((page) => page.getByRole('button', { name: 'Ready', exact: true }).click()))
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
    expect(latestSnapshot(games[0])!.room).toMatchObject({
      rulesVersion: 5, mapId: 'generated_v2', availableItems: ['heart', 'shield', 'goldenFlame', 'shortFuse', 'shortFlame'],
      settings: { enabledItems: ['goldenFlame'] },
    })
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await moveTo(one, games[0], { column: 2, row: 1 })
    await plant(one, games[0])
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length, { intervals: [30] }).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])?.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'goldenFlame',
    })
    await one.screenshot({ path: testInfo.outputPath('golden-flame-exposed-desktop.png'), fullPage: true })
    await two.getByText('Polski', { exact: true }).click()
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({
      path: testInfo.outputPath('golden-flame-exposed-polish-320.png'), fullPage: true,
    })
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 3, row: 1 })
    await Promise.all(games.map((game) => expect.poll(() => goldenTicks(game)).toBeGreaterThan(0)))
    const collectedAt = games.map(collectionTick)
    expect(collectedAt[0]).toBe(collectedAt[1])

    await plant(one, games[0])
    const goldenBomb = latestSnapshot(games[0])!.state.bombs.find((bomb) => bomb.range === 8)
    if (!goldenBomb) throw new Error('Captured range-eight bomb expected')
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.state.bombs)
      .toContainEqual(expect.objectContaining({ id: goldenBomb.id, range: 8 }))))
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length, { intervals: [30] }).toBe(0)
    for (const game of games) {
      const detonation = game.frames.flatMap((frame) => frame.type === 'snapshot' ? frame.events : [])
        .find(({ kind }) => kind.type === 'bombDetonated' && kind.bomb.id === goldenBomb.id)
      if (!detonation || detonation.kind.type !== 'bombDetonated') {
        throw new Error('The range-eight bomb must actually detonate')
      }
      expect(detonation.tick - detonation.kind.bomb.placedAt).toBe(180)
      expect(detonation.kind.bomb.trigger).toEqual({ type: 'timed', remainingTicks: 0, paused: false })
    }
    expect(visibleActor(latestSnapshot(games[0]), 'one')?.alive).toBe(true)
    await Promise.all(games.map((game) => expect.poll(() => goldenTicks(game)).toBeGreaterThan(0)))
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({ path: testInfo.outputPath('golden-flame-active-polish-320.png'), fullPage: true })
    await disconnectTransport(one)
    await expect.poll(async () => (await admissionBaseline(one))?.actionFloor).toBe(2)
    expect((await admissionBaseline(one))?.stableCapability).toBe(true)
    await Promise.all(games.map((game) => expect.poll(() => goldenTicks(game)).toBeGreaterThan(0)))
    await Promise.all(games.map((game) => expect.poll(() => goldenTicks(game), { intervals: [15] }).toBe(0)))
    games.forEach((game, index) => expectExactGoldenDeadline(game, collectedAt[index]))
    expect(visibleActor(latestSnapshot(games[0]), 'one')?.alive).toBe(true)
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await plant(one, games[0])
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.state.bombs)
      .toContainEqual(expect.objectContaining({ range: 2 }))))
    await Promise.all(games.map((game) => expect.poll(() =>
      visibleActor(latestSnapshot(game), 'one')?.alive).toBe(false)))
    expect(latestSnapshot(games[0])?.room.lastRound?.result).toEqual({ type: 'winner', seat: 'two' })
  } finally {
    try {
      await testInfo.attach('golden-flame-public-summary', {
        contentType: 'application/json',
        body: JSON.stringify(games.map((game) => ({
          commands: game.commands.slice(-64),
          snapshots: game.frames.filter((frame) => frame.type === 'snapshot').slice(-300),
        }))),
      })
    } finally { await Promise.all(contexts.map((context) => context.close())) }
  }
})
