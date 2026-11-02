import { createResponsiveBrowserContexts } from '../tests/support/browser-contexts'
import { expect, test, type Page } from '@playwright/test'
import { arenaTilePixels } from '../tests/support/compositor'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveSeatTo } from '../tests/support/movement'
import { applyItemSettings } from '../tests/support/settings'
import type { GameServerMessage } from '../../web/src/game/v20/schema'
import { retainedKickTimeline } from './kick-timeline'

test.beforeAll(() => expect(process.env.E2E_KICK_FIXTURE).toBe('arena-v3-kick'))
type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
const snapshots = (game: ObservedGame): Snapshot[] => game.frames.filter(
  (frame): frame is Snapshot => frame.type === 'snapshot')
const actor = (game: ObservedGame) => visibleActor(latestSnapshot(game), 'one')!
const eventsAfter = (game: ObservedGame, revision: number) => snapshots(game)
  .filter(({ room }) => room.revision > revision).flatMap(({ events }) => events)
const placedAfter = (game: ObservedGame, revision: number) => eventsAfter(game, revision)
  .flatMap(({ kind }) => kind.type === 'bombPlaced' ? [kind.bomb] : [])
const detonatedAfter = (game: ObservedGame, revision: number) => eventsAfter(game, revision)
  .flatMap(({ tick, kind }) => kind.type === 'bombDetonated' ? [{ tick, bomb: kind.bomb }] : [])

async function waitClear(game: ObservedGame): Promise<void> {
  await expect.poll(() => {
    const state = latestSnapshot(game)!.state
    return state.bombs.length + state.flames.length
  }).toBe(0)
}
async function revealAndCollect(page: Page, game: ObservedGame): Promise<void> {
  await moveSeatTo(page, game, 'one', { column: 2, row: 1 })
  await page.keyboard.press('Space')
  await moveSeatTo(page, game, 'one', { column: 1, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 1, row: 2 })
  await waitClear(game)
  await moveSeatTo(page, game, 'one', { column: 1, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 3, row: 1 })
  await expect.poll(() => actor(game).inventory.equipment).toBe('kick')
}

async function prepareEscape(page: Page, game: ObservedGame): Promise<void> {
  await page.keyboard.press('Space')
  await moveSeatTo(page, game, 'one', { column: 1, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 1, row: 2 })
  await waitClear(game)
  for (const cell of [{ column: 3, row: 2 }, { column: 4, row: 1 }]) {
    expect(latestSnapshot(game)!.state.arena.blocks).not.toContainEqual(cell)
  }
  await moveSeatTo(page, game, 'one', { column: 1, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 3, row: 1 })
}

test('Kick moves one retained bomb through authoritative cells without extra cues and preserves its fuse',
  async ({ browser }, info) => {
    const contexts = await createResponsiveBrowserContexts(browser)
    try {
      const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
      const games = [observeGame(one), observeGame(two)]
      await Promise.all([one.goto('/'), two.goto('/')])
      await one.getByRole('button', { name: 'Create room', exact: true }).click()
      const code = one.getByLabel('Room invitation code', { exact: true })
      await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
      await applyItemSettings(one, games[0], [{ id: 'kick', label: 'Kick' }],
        () => one.getByLabel('Bomb fuse').selectOption('300'))
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
      await revealAndCollect(one, games[0])
      await prepareEscape(one, games[0])
      await Promise.all(games.map((game) => expect.poll(() => actor(game).inventory.equipment)
        .toBe('kick')))
      // Rules7 starts at range two; the sole Kick route must not change that baseline.
      await Promise.all(games.map((game) => expect.poll(() => actor(game).bombRange).toBe(2)))
      await expect(one.getByRole('region', { name: 'Your items' })).toContainText('GearKick')
      await expect(two.getByLabel('Ulepszenia: Gracz pierwszy')).toContainText('WyposażenieKopnięcie')

      const beforePlant = latestSnapshot(games[0])!.room.revision
      await one.keyboard.press('Space')
      await expect.poll(() => placedAfter(games[0], beforePlant)).toHaveLength(1)
      const demo = placedAfter(games[0], beforePlant)[0]!
      expect(demo).toMatchObject({
        cell: { column: 3, row: 1 }, owner: actor(games[0]).id, range: 2,
        blast: 'standard', trigger: { type: 'timed', remainingTicks: 300, paused: false },
      })
      await moveSeatTo(one, games[0], 'one', { column: 4, row: 1 })
      const kickBaselines = await Promise.all(games.map(async (game) => {
        await expect.poll(() => latestSnapshot(game)!.state.bombs
          .find(({ id }) => id === demo.id)).toMatchObject({
          id: demo.id, cell: { column: 3, row: 1 }, owner: demo.owner,
          range: demo.range, placedAt: demo.placedAt, blast: demo.blast,
          trigger: expect.objectContaining({ type: 'timed', paused: false }),
        })
        const baseline = latestSnapshot(game)!
        expect(baseline.state.bombs.find(({ id }) => id === demo.id)?.cell)
          .toEqual({ column: 3, row: 1 })
        return baseline
      }))
      await one.keyboard.down('ArrowLeft')
      try {
        await Promise.all(games.map((game) => expect.poll(() =>
          latestSnapshot(game)!.state.bombs.find(({ id }) => id === demo.id)?.cell)
          .toEqual({ column: 1, row: 1 })))
      } finally { await one.keyboard.up('ArrowLeft') }
      for (const [index, game] of games.entries()) {
        const baseline = kickBaselines[index]!
        const stream = retainedKickTimeline(baseline, snapshots(game), demo)
        const cells = stream.map(({ cell }) => `${cell.column},${cell.row}`)
          .filter((cell, index, all) => index === 0 || cell !== all[index - 1])
        expect(cells).toEqual(expect.arrayContaining(['3,1', '2,1', '1,1']))
        expect(cells.indexOf('3,1')).toBeLessThan(cells.indexOf('2,1'))
        expect(cells.indexOf('2,1')).toBeLessThan(cells.indexOf('1,1'))
        expect(eventsAfter(game, baseline.room.revision)
          .filter(({ kind }) => kind.type === 'bombPlaced')).toEqual([])
        for (const bomb of stream) expect(bomb).toMatchObject({
          id: demo.id, owner: demo.owner, range: demo.range, placedAt: demo.placedAt,
          blast: demo.blast, trigger: expect.objectContaining({ type: 'timed', paused: false }),
        })
      }

      await moveSeatTo(one, games[0], 'one', { column: 3, row: 1 })
      await moveSeatTo(one, games[0], 'one', { column: 3, row: 2 })
      await Promise.all(games.map((game) => expect.poll(() =>
        visibleActor(latestSnapshot(game), 'one')?.position).toEqual(actor(games[0]).position)))
      const stoppedPixels = await Promise.all([one, two].map((page) => arenaTilePixels(page, 1, 1)))
      const vacatedPixels = await Promise.all([one, two].map((page) => arenaTilePixels(page, 3, 1)))
      expect(stoppedPixels[0]).not.toEqual(vacatedPixels[0])
      expect(stoppedPixels[1]).not.toEqual(vacatedPixels[1])
      await one.screenshot({ path: info.outputPath('kick-stopped-desktop-en.png'), fullPage: true })
      expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
      await two.screenshot({ path: info.outputPath('kick-stopped-polish-320-reduced.png'), fullPage: true })

      await Promise.all(games.map((game) => expect.poll(() => detonatedAfter(game, beforePlant)
        .find(({ bomb }) => bomb.id === demo.id)).toBeTruthy()))
      for (const game of games) {
        const detonation = detonatedAfter(game, beforePlant).find(({ bomb }) => bomb.id === demo.id)!
        expect(detonation.tick).toBe(demo.placedAt + 300)
        expect(detonation.bomb).toMatchObject({
          id: demo.id, cell: { column: 1, row: 1 }, owner: demo.owner,
          range: demo.range, placedAt: demo.placedAt, blast: demo.blast,
        })
        for (const seat of ['one', 'two'] as const) {
          expect(visibleActor(latestSnapshot(game), seat)?.alive).toBe(true)
        }
      }
    } finally {
      await Promise.all(contexts.map(async (context) => {
        for (const page of context.pages()) await page.keyboard.up('ArrowLeft').catch(() => undefined)
        await context.close()
      }))
    }
  })
