import { createResponsiveBrowserContexts } from '../tests/support/browser-contexts'
import { expect, test } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import {
  effectTicks, expectExact600TickExpiry, latestPublicRoom, ownCollectionTick,
  plantBombWithNativeSpace,
} from '../tests/support/effect-pickups'
import { moveTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'
import { applyItemSettings } from '../tests/support/settings'

test.beforeAll(() => {
  expect(process.env.E2E_SHORT_FLAME_FIXTURE, 'Configure the documented production Short Flame seed')
    .toBe('arena-v3-short-flame')
})

const shortTicks = (game: ObservedGame) => effectTicks(game, 'shortFlame')

test('two browsers collect Short Flame and restore base bomb range after its exact deadline', async ({ browser }, testInfo) => {
  const contexts = await createResponsiveBrowserContexts(browser)
  const games: ObservedGame[] = []
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    await installTransportProbe(one)
    games.push(observeGame(one), observeGame(two))
    await Promise.all([one, two].map((page) => page.goto('/')))
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await applyItemSettings(one, games[0], [{ id: 'shortFlame', label: 'Short flame' }])
    await expect.poll(() => latestPublicRoom(games[0])?.settings.enabledItems).toEqual(['shortFlame'])
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await Promise.all([one, two].map((page) => page.getByRole('button', { name: 'Ready', exact: true }).click()))
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
    expect(latestSnapshot(games[0])!.room).toMatchObject({
      rulesVersion: 10, mapId: 'generated_v2', availableItems: ['kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'oilBarrel', 'teleporter'],
      settings: { enabledItems: ['shortFlame'], environment: {
        stormCloud: false, lairs: { enabled: false, spawnSeconds: 10, maxLivingMobs: 1 },
      } },
    })
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await moveTo(one, games[0], { column: 2, row: 1 })
    await plantBombWithNativeSpace(one, games[0])
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length, { intervals: [30] }).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])?.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'shortFlame',
    })
    await one.screenshot({ path: testInfo.outputPath('short-flame-exposed-desktop.png'), fullPage: true })
    await two.getByText('Polski', { exact: true }).click()
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({
      path: testInfo.outputPath('short-flame-exposed-polish-320.png'), fullPage: true,
    })
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 3, row: 1 })
    await Promise.all(games.map((game) => expect.poll(() => shortTicks(game)).toBeGreaterThan(0)))
    const collectedAt = games.map((game) => ownCollectionTick(game, 'shortFlame'))
    expect(collectedAt[0]).toBe(collectedAt[1])
    const cursedBomb = await plantBombWithNativeSpace(one, games[0])
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.state.bombs)
      .toContainEqual(expect.objectContaining({ id: cursedBomb, range: 1, trigger: {
        type: 'timed', remainingTicks: expect.any(Number), paused: false,
      } }))))
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length, { intervals: [30] }).toBe(0)
    for (const game of games) {
      const detonation = game.frames.flatMap((frame) => frame.type === 'snapshot' ? frame.events : [])
        .find(({ kind }) => kind.type === 'bombDetonated' && kind.bomb.id === cursedBomb)
      if (!detonation || detonation.kind.type !== 'bombDetonated') {
        throw new Error('The cursed bomb must actually detonate')
      }
      expect(detonation.tick - detonation.kind.bomb.placedAt,
        'Short Flame must preserve the ordinary three-second fuse').toBe(180)
      const blast = game.frames.find((frame) => frame.type === 'snapshot'
        && frame.state.physicalTick >= detonation.tick
        && frame.state.flames.some(({ cell }) => cell.column === 2 && cell.row === 1))
      if (!blast || blast.type !== 'snapshot') throw new Error('Short Flame blast snapshot expected')
      expect(blast.state.flames.some(({ cell }) => cell.column === 3 && cell.row === 1)).toBe(true)
      expect(blast.state.flames.some(({ cell }) => cell.column === 1 && cell.row === 1),
        'The known clear west corridor must stop burning after one adjacent cell').toBe(false)
    }
    expect(visibleActor(latestSnapshot(games[0]), 'one')?.alive).toBe(true)
    await Promise.all(games.map((game) => expect.poll(() => shortTicks(game)).toBeGreaterThan(0)))
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({ path: testInfo.outputPath('short-flame-active-polish-320.png'), fullPage: true })
    await disconnectTransport(one)
    await expect.poll(async () => (await admissionBaseline(one))?.actionFloor).toBe(2)
    expect((await admissionBaseline(one))?.stableCapability).toBe(true)
    await Promise.all(games.map((game) => expect.poll(() => shortTicks(game)).toBeGreaterThan(0)))
    await Promise.all(games.map((game) => expect.poll(() => shortTicks(game), { intervals: [15] }).toBe(0)))
    games.forEach((game, index) => expectExact600TickExpiry(
      game, collectedAt[index], 'shortFlame', 'Short Flame',
    ))
    expect(visibleActor(latestSnapshot(games[0]), 'one')?.alive).toBe(true)
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await plantBombWithNativeSpace(one, games[0])
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.state.bombs)
      .toContainEqual(expect.objectContaining({ range: 2 }))))
    await Promise.all(games.map((game) => expect.poll(() =>
      visibleActor(latestSnapshot(game), 'one')?.alive).toBe(false)))
    expect(latestSnapshot(games[0])?.room.lastRound?.result).toEqual({ type: 'winner', seat: 'two' })
  } finally {
    try {
      await testInfo.attach('short-flame-public-summary', {
        contentType: 'application/json',
        body: JSON.stringify(games.map((game) => ({
          commands: game.commands.slice(-64),
          snapshots: game.frames.filter((frame) => frame.type === 'snapshot').slice(-300),
        }))),
      })
    } finally { await Promise.all(contexts.map((context) => context.close())) }
  }
})
