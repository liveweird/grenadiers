import { expect, test, type Page } from '@playwright/test'
import { expectPaintedCanvas, latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveSeatTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'
import { arenaTilePixels } from '../tests/support/compositor'
import type { GameServerMessage } from '../../web/src/game/v11/schema'

test.beforeAll(() => expect(process.env.E2E_INVISIBLE_BOMBS_FIXTURE).toBe('arena-v2-invisible-bombs'))

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
const snapshots = (game: ObservedGame): Snapshot[] => game.frames.filter(
  (frame): frame is Snapshot => frame.type === 'snapshot')
const effectTicks = (game: ObservedGame) => visibleActor(latestSnapshot(game), 'one')?.effects
  .find(({ kind }) => kind === 'invisibleBombs')?.remainingTicks ?? 0
const bombEvents = (game: ObservedGame, since: number) => snapshots(game)
  .filter(({ state }) => state.physicalTick >= since).flatMap(({ events }) => events)
  .filter(({ kind }) => kind.type === 'bombPlaced' || kind.type === 'bombDetonated')

async function plant(page: Page, actorGame: ObservedGame, sightedGame: ObservedGame,
  cell: { column: number; row: number }) {
  const beforeFrame = latestSnapshot(sightedGame)!
  const before = new Set(beforeFrame.state.bombs.map(({ id }) => id))
  const actionFloor = actorGame.commands.length
  await page.keyboard.press('Space')
  await expect.poll(() => actorGame.commands.slice(actionFloor)).toEqual([
    expect.objectContaining({ type: 'manualAction', action: 'bomb' }),
  ])
  await expect.poll(() => latestSnapshot(sightedGame)!.state.bombs.find((bomb) =>
    !before.has(bomb.id) && bomb.cell.column === cell.column && bomb.cell.row === cell.row)).toBeTruthy()
  const frame = latestSnapshot(sightedGame)!
  expect(frame.room.roundId).toBe(beforeFrame.room.roundId)
  expect(frame.room.revision).toBeGreaterThan(beforeFrame.room.revision)
  expect(frame.room.tick).toBeGreaterThanOrEqual(beforeFrame.room.tick)
  return frame.state.bombs.find((bomb) => !before.has(bomb.id))!
}

test('Invisible Bombs hides live bombs and cues until exact expiry', async ({ browser }, info) => {
  const contexts = await Promise.all([
    browser.newContext({ viewport: { width: 1440, height: 1050 } }),
    browser.newContext({ viewport: { width: 320, height: 844 }, reducedMotion: 'reduce' }),
  ])
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    await Promise.all([one, two].map(installTransportProbe))
    const games = [observeGame(one), observeGame(two)]
    await Promise.all([one.goto('/'), two.goto('/')])
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await one.getByRole('checkbox', { name: 'Heart', exact: true }).uncheck()
    await one.getByRole('checkbox', { name: 'Invisible bombs', exact: true }).check()
    await one.getByLabel('Bomb fuse').selectOption('300')
    await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await two.getByText('Polski', { exact: true }).click()
    await two.getByRole('textbox', { name: 'Kod pokoju', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Dołącz do pokoju', exact: true }).click()
    await Promise.all([one.getByRole('button', { name: 'Ready', exact: true }).click(),
      two.getByRole('button', { name: 'Gotowy', exact: true }).click()])
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await two.getByRole('region', { name: 'Dziedziniec Grenadierów' }).click()
    await moveSeatTo(one, games[0], 'one', { column: 2, row: 1 })
    await plant(one, games[0], games[0], { column: 2, row: 1 })
    await moveSeatTo(one, games[0], 'one', { column: 1, row: 1 }); await moveSeatTo(one, games[0], 'one', { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length + latestSnapshot(games[0])!.state.flames.length).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])!.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'invisibleBombs' })
    await Promise.all([one, two].map(expectPaintedCanvas))
    await one.screenshot({ path: info.outputPath('invisible-bombs-before-collection-desktop-en.png'), fullPage: true })
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({ path: info.outputPath('invisible-bombs-before-collection-polish-320-reduced.png'), fullPage: true })

    await moveSeatTo(one, games[0], 'one', { column: 1, row: 1 })
    await moveSeatTo(one, games[0], 'one', { column: 3, row: 1 })
    await expect.poll(() => effectTicks(games[0])).toBeGreaterThan(0)
    const collection = snapshots(games[0]).flatMap(({ events }) => events).find(({ kind }) =>
      kind.type === 'pickupCollected' && kind.seat === 'one' && kind.kind === 'invisibleBombs')!
    const deadline = collection.tick + 600
    const privacyStart = latestSnapshot(games[0])!.state.physicalTick
    await moveSeatTo(one, games[0], 'one', { column: 2, row: 1 })
    const floor = await arenaTilePixels(one, 3, 1)
    await moveSeatTo(one, games[0], 'one', { column: 3, row: 1 })
    const ownBomb = await plant(one, games[0], games[1], { column: 3, row: 1 })
    await moveSeatTo(one, games[0], 'one', { column: 2, row: 1 }); await moveSeatTo(one, games[0], 'one', { column: 1, row: 2 })
    await moveSeatTo(two, games[1], 'two', { column: 12, row: 11 })
    const rivalBomb = await plant(two, games[1], games[1], { column: 12, row: 11 })
    await moveSeatTo(two, games[1], 'two', { column: 13, row: 11 }); await moveSeatTo(two, games[1], 'two', { column: 13, row: 10 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs).toEqual([])
    expect(visibleActor(latestSnapshot(games[0]), 'one')?.liveBombCount).toBe(1)
    await expect.poll(() => latestSnapshot(games[1])!.state.bombs.map(({ id }) => id))
      .toEqual(expect.arrayContaining([ownBomb.id, rivalBomb.id]))
    expect(bombEvents(games[0], privacyStart)).toHaveLength(0)
    expect(await arenaTilePixels(one, 3, 1)).toEqual(floor)
    expect(await arenaTilePixels(two, 3, 1)).not.toEqual(floor)

    await disconnectTransport(one)
    await expect.poll(async () => (await admissionBaseline(one))?.tick).toBeGreaterThan(privacyStart)
    expect(latestSnapshot(games[0])!.state.bombs).toEqual([])
    expect(bombEvents(games[0], privacyStart)).toHaveLength(0)
    await expect.poll(() => snapshots(games[0]).some(({ state }) => state.physicalTick >= privacyStart
      && state.flames.length > 0)).toBe(true)
    await expect.poll(() => latestSnapshot(games[1])!.state.bombs.length + latestSnapshot(games[1])!.state.flames.length).toBe(0)
    await moveSeatTo(two, games[1], 'two', { column: 13, row: 11 })
    await moveSeatTo(two, games[1], 'two', { column: 12, row: 11 })
    const survivor = await plant(two, games[1], games[1], { column: 12, row: 11 })
    expect(latestSnapshot(games[1])!.state.physicalTick).toBeGreaterThanOrEqual(deadline - 210)
    expect(latestSnapshot(games[1])!.state.physicalTick).toBeLessThanOrEqual(deadline - 130)
    await moveSeatTo(two, games[1], 'two', { column: 13, row: 11 }); await moveSeatTo(two, games[1], 'two', { column: 13, row: 10 })
    await expect.poll(() => latestSnapshot(games[0])!.state.physicalTick).toBeGreaterThanOrEqual(deadline)
    const before = snapshots(games[0]).filter(({ state }) => state.physicalTick < deadline)
      .findLast(({ state }) => snapshots(games[1]).some((frame) =>
        frame.state.physicalTick === state.physicalTick))!
    const beforeSighted = snapshots(games[1]).find(({ state }) =>
      state.physicalTick === before.state.physicalTick)!
    expect(before.state.physicalTick).toBeGreaterThanOrEqual(deadline - 3)
    expect(before.state.bombs.some(({ id }) => id === survivor.id)).toBe(false)
    expect(beforeSighted.state.bombs.some(({ id }) => id === survivor.id)).toBe(true)
    await expect.poll(() => snapshots(games[0]).find(({ state }) => state.physicalTick >= deadline
      && snapshots(games[1]).some((frame) => frame.state.physicalTick === state.physicalTick))
      ?.state.bombs.find(({ id }) => id === survivor.id)).toBeTruthy()
    const after = snapshots(games[0]).find(({ state }) => state.physicalTick >= deadline
      && snapshots(games[1]).some((frame) => frame.state.physicalTick === state.physicalTick))!
    const afterSighted = snapshots(games[1]).find(({ state }) =>
      state.physicalTick === after.state.physicalTick)!
    expect(after.state.physicalTick).toBeLessThanOrEqual(deadline + 2)
    const revealed = after.state.bombs.find(({ id }) => id === survivor.id)!
    const sighted = afterSighted.state.bombs.find(({ id }) => id === survivor.id)!
    expect(revealed.cell).toEqual({ column: 12, row: 11 })
    expect(revealed.trigger.type).toBe('timed')
    expect(sighted.trigger.type).toBe('timed')
    if (revealed.trigger.type !== 'timed' || sighted.trigger.type !== 'timed') {
      throw new Error('surviving fixture bomb must remain Timed')
    }
    expect(revealed.trigger.remainingTicks).toBe(sighted.trigger.remainingTicks)
    expect(bombEvents(games[0], privacyStart)).toHaveLength(0)
    expect(await arenaTilePixels(one, 12, 11)).not.toEqual(await arenaTilePixels(one, 3, 1))
  } finally { await Promise.all(contexts.map((context) => context.close())) }
})
