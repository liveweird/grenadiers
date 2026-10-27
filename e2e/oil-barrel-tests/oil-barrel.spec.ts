import { createResponsiveBrowserContexts } from '../tests/support/browser-contexts'
import { setTimeout as delay } from 'node:timers/promises'
import { expect, test, type Page } from '@playwright/test'
import { arenaTilePixels } from '../tests/support/compositor'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveSeatTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'
import { applyItemSettings } from '../tests/support/settings'
import type { GameServerMessage, Seat } from '../../web/src/game/v13/schema'

test.beforeAll(() => expect(process.env.E2E_OIL_BARREL_FIXTURE).toBe('arena-v2-oil-barrel'))
type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
const snapshots = (game: ObservedGame): Snapshot[] => game.frames.filter(
  (frame): frame is Snapshot => frame.type === 'snapshot')
const actor = (game: ObservedGame, seat: Seat) => visibleActor(latestSnapshot(game), seat)!
const oilAt = (game: ObservedGame, column: number, row: number) =>
  latestSnapshot(game)!.state.oilBarrels.find((oil) =>
    oil.cell.column === column && oil.cell.row === row)
const hasBlock = (game: ObservedGame, column: number, row: number) =>
  latestSnapshot(game)!.state.arena.blocks.some((cell) => cell.column === column && cell.row === row)
const oilArt = {
  hoop: '79,86,105,255', hoopShade: '57,60,76,255', stave: '203,109,40,255',
  staveShade: '183,94,35,255', char: '26,22,24,255', flame: '239,112,25,255',
  flameLight: '255,221,118,255', flameCore: '255,246,209,255',
} as const

function expectUnlitOilArt(pixels: string[]): void {
  const colors = new Set(pixels)
  expect([...colors]).toEqual(expect.arrayContaining([
    oilArt.hoop, oilArt.hoopShade, oilArt.stave, oilArt.staveShade,
  ]))
}

function expectBurningOilArt(pixels: string[]): void {
  const colors = new Set(pixels)
  expect([...colors]).toEqual(expect.arrayContaining([
    oilArt.char, oilArt.flame, oilArt.flameLight, oilArt.flameCore,
  ]))
}

async function plantAndObserve(page: Page, game: ObservedGame): Promise<number> {
  const known = new Set(latestSnapshot(game)!.state.bombs.map(({ id }) => id))
  await page.keyboard.press('Space')
  await expect.poll(() => latestSnapshot(game)!.state.bombs
    .find(({ id }) => !known.has(id))?.id).toEqual(expect.any(Number))
  return latestSnapshot(game)!.state.bombs.find(({ id }) => !known.has(id))!.id
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

test('Oil Barrel reveals, blocks, burns persistently, and resumes without synthetic cues',
  async ({ browser, browserName }, info) => {
    expect(browserName).toBe('chromium')
    const contexts = await createResponsiveBrowserContexts(browser)
    try {
      const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
      await Promise.all([one, two].map(installTransportProbe))
      const games = [observeGame(one), observeGame(two)]
      await Promise.all([one.goto('/'), two.goto('/')])
      await one.getByRole('button', { name: 'Create room', exact: true }).click()
      const code = one.getByLabel('Room invitation code', { exact: true })
      await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
      await applyItemSettings(one, games[0], [{ id: 'oilBarrel', label: 'Oil barrel' }],
        () => one.getByLabel('Bomb fuse').selectOption('180'))
      await expect(one.getByText(/up to 8 Oil Barrel objects/)).toBeVisible()
      await two.getByText('Polski', { exact: true }).click()
      await two.getByRole('textbox', { name: 'Kod pokoju', exact: true }).fill(await code.innerText())
      await two.getByRole('button', { name: 'Dołącz do pokoju', exact: true }).click()
      await expect(two.getByText(/do 8 obiektów Beczki oleju/)).toBeVisible()
      await Promise.all([
        one.getByRole('button', { name: 'Ready', exact: true }).click(),
        two.getByRole('button', { name: 'Gotowy', exact: true }).click(),
      ])
      await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type)
        .toBe('running')))
      await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
      await two.getByRole('region', { name: 'Dziedziniec Grenadiers', exact: true }).click()
      for (const game of games) expect(latestSnapshot(game)!.state.oilBarrels).toEqual([])
      const privacyFloors = games.map((game) => latestSnapshot(game)!.room.revision)
      const concealedPixels = await Promise.all([one, two].map((page) => arenaTilePixels(page, 3, 1)))
      const originalInventory = structuredClone(actor(games[0], 'one').inventory)

      await moveSeatTo(one, games[0], 'one', { column: 2, row: 1 })
      const revealBomb = await plantAndObserve(one, games[0])
      expect(revealBomb).toBeGreaterThanOrEqual(0)
      await moveSeatTo(one, games[0], 'one', { column: 1, row: 1 })
      await moveSeatTo(one, games[0], 'one', { column: 1, row: 2 })
      await Promise.all(games.map((game) => expect.poll(() => oilAt(game, 3, 1)?.state)
        .toBe('unlit')))
      games.forEach((game, index) => {
        const retained = snapshots(game).filter(({ room }) => room.revision >= privacyFloors[index]!)
        expect(retained[0]!.room.revision).toBe(privacyFloors[index])
        const firstOil = retained.findIndex(({ state }) => state.oilBarrels.length > 0)
        expect(firstOil).toBeGreaterThan(0)
        const privateFrames = retained.slice(0, firstOil)
        expect(privateFrames.every(({ state }) => state.oilBarrels.length === 0)).toBe(true)
        expect(privateFrames.some(({ state }) => state.flames.length > 0)).toBe(true)
        expect(retained[firstOil]!.state.flames).toEqual([])
        expect(retained[firstOil]!.state.oilBarrels).toEqual([
          { cell: { column: 3, row: 1 }, state: 'unlit' },
        ])
      })
      expect(actor(games[0], 'one').inventory).toEqual(originalInventory)
      expect(hasBlock(games[0], 4, 1)).toBe(true)
      expect(hasBlock(games[0], 3, 2)).toBe(true)
      const unlitPixels = await Promise.all([one, two].map((page) => arenaTilePixels(page, 3, 1)))
      unlitPixels.forEach((pixels, index) => {
        expect(pixels).not.toEqual(concealedPixels[index])
        expectUnlitOilArt(pixels)
      })

      await moveSeatTo(one, games[0], 'one', { column: 1, row: 1 })
      await moveSeatTo(one, games[0], 'one', { column: 2, row: 1 })
      const movementFloor = latestSnapshot(games[0])!.room.revision
      await one.keyboard.down('ArrowRight')
      try {
        await expect.poll(() => actor(games[0], 'one').position.x).toBe(2_752)
      } finally {
        await one.keyboard.up('ArrowRight')
      }
      await expectStationary(games[0], 'one', movementFloor)
      expect(actor(games[0], 'one').position.x).toBe(2_752)

      const cueFloor = latestSnapshot(games[0])!.room.revision
      const ignitionBomb = await plantAndObserve(one, games[0])
      expect(ignitionBomb).not.toBe(revealBomb)
      expect(latestSnapshot(games[0])!.state.bombs).toContainEqual(expect.objectContaining({
        id: ignitionBomb, cell: { column: 2, row: 1 },
      }))
      await moveSeatTo(one, games[0], 'one', { column: 1, row: 1 })
      await moveSeatTo(one, games[0], 'one', { column: 1, row: 2 })
      await Promise.all(games.map((game) => expect.poll(() => oilAt(game, 3, 1)?.state)
        .toBe('burning')))
      const ignitionTicks = games.map((game) => snapshots(game).find((snapshot) =>
        snapshot.room.revision > cueFloor
          && snapshot.state.oilBarrels.some(({ cell, state }) =>
            cell.column === 3 && cell.row === 1 && state === 'burning'))!.state.physicalTick)
      const burningPixels = await Promise.all([one, two].map((page) => arenaTilePixels(page, 3, 1)))
      burningPixels.forEach((pixels, index) => {
        expect(pixels).not.toEqual(unlitPixels[index])
        expectBurningOilArt(pixels)
      })
      expect(hasBlock(games[0], 4, 1)).toBe(true)
      expect(hasBlock(games[0], 3, 2)).toBe(true)
      expect(latestSnapshot(games[0])!.state.oilBarrels).toHaveLength(1)

      await Promise.all(games.map((game, index) => expect.poll(() => {
        const state = latestSnapshot(game)!.state
        return state.physicalTick >= ignitionTicks[index]! + 60 && state.flames.length === 0
          ? oilAt(game, 3, 1)?.state : undefined
      }).toBe('burning')))
      const persistentPixels = await Promise.all([one, two]
        .map((page) => arenaTilePixels(page, 3, 1)))
      persistentPixels.forEach((pixels, index) => {
        expect(pixels).not.toEqual(unlitPixels[index])
        expectBurningOilArt(pixels)
      })
      await moveSeatTo(one, games[0], 'one', { column: 1, row: 1 })
      await moveSeatTo(one, games[0], 'one', { column: 2, row: 1 })
      for (const game of games) {
        expect(actor(game, 'one').alive).toBe(true)
        expect(actor(game, 'two').alive).toBe(true)
      }
      expect(games.flatMap(snapshots).flatMap(({ events }) => events).some(({ kind }) =>
        kind.type === 'pickupCollected' && kind.kind === 'oilBarrel')).toBe(false)

      const cdp = await contexts[0]!.newCDPSession(one)
      const audioNodes: unknown[] = []
      cdp.on('WebAudio.audioNodeCreated', ({ node }) => audioNodes.push(node))
      await cdp.send('WebAudio.enable')
      await one.getByRole('button', { name: 'Enable sound', exact: true }).click()
      await delay(250)
      const quietBaseline = audioNodes.length
      const admission = (await admissionBaseline(one))!
      const sockets = games[0].sockets
      await disconnectTransport(one)
      await expect.poll(() => games[0].sockets).toBeGreaterThan(sockets)
      await expect.poll(async () => (await admissionBaseline(one))?.index).toBeGreaterThan(admission.index)
      const resumed = (await admissionBaseline(one))!
      expect(resumed.oilBarrels).toContainEqual({ column: 3, row: 1, state: 'burning' })
      const resumedArena = one.getByRole('region', { name: 'Grenadiers courtyard' })
      await expect(resumedArena).toHaveAttribute('tabindex', '0')
      const resumedPixels = await arenaTilePixels(one, 3, 1)
      expect(resumedPixels).not.toEqual(unlitPixels[0])
      expectBurningOilArt(resumedPixels)
      await delay(250)
      expect(audioNodes).toHaveLength(quietBaseline)
      await Promise.all([
        one.screenshot({ path: info.outputPath('oil-burning-desktop-en.png'), fullPage: true }),
        two.screenshot({ path: info.outputPath('oil-burning-polish-320-reduced.png'), fullPage: true }),
      ])
      expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    } finally {
      await Promise.all(contexts.map(async (context) => {
        for (const page of context.pages()) await page.keyboard.up('ArrowRight').catch(() => undefined)
        await context.close()
      }))
    }
  })
