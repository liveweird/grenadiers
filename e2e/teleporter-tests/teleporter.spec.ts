import { expect, test, type Page } from '@playwright/test'
import { expectPaintedCanvas, latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveSeatTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'
import type { GameServerMessage, Seat } from '../../web/src/game/v10/schema'

test.beforeAll(() => expect(process.env.E2E_TELEPORTER_FIXTURE).toBe('arena-v2-teleporter'))
type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
const snapshots = (game: ObservedGame): Snapshot[] => game.frames.filter(
  (frame): frame is Snapshot => frame.type === 'snapshot')
const collectionSnapshots = (game: ObservedGame) => snapshots(game).filter(({ events }) =>
  events.some(({ kind }) => kind.type === 'pickupCollected' && kind.kind === 'teleporter'))
const teleporterEvents = (snapshot: Snapshot) => snapshot.events.filter(({ kind }) =>
  kind.type === 'pickupCollected' && kind.kind === 'teleporter')
const actorAt = (snapshot: Snapshot, seat: Seat) => visibleActor(snapshot, seat)!
const positionAt = (snapshot: Snapshot, seat: Seat) => actorAt(snapshot, seat).position
const position = (game: ObservedGame, seat: Seat) => visibleActor(latestSnapshot(game), seat)!.position
const marker = (game: ObservedGame, seat: Seat) =>
  visibleActor(latestSnapshot(game), seat)!.lastTeleportTick
const cell = ({ x, y }: { x: number; y: number }) =>
  ({ column: Math.floor(x / 1024), row: Math.floor(y / 1024) })

async function reveal(
  page: Page, game: ObservedGame, seat: Seat, bomb: { column: number; row: number },
  escape: readonly { column: number; row: number }[],
) {
  await moveSeatTo(page, game, seat, bomb)
  await page.keyboard.press('Space')
  for (const waypoint of escape) await moveSeatTo(page, game, seat, waypoint)
  await expect.poll(() => latestSnapshot(game)!.state.bombs.length
    + latestSnapshot(game)!.state.flames.length).toBe(0)
}

async function collectUntil(
  page: Page, game: ObservedGame, key: 'ArrowLeft' | 'ArrowRight', count: number,
  expectedCell: { column: number; row: number },
): Promise<Snapshot> {
  let collected!: Snapshot
  await moveAndSettle(page, game, key, async () => {
    await expect.poll(() => collectionSnapshots(game).length).toBe(count)
    collected = collectionSnapshots(game)[count - 1]
    return collected.room.revision
  })
  expect(teleporterEvents(collected)).toHaveLength(1)
  expect(teleporterEvents(collected)[0].kind).toMatchObject({
    type: 'pickupCollected', seat: 'one', cell: expectedCell, kind: 'teleporter',
  })
  return collected
}

async function moveAndSettle(
  page: Page, game: ObservedGame, key: 'ArrowLeft' | 'ArrowRight',
  observeMovement: () => Promise<number>,
): Promise<Snapshot> {
  const commandCount = game.commands.length
  const direction = key === 'ArrowLeft' ? 'west' : 'east'
  let movementRevision: number
  await page.keyboard.down(key)
  try {
    movementRevision = await observeMovement()
  } finally {
    await page.keyboard.up(key)
  }
  await expect.poll(() => {
    const commands = game.commands.slice(commandCount)
    const requested = commands.findIndex((command) => command.type === 'movement'
      && command.direction === direction)
    return requested >= 0 && commands.slice(requested + 1)
      .some((command) => command.type === 'movement' && command.direction === null)
  }, { intervals: [15] }).toBe(true)
  const releaseRevision = latestSnapshot(game)!.room.revision
  const settleAfter = Math.max(movementRevision, releaseRevision)
  await expect.poll(() => {
    const later = snapshots(game).filter(({ room }) => room.revision > settleAfter).slice(-2)
    if (later.length !== 2) return false
    return (['one', 'two'] as const).every((seat) => {
      const before = positionAt(later[0], seat)
      const after = positionAt(later[1], seat)
      return before.x === after.x && before.y === after.y
    })
  }, { intervals: [15] }).toBe(true)
  return latestSnapshot(game)!
}

test('Teleporter swaps safe players, survives reconnect, and is consumed when unsafe', async ({ browser }, info) => {
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
    await one.getByRole('checkbox', { name: 'Teleporter', exact: true }).check()
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
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await two.getByRole('region', { name: 'Dziedziniec Grenadiers', exact: true }).click()

    await reveal(one, games[0], 'one', { column: 2, row: 1 },
      [{ column: 1, row: 1 }, { column: 1, row: 2 }])
    await expect.poll(() => latestSnapshot(games[0])!.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'teleporter',
    })
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)!.state.bombs.length
      + latestSnapshot(game)!.state.flames.length).toBe(0)))
    await Promise.all([one, two].map(expectPaintedCanvas))
    await one.screenshot({ path: info.outputPath('teleporter-before-collection-desktop-en.png'), fullPage: true })
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({ path: info.outputPath('teleporter-before-collection-polish-320-reduced.png'), fullPage: true })
    await moveSeatTo(one, games[0], 'one', { column: 1, row: 1 })
    const rivalStart = position(games[0], 'two')
    const successfulCollection = await collectUntil(
      one, games[0], 'ArrowRight', 1, { column: 3, row: 1 })
    await expect.poll(() => collectionSnapshots(games[1]).length).toBe(1)
    const otherCollection = collectionSnapshots(games[1])[0]
    expect({
      revision: otherCollection.room.revision,
      tick: otherCollection.room.tick,
      events: teleporterEvents(otherCollection),
    }).toEqual({
      revision: successfulCollection.room.revision,
      tick: successfulCollection.room.tick,
      events: teleporterEvents(successfulCollection),
    })
    const successfulEvent = teleporterEvents(successfulCollection)[0]
    const successfulMarker = successfulEvent.tick
    for (const snapshot of [successfulCollection, otherCollection]) {
      expect(snapshot.room.tick).toBeGreaterThanOrEqual(successfulEvent.tick)
      expect(snapshot.room.tick).toBeLessThanOrEqual(successfulEvent.tick + 2)
      expect(actorAt(snapshot, 'one').lastTeleportTick).toBe(successfulMarker)
      expect(actorAt(snapshot, 'two').lastTeleportTick).toBe(successfulMarker)
      expect(cell(positionAt(snapshot, 'one'))).toEqual(cell(rivalStart))
      expect(cell(positionAt(snapshot, 'two'))).toEqual({ column: 3, row: 1 })
    }

    const baseline = (await admissionBaseline(one))!
    const expectedActors = (['one', 'two'] as const).map((seat) => ({
      seat, ...position(games[0], seat), lastTeleportTick: successfulMarker,
    }))
    const sockets = games[0].sockets
    await disconnectTransport(one)
    await expect.poll(() => games[0].sockets).toBeGreaterThan(sockets)
    await expect.poll(async () => (await admissionBaseline(one))?.index).toBeGreaterThan(baseline.index)
    const resumed = (await admissionBaseline(one))!
    expect(resumed.hasState).toBe(true)
    expect(resumed.visibleActors).toEqual(expectedActors)

    await moveSeatTo(one, games[0], 'one', { column: 12, row: 11 })
    await one.keyboard.press('Space')
    await moveSeatTo(one, games[0], 'one', { column: 13, row: 11 })
    await moveSeatTo(one, games[0], 'one', { column: 13, row: 10 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])!.state.pickups).toContainEqual({
      cell: { column: 11, row: 11 }, kind: 'teleporter',
    })
    const unsafeCell = cell(position(games[1], 'two'))
    await two.keyboard.press('Space')
    await expect.poll(() => latestSnapshot(games[1])!.state.bombs.some(
      ({ cell: bombCell }) => bombCell.column === unsafeCell.column
        && bombCell.row === unsafeCell.row)).toBe(true)
    const unsafeRivalStart = position(games[1], 'two')
    const settledRival = await moveAndSettle(two, games[1], 'ArrowLeft', async () => {
      await expect.poll(() => position(games[1], 'two').x, {
        intervals: [15], timeout: 1_000,
      }).not.toBe(unsafeRivalStart.x)
      return latestSnapshot(games[1])!.room.revision
    })
    const unsafeTarget = positionAt(settledRival, 'two')
    expect(cell(unsafeTarget)).toEqual(unsafeCell)
    await expect.poll(() => latestSnapshot(games[1])!.state.bombs.some(
      ({ cell: bombCell }) => bombCell.column === unsafeCell.column
        && bombCell.row === unsafeCell.row)).toBe(true)
    await expect.poll(() => {
      const current = latestSnapshot(games[0])!
      return current.room.revision >= settledRival.room.revision
        ? positionAt(current, 'two') : undefined
    }).toEqual(unsafeTarget)
    await moveSeatTo(one, games[0], 'one', { column: 13, row: 11 })
    await collectUntil(one, games[0], 'ArrowLeft', 2, { column: 11, row: 11 })
    await expect.poll(() => latestSnapshot(games[0])!.state.pickups.some(
      ({ kind }) => kind === 'teleporter')).toBe(false)
    expect(marker(games[0], 'one')).toBe(successfulMarker)
    expect(marker(games[0], 'two')).toBe(successfulMarker)
    expect(position(games[0], 'two')).toEqual(unsafeTarget)
    expect(position(games[0], 'one')).not.toEqual(unsafeTarget)
    expect(cell(position(games[0], 'one'))).toEqual({ column: 11, row: 11 })
  } finally {
    await Promise.all(contexts.map(async (context) => {
      for (const page of context.pages()) {
        await page.keyboard.up('ArrowLeft').catch(() => undefined)
        await page.keyboard.up('ArrowRight').catch(() => undefined)
      }
      await context.close()
    }))
  }
})
