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
import type { GameServerMessage } from '../../web/src/game/v13/schema'

test.beforeAll(() => {
  expect(process.env.E2E_SHORT_FUSE_FIXTURE, 'Configure the documented production Short Fuse seed')
    .toBe('arena-v2-short-fuse')
})

const shortTicks = (game: ObservedGame) => effectTicks(game, 'shortFuse')

function capturedBomb(game: ObservedGame, expectedTicks: number, id?: number): number {
  const snapshot = id === undefined ? latestSnapshot(game)! : game.frames.filter(
    (frame): frame is Extract<GameServerMessage, { type: 'snapshot' }> => frame.type === 'snapshot'
      && frame.state.bombs.some((bomb) => bomb.id === id),
  ).at(-1)!
  const bomb = id === undefined ? snapshot.state.bombs[0] : snapshot.state.bombs.find((value) => value.id === id)
  if (!bomb || bomb.trigger.type !== 'timed') throw new Error('Ordinary timed bomb expected')
  expect(bomb.trigger.paused).toBe(false)
  expect(snapshot.state.physicalTick - bomb.placedAt + bomb.trigger.remainingTicks).toBe(expectedTicks)
  return bomb.id
}

async function expectDetonation(game: ObservedGame, id: number, expectedTicks: number): Promise<void> {
  const find = () => game.frames.flatMap((frame) => frame.type === 'snapshot' ? frame.events : [])
    .find(({ kind }) => kind.type === 'bombDetonated' && kind.bomb.id === id)
  await expect.poll(find).toBeDefined()
  const event = find()
  if (!event || event.kind.type !== 'bombDetonated') throw new Error('Natural detonation event expected')
  expect(event.tick - event.kind.bomb.placedAt).toBe(expectedTicks)
  expect(event.kind.bomb.trigger).toEqual({ type: 'timed', remainingTicks: 0, paused: false })
}

async function expectCapturedOnBoth(
  games: ObservedGame[], expectedTicks: number, id: number,
): Promise<number> {
  capturedBomb(games[0], expectedTicks, id)
  await Promise.all(games.map(async (game) => {
    await expect.poll(() => game.frames.some((frame) => frame.type === 'snapshot'
      && frame.state.bombs.some((bomb) => bomb.id === id))).toBe(true)
    capturedBomb(game, expectedTicks, id)
  }))
  return id
}

test('two browsers collect Short Fuse and restore ordinary bomb timing after its exact deadline', async ({ browser }, testInfo) => {
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
    await applyItemSettings(one, games[0], [{ id: 'shortFuse', label: 'Short fuse' }],
      () => one.getByLabel('Bomb fuse').selectOption('300'))
    await expect.poll(() => latestPublicRoom(games[0])?.settings.enabledItems).toEqual(['shortFuse'])
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await Promise.all([one, two].map((page) => page.getByRole('button', { name: 'Ready', exact: true }).click()))
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
    expect(latestSnapshot(games[0])!.room).toMatchObject({
      rulesVersion: 5, mapId: 'generated_v2', availableItems: ['kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'oilBarrel', 'teleporter'],
      settings: { enabledItems: ['shortFuse'], bombFuseTicks: 300 },
    })
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await moveTo(one, games[0], { column: 2, row: 1 })
    await plantBombWithNativeSpace(one, games[0])
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length, { intervals: [30] }).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])?.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'shortFuse',
    })
    await one.screenshot({ path: testInfo.outputPath('short-fuse-exposed-desktop.png'), fullPage: true })
    await two.getByText('Polski', { exact: true }).click()
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({
      path: testInfo.outputPath('short-fuse-exposed-polish-320.png'), fullPage: true,
    })
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 3, row: 1 })
    await Promise.all(games.map((game) => expect.poll(() => shortTicks(game)).toBeGreaterThan(0)))
    const collectedAt = games.map((game) => ownCollectionTick(game, 'shortFuse'))
    expect(collectedAt[0]).toBe(collectedAt[1])
    const shortBomb = await plantBombWithNativeSpace(one, games[0])
    await expectCapturedOnBoth(games, 150, shortBomb)
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length, { intervals: [30] }).toBe(0)
    expect(visibleActor(latestSnapshot(games[0]), 'one')?.alive).toBe(true)
    await Promise.all(games.map((game) => expectDetonation(game, shortBomb, 150)))
    await Promise.all(games.map((game) => expect.poll(() => shortTicks(game)).toBeGreaterThan(0)))
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({ path: testInfo.outputPath('short-fuse-active-polish-320.png'), fullPage: true })
    await disconnectTransport(one)
    await expect.poll(async () => (await admissionBaseline(one))?.actionFloor).toBe(2)
    expect((await admissionBaseline(one))?.stableCapability).toBe(true)
    await Promise.all(games.map((game) => expect.poll(() => shortTicks(game)).toBeGreaterThan(0)))
    await Promise.all(games.map((game) => expect.poll(() => shortTicks(game), { intervals: [15] }).toBe(0)))
    games.forEach((game, index) => expectExact600TickExpiry(
      game, collectedAt[index], 'shortFuse', 'Short Fuse',
    ))
    expect(visibleActor(latestSnapshot(games[0]), 'one')?.alive).toBe(true)
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    const ordinaryBomb = await plantBombWithNativeSpace(one, games[0])
    await expectCapturedOnBoth(games, 300, ordinaryBomb)
    await Promise.all(games.map((game) => expect.poll(() =>
      visibleActor(latestSnapshot(game), 'one')?.alive).toBe(false)))
    await Promise.all(games.map((game) => expectDetonation(game, ordinaryBomb, 300)))
    expect(latestSnapshot(games[0])?.room.lastRound?.result).toEqual({ type: 'winner', seat: 'two' })
  } finally {
    try {
      await testInfo.attach('short-fuse-public-summary', {
        contentType: 'application/json',
        body: JSON.stringify(games.map((game) => ({
          commands: game.commands.slice(-64),
          snapshots: game.frames.filter((frame) => frame.type === 'snapshot').slice(-300),
        }))),
      })
    } finally { await Promise.all(contexts.map((context) => context.close())) }
  }
})
