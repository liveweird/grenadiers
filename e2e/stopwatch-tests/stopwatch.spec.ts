import { expect, test, type Page } from '@playwright/test'
import { expectPaintedCanvas, latestSnapshot, observeGame, visibleActor,
  type ObservedGame } from '../tests/support/duel'
import { moveSeatTo } from '../tests/support/movement'
import type { GameServerMessage, Seat } from '../../web/src/game/v10/schema'
test.beforeAll(() => expect(process.env.E2E_STOPWATCH_FIXTURE).toBe('arena-v2-stopwatch'))
const snapshots = (game: ObservedGame) => game.frames.filter(
  (frame): frame is Extract<GameServerMessage, { type: 'snapshot' }> => frame.type === 'snapshot',
)
const stopwatchTicks = (game: ObservedGame, seat: Seat) =>
  visibleActor(latestSnapshot(game), seat)?.effects
    .find(({ kind }) => kind === 'stopwatch')?.remainingTicks ?? 0
const frozen = (game: ObservedGame, seat: Seat) =>
  visibleActor(latestSnapshot(game), seat)?.voluntaryActionsFrozen ?? false
const placementIds = (game: ObservedGame, owner: number) => snapshots(game)
  .flatMap(({ events }) => events).flatMap(({ kind }) =>
    kind.type === 'bombPlaced' && kind.bomb.owner === owner ? [kind.bomb.id] : [])
async function plant(page: Page, game: ObservedGame): Promise<number> {
  const previous = new Set(latestSnapshot(game)!.state.bombs.map(({ id }) => id))
  await page.keyboard.press('Space')
  await expect.poll(() => latestSnapshot(game)?.state.bombs
    .find(({ id }) => !previous.has(id))?.id).toBeDefined()
  return latestSnapshot(game)!.state.bombs.find(({ id }) => !previous.has(id))!.id
}
async function reveal(
  page: Page, game: ObservedGame, seat: Seat,
  bombCell: { column: number; row: number }, escape: readonly { column: number; row: number }[],
) {
  await moveSeatTo(page, game, seat, bombCell)
  await page.keyboard.press('Space')
  for (const cell of escape) await moveSeatTo(page, game, seat, cell)
}
async function expectPausedAcrossTicks(game: ObservedGame, id: number): Promise<void> {
  await expect.poll(() => snapshots(game).filter(({ state }) => state.bombs.some((bomb) =>
    bomb.id === id && bomb.trigger.type === 'timed' && bomb.trigger.paused)).length)
    .toBeGreaterThanOrEqual(2)
  const observations = snapshots(game).flatMap(({ state }) => {
    const bomb = state.bombs.find(({ id: candidate }) => candidate === id)
    return bomb?.trigger.type === 'timed' && bomb.trigger.paused
      ? [{ tick: state.physicalTick, remaining: bomb.trigger.remainingTicks }] : []
  })
  expect(new Set(observations.map(({ tick }) => tick)).size).toBeGreaterThan(1)
  expect(new Set(observations.map(({ remaining }) => remaining)).size).toBe(1)
}
async function expectNaturalDetonation(game: ObservedGame, id: number, pausedTicks: number) {
  const event = () => snapshots(game).flatMap(({ events }) => events).find(({ kind }) =>
    kind.type === 'bombDetonated' && kind.bomb.id === id)
  await expect.poll(event).toBeDefined()
  const found = event()!
  if (found.kind.type !== 'bombDetonated') throw new Error('detonation expected')
  expect(found.tick - found.kind.bomb.placedAt).toBe(180 + pausedTicks)
  expect(found.kind.bomb.trigger).toEqual({ type: 'timed', remainingTicks: 0, paused: false })
}
test('Stopwatch freezes voluntary actions while Timed bombs pause', async ({ browser }, info) => {
  const contexts = await Promise.all([
    browser.newContext({ viewport: { width: 1440, height: 1050 } }),
    browser.newContext({ viewport: { width: 320, height: 844 }, reducedMotion: 'reduce' }),
  ])
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    const games = [observeGame(one), observeGame(two)]
    await Promise.all([one, two].map((page) => page.goto('/')))
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await one.getByRole('checkbox', { name: 'Heart', exact: true }).uncheck()
    await one.getByRole('checkbox', { name: 'Stopwatch', exact: true }).check()
    await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await two.getByText('Polski', { exact: true }).click()
    await two.getByRole('textbox', { name: 'Kod pokoju', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Dołącz do pokoju', exact: true }).click()
    await Promise.all([
      one.getByRole('button', { name: 'Ready', exact: true }).click(),
      two.getByRole('button', { name: 'Gotowy', exact: true }).click(),
    ])
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type)
      .toBe('running')))
    await Promise.all([
      one.getByRole('region', { name: 'Grenadiers courtyard', exact: true }).click(),
      two.getByRole('region', { name: 'Dziedziniec Grenadiers', exact: true }).click(),
    ])
    expect(latestSnapshot(games[0])!.room.availableItems).toEqual([
      'kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch',
      'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame',
      'longFuse', 'reverseControls', 'invisibleBombs', 'teleporter',
    ])
    await Promise.all([
      reveal(one, games[0], 'one', { column: 2, row: 1 },
        [{ column: 1, row: 1 }, { column: 1, row: 2 }]),
      reveal(two, games[1], 'two', { column: 12, row: 11 },
        [{ column: 13, row: 11 }, { column: 13, row: 10 }]),
    ])
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length).toBe(0)
    await Promise.all([one, two].map(expectPaintedCanvas))
    await one.screenshot({ path: info.outputPath('stopwatch-exposed-desktop-en.png'), fullPage: true })
    await two.screenshot({ path: info.outputPath('stopwatch-exposed-polish-320-reduced.png'), fullPage: true })

    await moveSeatTo(one, games[0], 'one', { column: 1, row: 2 })
    const original = await plant(one, games[0])
    await moveSeatTo(one, games[0], 'one', { column: 1, row: 1 })
    await moveSeatTo(one, games[0], 'one', { column: 3, row: 1 })
    await Promise.all(games.map((game) => expect.poll(() => stopwatchTicks(game, 'one'))
      .toBeGreaterThan(0)))
    await Promise.all(games.map((game) => expectPausedAcrossTicks(game, original)))
    const p2Starts = games.map((game) => visibleActor(latestSnapshot(game), 'two')!.position)
    const p2Start = p2Starts[1]
    const p2BombCount = latestSnapshot(games[1])!.state.bombs.length
    const beforeP2BombIds = games.map((game) => latestSnapshot(game)!.state.bombs.map(({ id }) => id))
    const beforeP2ActionTicks = games.map((game) => latestSnapshot(game)!.state.physicalTick)
    await two.keyboard.down('ArrowDown')
    await two.keyboard.press('Space')
    await expect.poll(() => frozen(games[1], 'two')).toBe(true)
    await expect.poll(() => games.every((game, index) =>
      latestSnapshot(game)!.state.physicalTick >= beforeP2ActionTicks[index] + 3)).toBe(true)
    expect(games.map((game) => visibleActor(latestSnapshot(game), 'two')!.position)).toEqual(p2Starts)
    expect(games.map((game) => latestSnapshot(game)!.state.bombs.map(({ id }) => id)))
      .toEqual(beforeP2BombIds)
    await moveSeatTo(one, games[0], 'one', { column: 2, row: 1 })
    await moveSeatTo(one, games[0], 'one', { column: 3, row: 1 })
    await expect.poll(() => frozen(games[1], 'two')).toBe(false)
    await expect.poll(() => visibleActor(latestSnapshot(games[1]), 'two')!.position.y)
      .toBeGreaterThan(p2Start.y)
    await two.keyboard.up('ArrowDown')
    expect(latestSnapshot(games[1])!.state.bombs).toHaveLength(p2BombCount)
    await Promise.all(games.map((game) => expectNaturalDetonation(game, original, 179)))
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length).toBe(0)

    await moveSeatTo(two, games[1], 'two', { column: 13, row: 11 })
    await moveSeatTo(two, games[1], 'two', { column: 11, row: 11 })
    await Promise.all(games.map((game) => expect.poll(() => stopwatchTicks(game, 'two'))
      .toBeGreaterThan(0)))
    const secondDeadlines = games.map((game) => {
      const snapshot = latestSnapshot(game)!
      const remainingTicks = visibleActor(snapshot, 'two')!.effects
        .find(({ kind }) => kind === 'stopwatch')!.remainingTicks
      expect(remainingTicks).toBeGreaterThan(0)
      return snapshot.state.physicalTick + remainingTicks
    })
    expect(secondDeadlines[1]).toBe(secondDeadlines[0])
    const secondDeadline = secondDeadlines[0]
    const p1Start = visibleActor(latestSnapshot(games[0]), 'one')!.position
    const p1Id = visibleActor(latestSnapshot(games[0]), 'one')!.id
    const beforeBombIds = games.map((game) => latestSnapshot(game)!.state.bombs.map(({ id }) => id))
    const beforeP1Placements = games.map((game) => placementIds(game, p1Id))
    const beforeActionTicks = games.map((game) => latestSnapshot(game)!.state.physicalTick)
    await one.keyboard.down('ArrowLeft')
    await one.keyboard.press('Space')
    await expect.poll(() => frozen(games[0], 'one')).toBe(true)
    await expect.poll(() => games.every((game, index) =>
      latestSnapshot(game)!.state.physicalTick >= beforeActionTicks[index] + 3)).toBe(true)
    expect(visibleActor(latestSnapshot(games[0]), 'one')!.position).toEqual(p1Start)
    expect(games.map((game) => placementIds(game, p1Id))).toEqual(beforeP1Placements)
    expect(games.map((game) => latestSnapshot(game)!.state.bombs.map(({ id }) => id)))
      .toEqual(beforeBombIds)
    const fresh = await plant(two, games[1])
    await expect.poll(() => games.map((game) => latestSnapshot(game)!.state.bombs
      .filter(({ id }) => !beforeBombIds[0].includes(id)).map(({ id }) => id))).toEqual([[fresh], [fresh]])
    const placements = games.map((game) => snapshots(game).flatMap(({ events }) => events)
      .find(({ kind }) => kind.type === 'bombPlaced' && kind.bomb.id === fresh))
    expect(placements.every((event) => event?.kind.type === 'bombPlaced')).toBe(true)
    const placedAts = placements.map((event) => event?.kind.type === 'bombPlaced'
      ? event.kind.bomb.placedAt : -1)
    expect(placedAts[1]).toBe(placedAts[0])
    expect(placements.map((event) => event?.kind.type === 'bombPlaced'
      ? event.kind.bomb.trigger : null)).toEqual([
      { type: 'timed', remainingTicks: 180, paused: true },
      { type: 'timed', remainingTicks: 180, paused: true },
    ])
    const expectedPausedTicks = secondDeadline - placedAts[0] - 1
    expect(placedAts[0]).toBeLessThan(secondDeadline)
    expect(expectedPausedTicks).toBeGreaterThanOrEqual(0)
    expect(expectedPausedTicks).toBeLessThan(180)
    await moveSeatTo(two, games[1], 'two', { column: 12, row: 11 })
    await moveSeatTo(two, games[1], 'two', { column: 13, row: 11 })
    await moveSeatTo(two, games[1], 'two', { column: 13, row: 10 })
    await expect.poll(() => frozen(games[0], 'one')).toBe(false)
    await expect.poll(() => visibleActor(latestSnapshot(games[0]), 'one')!.position.x)
      .toBeLessThan(p1Start.x)
    await one.keyboard.up('ArrowLeft')
    await expect.poll(() => games.map((game) => placementIds(game, p1Id)))
      .toEqual(beforeP1Placements)
    await Promise.all(games.map((game) =>
      expectNaturalDetonation(game, fresh, expectedPausedTicks)))
  } finally {
    await Promise.all(contexts.map(async (context) => { try {
      for (const page of context.pages()) await page.keyboard.up('ArrowDown').catch(() => undefined)
      for (const page of context.pages()) await page.keyboard.up('ArrowLeft').catch(() => undefined)
    } finally { await context.close() } }))
  }
})
