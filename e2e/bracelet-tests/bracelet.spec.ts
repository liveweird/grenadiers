import { createResponsiveBrowserContexts } from '../tests/support/browser-contexts'
import { expect, test, type Page } from '@playwright/test'
import { arenaTilePixels } from '../tests/support/compositor'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveSeatTo } from '../tests/support/movement'
import { applyItemSettings } from '../tests/support/settings'
import type { GameServerMessage, Seat } from '../../web/src/game/v19/schema'

test.beforeAll(() => expect(process.env.E2E_BRACELET_FIXTURE).toBe('arena-v3-bracelet'))
type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
const snapshots = (game: ObservedGame): Snapshot[] => game.frames.filter(
  (frame): frame is Snapshot => frame.type === 'snapshot')
const actor = (game: ObservedGame, seat: Seat) => visibleActor(latestSnapshot(game), seat)!
const privateShoveFacts = /lastSuccessfulShoveTick|forcedMovement/i

async function waitClear(game: ObservedGame): Promise<void> {
  await expect.poll(() => {
    const state = latestSnapshot(game)!.state
    return state.bombs.length + state.flames.length
  }).toBe(0)
}

async function plantAndEscape(
  page: Page, game: ObservedGame, seat: Seat, escape: { column: number, row: number },
): Promise<void> {
  await page.keyboard.press('Space')
  await moveSeatTo(page, game, seat, escape)
  await waitClear(game)
}

async function collectAndStagePusher(page: Page, game: ObservedGame): Promise<void> {
  await moveSeatTo(page, game, 'one', { column: 2, row: 1 })
  await page.keyboard.press('Space')
  await moveSeatTo(page, game, 'one', { column: 1, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 1, row: 2 })
  await waitClear(game)
  await moveSeatTo(page, game, 'one', { column: 1, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 3, row: 1 })
  await expect.poll(() => actor(game, 'one').inventory.equipment).toBe('bracelet')

  await page.keyboard.press('Space')
  await moveSeatTo(page, game, 'one', { column: 1, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 1, row: 2 })
  await waitClear(game)
  await moveSeatTo(page, game, 'one', { column: 1, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 4, row: 1 })
  await page.keyboard.press('Space')
  await moveSeatTo(page, game, 'one', { column: 3, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 3, row: 2 })
  await waitClear(game)
  await moveSeatTo(page, game, 'one', { column: 3, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 5, row: 1 })
  for (const [plant, escape, stage] of [
    [{ column: 5, row: 1 }, { column: 2, row: 1 }, { column: 7, row: 1 }],
    [{ column: 7, row: 1 }, { column: 4, row: 1 }, { column: 8, row: 1 }],
    [{ column: 8, row: 1 }, { column: 5, row: 1 }, { column: 9, row: 1 }],
    [{ column: 9, row: 1 }, { column: 6, row: 1 }, { column: 10, row: 1 }],
  ] as const) {
    await test.step(`clear pusher corridor through ${stage.column},${stage.row}`, async () => {
      await moveSeatTo(page, game, 'one', plant)
      await plantAndEscape(page, game, 'one', escape)
      await moveSeatTo(page, game, 'one', stage)
    })
  }
}

async function clearAndStageVictim(page: Page, game: ObservedGame): Promise<void> {
  await moveSeatTo(page, game, 'two', { column: 13, row: 9 })
  await page.keyboard.press('Space')
  await moveSeatTo(page, game, 'two', { column: 13, row: 11 })
  await moveSeatTo(page, game, 'two', { column: 12, row: 11 })
  await waitClear(game)
  await moveSeatTo(page, game, 'two', { column: 13, row: 11 })
  await moveSeatTo(page, game, 'two', { column: 13, row: 8 })
  for (const [plant, escape, stage] of [
    [{ column: 13, row: 8 }, { column: 13, row: 11 }, { column: 13, row: 7 }],
    [{ column: 13, row: 7 }, { column: 13, row: 10 }, { column: 13, row: 6 }],
    [{ column: 13, row: 6 }, { column: 13, row: 9 }, { column: 13, row: 5 }],
    [{ column: 13, row: 5 }, { column: 13, row: 8 }, { column: 13, row: 4 }],
  ] as const) {
    await test.step(`clear victim corridor through ${stage.column},${stage.row}`, async () => {
      await moveSeatTo(page, game, 'two', plant)
      await plantAndEscape(page, game, 'two', escape)
      await moveSeatTo(page, game, 'two', stage)
    })
  }
  await moveSeatTo(page, game, 'two', { column: 13, row: 1 })
  await page.keyboard.press('Space')
  await moveSeatTo(page, game, 'two', { column: 13, row: 4 })
  await waitClear(game)
  await moveSeatTo(page, game, 'two', { column: 13, row: 1 })
  await moveSeatTo(page, game, 'two', { column: 11, row: 1 })
}

function expectSolidPlayers(frame: Snapshot): void {
  const players = frame.state.actors.filter((value) => value.type === 'visible' && value.alive)
  for (const [index, left] of players.entries()) {
    if (left.type !== 'visible') continue
    for (const right of players.slice(index + 1)) {
      if (right.type !== 'visible') continue
      expect({ column: Math.floor(left.position.x / 1024), row: Math.floor(left.position.y / 1024) })
        .not.toEqual({ column: Math.floor(right.position.x / 1024), row: Math.floor(right.position.y / 1024) })
      expect(Math.abs(left.position.x - right.position.x) >= 640
        || Math.abs(left.position.y - right.position.y) >= 640).toBe(true)
    }
  }
}

async function expectStationary(game: ObservedGame, seat: Seat, after: number): Promise<void> {
  await expect.poll(() => {
    const frames = snapshots(game).filter(({ room }) => room.revision > after).slice(-2)
    if (frames.length !== 2 || frames[0]!.state.physicalTick >= frames[1]!.state.physicalTick) return false
    const positions = frames.map((frame) => visibleActor(frame, seat)!.position)
    return positions.every((position) => position.x === positions[0]!.x
      && position.y === positions[0]!.y)
  }, { intervals: [15] }).toBe(true)
}

test('Bracelet shoves a stationary rival one tile without an action or private cue',
  async ({ browser }, info) => {
    const contexts = await createResponsiveBrowserContexts(browser)
    try {
      const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
      const games = [observeGame(one), observeGame(two)]
      await Promise.all([one.goto('/'), two.goto('/')])
      await one.getByRole('button', { name: 'Create room', exact: true }).click()
      const code = one.getByLabel('Room invitation code', { exact: true })
      await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
      await applyItemSettings(one, games[0], [{ id: 'bracelet', label: 'Bracelet' }],
        () => one.getByLabel('Bomb fuse').selectOption('120'))
      await expect(one.getByText(/shoves them up to 1 tile/)).toBeVisible()
      await expect(one.getByText(/0.5 second cooldown/)).toBeVisible()
      await two.getByText('Polski', { exact: true }).click()
      await two.getByRole('textbox', { name: 'Kod pokoju', exact: true }).fill(await code.innerText())
      await two.getByRole('button', { name: 'Dołącz do pokoju', exact: true }).click()
      await expect(two.getByText(/0,5 sekundy odnowienia/)).toBeVisible()
      await Promise.all([
        one.getByRole('button', { name: 'Ready', exact: true }).click(),
        two.getByRole('button', { name: 'Gotowy', exact: true }).click(),
      ])
      await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type)
        .toBe('running')))
      await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
      await two.getByRole('region', { name: 'Dziedziniec Grenadiers', exact: true }).click()

      await collectAndStagePusher(one, games[0])
      await clearAndStageVictim(two, games[1])
      await Promise.all(games.map((game) => expect.poll(() => ({
        bracelet: actor(game, 'one').inventory.equipment,
        one: { column: Math.floor(actor(game, 'one').position.x / 1_024),
          row: Math.floor(actor(game, 'one').position.y / 1_024) },
        two: { column: Math.floor(actor(game, 'two').position.x / 1_024),
          row: Math.floor(actor(game, 'two').position.y / 1_024) },
        hazards: latestSnapshot(game)!.state.bombs.length + latestSnapshot(game)!.state.flames.length,
      })).toEqual({
        bracelet: 'bracelet', one: { column: 10, row: 1 },
        two: { column: 11, row: 1 }, hazards: 0,
      })))
      await expect(one.getByRole('region', { name: 'Your items' })).toContainText('GearBracelet')
      await expect(two.getByLabel('Ulepszenia: Gracz pierwszy')).toContainText('WyposażenieBransoleta')
      expect(games.flatMap(({ commands }) => commands)
        .some((command) => command.type === 'manualAction' && command.action === 'equipment')).toBe(false)

      games.forEach((game) => snapshots(game).forEach(expectSolidPlayers))
      const settleFloors = games.map((game) => latestSnapshot(game)!.room.revision)
      await Promise.all(games.flatMap((game, index) => [
        expectStationary(game, 'one', settleFloors[index]!),
        expectStationary(game, 'two', settleFloors[index]!),
      ]))
      const floors = games.map((game) => latestSnapshot(game)!.room.revision)
      const commandFloors = games.map((game) => game.commands.length)
      const starts = games.map((game) => ({
        one: actor(game, 'one').position,
        two: actor(game, 'two').position,
      }))
      const targetBaselines = await Promise.all([one, two]
        .map((page) => arenaTilePixels(page, 12, 1)))
      await one.keyboard.down('ArrowRight')
      try {
        await expect.poll(() => actor(games[0], 'two').position.x).toBe(starts[0]!.two.x + 1_024)
      } finally {
        await one.keyboard.up('ArrowRight')
      }
      await expect.poll(() => games[0].commands.slice(commandFloors[0])
        .some((command) => command.type === 'movement' && command.direction === null)).toBe(true)
      await Promise.all(games.map((game, index) => expect.poll(() => {
        const snapshot = latestSnapshot(game)!
        if (snapshot.room.revision <= floors[index]!) return undefined
        return visibleActor(snapshot, 'two')?.position
      }).toEqual({ x: starts[index]!.two.x + 1_024, y: starts[index]!.two.y })))

      const movedRevisions: number[] = []
      for (const [index, game] of games.entries()) {
        const moved = snapshots(game).find((snapshot) => snapshot.room.revision > floors[index]!
          && visibleActor(snapshot, 'two')?.position.x === starts[index]!.two.x + 1_024)!
        movedRevisions[index] = moved.room.revision
        expect(visibleActor(moved, 'one')).toMatchObject({
          alive: true, inventory: { equipment: 'bracelet' },
        })
        expect(visibleActor(moved, 'one')!.position.x).toBeGreaterThan(starts[index]!.one.x)
        expect(visibleActor(moved, 'two')).toMatchObject({
          alive: true, position: {
            x: starts[index]!.two.x + 1_024, y: starts[index]!.two.y,
          },
        })
        const raw = snapshots(game).filter(({ room }) =>
          room.revision > floors[index]! && room.revision <= moved.room.revision)
        expect(raw.length).toBeGreaterThan(0)
        raw.forEach(expectSolidPlayers)
        expect(raw.flatMap(({ events }) => events)).toEqual([])
        expect(JSON.stringify(raw)).not.toMatch(privateShoveFacts)
        expect(game.commands.slice(commandFloors[index]!).some((command) =>
          command.type === 'manualAction' && command.action === 'equipment')).toBe(false)
      }
      expect(games[1].commands.slice(commandFloors[1]!)).toEqual([])
      await Promise.all(games.flatMap((game, index) => [
        expectStationary(game, 'one', movedRevisions[index]!),
        expectStationary(game, 'two', movedRevisions[index]!),
      ]))

      const shovePixels = await Promise.all([one, two].map((page) => arenaTilePixels(page, 12, 1)))
      expect(shovePixels[0]).not.toEqual(targetBaselines[0])
      expect(shovePixels[1]).not.toEqual(targetBaselines[1])
      expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
      await Promise.all([
        one.screenshot({ path: info.outputPath('bracelet-shoved-desktop-en.png'), fullPage: true }),
        two.screenshot({ path: info.outputPath('bracelet-shoved-polish-320-reduced.png'), fullPage: true }),
      ])
      for (const game of games) {
        const index = games.indexOf(game)
        expect(actor(game, 'two').position).toEqual({
          x: starts[index]!.two.x + 1_024, y: starts[index]!.two.y,
        })
      }
    } finally {
      await Promise.all(contexts.map(async (context) => {
        for (const page of context.pages()) await page.keyboard.up('ArrowRight').catch(() => undefined)
        await context.close()
      }))
    }
  })
