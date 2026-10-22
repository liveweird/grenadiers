import { expect, test, type Page } from '@playwright/test'
import { arenaTilePixels } from '../tests/support/compositor'
import { expectPaintedCanvas, latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveSeatTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'
import type { GameServerMessage, Seat } from '../../web/src/game/v10/schema'

test.beforeAll(() => expect(process.env.E2E_WALL_PASS_FIXTURE).toBe('arena-v2-wall-pass'))

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
const snapshots = (game: ObservedGame): Snapshot[] => game.frames.filter(
  (frame): frame is Snapshot => frame.type === 'snapshot')
const actor = (game: ObservedGame, seat: Seat) => visibleActor(latestSnapshot(game), seat)!
const collections = (game: ObservedGame) => snapshots(game).flatMap(({ events }) => events)
  .filter(({ kind }) => kind.type === 'pickupCollected' && kind.kind === 'wallPass')
const bombPlacements = (game: ObservedGame) => snapshots(game).flatMap(({ events }) => events)
  .flatMap(({ kind }) => kind.type === 'bombPlaced' ? [kind.bomb] : [])
const bombPlacementsAfter = (game: ObservedGame, revision: number) => snapshots(game)
  .filter(({ room }) => room.revision > revision).flatMap(({ events }) => events)
  .flatMap(({ kind }) => kind.type === 'bombPlaced' ? [kind.bomb] : [])

async function revealWallPass(page: Page, game: ObservedGame): Promise<number> {
  await moveSeatTo(page, game, 'one', { column: 2, row: 1 })
  await page.keyboard.press('Space')
  await moveSeatTo(page, game, 'one', { column: 1, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 1, row: 2 })
  await expect.poll(() => latestSnapshot(game)!.state.bombs.length
    + latestSnapshot(game)!.state.flames.length).toBe(0)
  await expect.poll(() => latestSnapshot(game)!.state.pickups).toContainEqual({
    cell: { column: 3, row: 1 }, kind: 'wallPass',
  })
  expect(bombPlacements(game)).toHaveLength(1)
  return bombPlacements(game)[0]!.id
}

async function expectStationary(game: ObservedGame, seat: Seat): Promise<void> {
  await expect.poll(() => {
    const [left, right] = snapshots(game).slice(-2).map((frame) =>
      visibleActor(frame, seat)?.position)
    return left !== undefined && JSON.stringify(left) === JSON.stringify(right)
  }).toBe(true)
}

test('Wall Pass crosses one intact block, rejects planting and hard walls, and resumes inside',
  async ({ browser }, info) => {
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
      await one.getByRole('checkbox', { name: 'Wall pass', exact: true }).check()
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
      await one.getByRole('region', { name: 'Grenadiers courtyard', exact: true }).click()
      await two.getByRole('region', { name: 'Dziedziniec Grenadiers', exact: true }).click()

      const revealBombId = await revealWallPass(one, games[0])
      await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)!.state.bombs.length
        + latestSnapshot(game)!.state.flames.length).toBe(0)))
      await Promise.all([one, two].map(expectPaintedCanvas))
      await one.screenshot({
        path: info.outputPath('wall-pass-before-collection-desktop-en.png'), fullPage: true,
      })
      expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
      await two.screenshot({
        path: info.outputPath('wall-pass-before-collection-polish-320-reduced.png'), fullPage: true,
      })
      await moveSeatTo(one, games[0], 'one', { column: 1, row: 1 })
      await moveSeatTo(one, games[0], 'one', { column: 3, row: 1 })
      await Promise.all(games.map((game) => expect.poll(() =>
        actor(game, 'one').inventory.equipment).toBe('wallPass')))
      expect(collections(games[0])).toHaveLength(1)
      const blockOnly = await Promise.all([one, two].map((page) => arenaTilePixels(page, 3, 2)))

      await moveSeatTo(one, games[0], 'one', { column: 3, row: 2 })
      const insidePosition = actor(games[0], 'one').position
      expect(Math.floor(insidePosition.x / 1024)).toBe(3)
      expect(Math.floor(insidePosition.y / 1024)).toBe(2)
      for (const game of games) {
        const snapshot = latestSnapshot(game)!
        expect(snapshot.state.arena.blocks).toContainEqual({ column: 3, row: 2 })
        expect(snapshot.state.pickups).not.toContainEqual(expect.objectContaining({
          cell: { column: 3, row: 2 },
        }))
        expect(actor(game, 'one')).toMatchObject({
          position: insidePosition, inventory: { equipment: 'wallPass', weapon: null },
        })
      }
      await expect(one.getByLabel('Player one upgrades')).toContainText('EquipmentWall pass')
      await expect(two.getByLabel('Ulepszenia: Gracz pierwszy'))
        .toContainText('WyposażeniePrzejście przez ściany')
      const occupied = await Promise.all([one, two].map((page) => arenaTilePixels(page, 3, 2)))
      for (const [index, pixels] of occupied.entries()) {
        expect(pixels).not.toEqual(blockOnly[index])
        const oldColors = new Set(blockOnly[index])
        expect(new Set(pixels.filter((pixel) => oldColors.has(pixel))).size).toBeGreaterThan(3)
      }
      const beforeBomb = latestSnapshot(games[0])!
      const beforeActor = actor(games[0], 'one')
      const commandFloor = games[0].commands.length
      await one.keyboard.press('Space')
      await expect.poll(() => games[0].commands.slice(commandFloor)).toContainEqual(
        expect.objectContaining({ type: 'manualAction', action: 'bomb' }))
      await expect.poll(() => latestSnapshot(games[0])!.room.revision)
        .toBeGreaterThan(beforeBomb.room.revision)
      for (const game of games) {
        expect(latestSnapshot(game)!.state.bombs.map(({ id }) => id))
          .toEqual(beforeBomb.state.bombs.map(({ id }) => id))
        expect(bombPlacementsAfter(game, beforeBomb.room.revision)).toHaveLength(0)
        expect(actor(game, 'one')).toMatchObject({
          bombCapacity: beforeActor.bombCapacity, liveBombCount: beforeActor.liveBombCount,
          inventory: beforeActor.inventory,
        })
      }

      await one.keyboard.down('ArrowLeft')
      try {
        await expect.poll(() => actor(games[0], 'one').position.x).toBe(3392)
      } finally {
        await one.keyboard.up('ArrowLeft')
      }
      await expectStationary(games[0], 'one')
      expect(actor(games[0], 'one').position).toEqual({ x: 3392, y: insidePosition.y })
      expect(latestSnapshot(games[0])!.state.arena.terrain[2 * 15 + 2]).toBe('wall')
      await moveSeatTo(one, games[0], 'one', { column: 3, row: 2 })
      const resumePosition = actor(games[0], 'one').position

      const baseline = (await admissionBaseline(one))!
      const sockets = games[0].sockets
      await disconnectTransport(one)
      await expect.poll(() => games[0].sockets).toBeGreaterThan(sockets)
      await expect.poll(async () => (await admissionBaseline(one))?.index).toBeGreaterThan(baseline.index)
      const resumed = (await admissionBaseline(one))!
      expect(resumed.hasState).toBe(true)
      expect(resumed.positions.find(({ seat }) => seat === 'one')).toEqual({
        seat: 'one', ...resumePosition,
      })
      expect(resumed.equipment.find(({ seat }) => seat === 'one'))
        .toEqual({ seat: 'one', equipment: 'wallPass' })

      await moveSeatTo(one, games[0], 'one', { column: 3, row: 1 })
      const exited = actor(games[0], 'one')
      expect(Math.floor(exited.position.x / 1024)).toBe(3)
      expect(Math.floor(exited.position.y / 1024)).toBe(1)
      expect(exited.inventory.equipment).toBe('wallPass')
      expect(latestSnapshot(games[0])!.state.arena.blocks)
        .toContainEqual({ column: 3, row: 2 })

      const afterExit = latestSnapshot(games[0])!
      await one.keyboard.press('Space')
      await Promise.all(games.map((game) => expect.poll(() =>
        bombPlacementsAfter(game, afterExit.room.revision).map(({ id, blast, trigger }) => ({
          id, blast, trigger,
        }))).toEqual([{
        id: revealBombId + 1, blast: 'standard',
        trigger: { type: 'timed', remainingTicks: 180, paused: false },
      }])))
      for (const game of games) {
        expect(latestSnapshot(game)!.state.bombs).toContainEqual(expect.objectContaining({
          id: revealBombId + 1, owner: exited.id, cell: { column: 3, row: 1 },
          blast: 'standard', trigger: expect.objectContaining({ type: 'timed', paused: false }),
        }))
      }
    } finally {
      await Promise.all(contexts.map(async (context) => {
        for (const page of context.pages()) {
          await page.keyboard.up('ArrowLeft').catch(() => undefined)
          await page.keyboard.up('ArrowRight').catch(() => undefined)
          await page.keyboard.up('ArrowUp').catch(() => undefined)
          await page.keyboard.up('ArrowDown').catch(() => undefined)
        }
        await context.close()
      }))
    }
  })
