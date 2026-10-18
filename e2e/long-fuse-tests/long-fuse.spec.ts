import { expect, test, type Page } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'
import type { GameServerMessage } from '../../web/src/game/v8/schema'

test.beforeAll(() => expect(process.env.E2E_LONG_FUSE_FIXTURE).toBe('arena-v2-long-fuse'))

const ticks = (game: ObservedGame) => visibleActor(latestSnapshot(game), 'one')?.effects
  .find((effect) => effect.kind === 'longFuse')?.remainingTicks ?? 0
const room = (game: ObservedGame) => game.frames.filter((frame) =>
  frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room

async function plant(page: Page, game: ObservedGame): Promise<number> {
  const old = latestSnapshot(game)!.state.bombs.map(({ id }) => id)
  await page.keyboard.press('Space')
  await expect.poll(() => latestSnapshot(game)?.state.bombs.find(({ id }) => !old.includes(id))?.id).toBeTruthy()
  return latestSnapshot(game)!.state.bombs.find(({ id }) => !old.includes(id))!.id
}

function captured(game: ObservedGame, id: number, duration: number): boolean {
  const frame = game.frames.filter((value) => value.type === 'snapshot'
    && value.state.bombs.some((bomb) => bomb.id === id)).at(-1)
  if (!frame || frame.type !== 'snapshot') return false
  const bomb = frame.state.bombs.find((value) => value.id === id)!
  return bomb.trigger.type === 'timed' && !bomb.trigger.paused
    && frame.state.physicalTick - bomb.placedAt + bomb.trigger.remainingTicks === duration
}

function collectedAt(game: ObservedGame): number {
  const event = game.frames.flatMap((frame) => frame.type === 'snapshot' ? frame.events : [])
    .find(({ kind }) => kind.type === 'pickupCollected' && kind.kind === 'longFuse')
  expect(event).toBeDefined()
  return event!.tick
}

function expectExactExpiry(game: ObservedGame, collectionTick: number): void {
  const expiry = collectionTick + 600
  const frames = game.frames.filter((frame): frame is Extract<GameServerMessage, { type: 'snapshot' }> =>
    frame.type === 'snapshot' && frame.state.physicalTick >= collectionTick)
  expect(frames.some((frame) => frame.state.physicalTick < expiry)).toBe(true)
  expect(frames.some((frame) => frame.state.physicalTick >= expiry)).toBe(true)
  for (const frame of frames) {
    const observed = visibleActor(frame, 'one')?.effects
      .find(({ kind }) => kind === 'longFuse')?.remainingTicks ?? 0
    expect(observed).toBe(Math.max(0, expiry - frame.state.physicalTick))
  }
}

async function expectDetonation(game: ObservedGame, id: number, duration: number): Promise<void> {
  const event = () => game.frames.flatMap((frame) => frame.type === 'snapshot' ? frame.events : [])
    .find(({ kind }) => kind.type === 'bombDetonated' && kind.bomb.id === id)
  await expect.poll(event).toBeDefined()
  const found = event()!
  if (found.kind.type !== 'bombDetonated') throw new Error('detonation expected')
  expect(found.tick - found.kind.bomb.placedAt).toBe(duration)
  expect(found.kind.bomb.trigger).toEqual({ type: 'timed', remainingTicks: 0, paused: false })
}

test('two browsers collect Long Fuse and restore ordinary bomb timing after its exact deadline', async ({ browser }, info) => {
  const contexts = await Promise.all([
    browser.newContext({ viewport: { width: 1440, height: 1050 } }),
    browser.newContext({ viewport: { width: 320, height: 844 }, reducedMotion: 'reduce' }),
  ])
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    await installTransportProbe(one)
    const games = [observeGame(one), observeGame(two)]
    await Promise.all([one, two].map((page) => page.goto('/')))
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await one.getByRole('checkbox', { name: 'Heart', exact: true }).uncheck()
    await one.getByRole('checkbox', { name: 'Long fuse', exact: true }).check()
    await one.getByLabel('Bomb fuse').selectOption('120')
    await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await expect.poll(() => room(games[0])?.settings.enabledItems).toEqual(['longFuse'])
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await Promise.all([one, two].map((page) => page.getByRole('button', { name: 'Ready', exact: true }).click()))
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
    expect(latestSnapshot(games[0])!.room.availableItems).toEqual(
      ['heart', 'shield', 'goldenFlame', 'shortFuse', 'shortFlame', 'longFuse', 'reverseControls'])
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await moveTo(one, games[0], { column: 2, row: 1 }); await plant(one, games[0])
    await moveTo(one, games[0], { column: 1, row: 1 }); await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])?.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'longFuse',
    })
    await one.screenshot({ path: info.outputPath('long-fuse-exposed-desktop.png'), fullPage: true })
    await moveTo(one, games[0], { column: 1, row: 1 }); await moveTo(one, games[0], { column: 3, row: 1 })
    await Promise.all(games.map((game) => expect.poll(() => ticks(game)).toBeGreaterThan(0)))
    const collectionTicks = games.map(collectedAt)
    expect(collectionTicks[0]).toBe(collectionTicks[1])
    const longBomb = await plant(one, games[0])
    await Promise.all(games.map((game) => expect.poll(() => captured(game, longBomb, 240)).toBe(true)))
    await moveTo(one, games[0], { column: 1, row: 1 }); await moveTo(one, games[0], { column: 1, row: 2 })
    await two.getByText('Polski', { exact: true }).click()
    await two.screenshot({ path: info.outputPath('long-fuse-polish-320-after-escape.png'), fullPage: true })
    await Promise.all(games.map((game) => expectDetonation(game, longBomb, 240)))
    await disconnectTransport(one)
    await expect.poll(async () => (await admissionBaseline(one))?.actionFloor).toBe(2)
    await Promise.all(games.map((game) => expect.poll(() => ticks(game)).toBe(0)))
    games.forEach((game, index) => expectExactExpiry(game, collectionTicks[index]))
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    const ordinary = await plant(one, games[0])
    await Promise.all(games.map((game) => expect.poll(() => captured(game, ordinary, 120)).toBe(true)))
    await Promise.all(games.map((game) => expect.poll(() => visibleActor(latestSnapshot(game), 'one')?.alive).toBe(false)))
    await Promise.all(games.map((game) => expectDetonation(game, ordinary, 120)))
  } finally { await Promise.all(contexts.map((context) => context.close())) }
})
