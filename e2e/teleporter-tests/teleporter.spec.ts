import { expect, test, type Page } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveSeatTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'
import type { GameServerMessage, Seat } from '../../web/src/game/v11/schema'

test.beforeAll(() => expect(process.env.E2E_TELEPORTER_FIXTURE).toBe('arena-v2-teleporter'))
type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
const snapshots = (game: ObservedGame): Snapshot[] => game.frames.filter(
  (frame): frame is Snapshot => frame.type === 'snapshot')
const collections = (game: ObservedGame) => snapshots(game).flatMap(({ events }) => events)
  .filter(({ kind }) => kind.type === 'pickupCollected' && kind.kind === 'teleporter')
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
) {
  await page.keyboard.down(key)
  try {
    await expect.poll(() => collections(game).length).toBe(count)
  } finally {
    await page.keyboard.up(key)
  }
}

test('Teleporter swaps safe players, survives reconnect, and is consumed when unsafe', async ({ browser }) => {
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
    await two.getByRole('region', { name: 'Dziedziniec Grenadierów' }).click()

    await reveal(one, games[0], 'one', { column: 2, row: 1 },
      [{ column: 1, row: 1 }, { column: 1, row: 2 }])
    await expect.poll(() => latestSnapshot(games[0])!.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'teleporter',
    })
    await moveSeatTo(one, games[0], 'one', { column: 1, row: 1 })
    const rivalStart = position(games[0], 'two')
    await collectUntil(one, games[0], 'ArrowRight', 1)
    await Promise.all(games.map((game) => expect.poll(() => marker(game, 'one')).not.toBeNull()))
    const successfulMarker = marker(games[0], 'one')
    expect(marker(games[0], 'two')).toBe(successfulMarker)
    expect(position(games[0], 'one')).toEqual(rivalStart)
    expect(cell(position(games[0], 'two'))).toEqual({ column: 3, row: 1 })

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
    await two.keyboard.press('Space')
    await two.keyboard.press('ArrowLeft', { delay: 34 })
    const unsafeTarget = position(games[0], 'two')
    await moveSeatTo(one, games[0], 'one', { column: 13, row: 11 })
    await collectUntil(one, games[0], 'ArrowLeft', 2)
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
