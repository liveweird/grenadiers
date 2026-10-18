import { expect, test } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'
import type { GameServerMessage } from '../../web/src/game/v8/schema'

test.beforeAll(() => expect(process.env.E2E_POWER_BOMB_FIXTURE).toBe('arena-v2-power-bomb'))

const snapshots = (game: ObservedGame) => game.frames.filter(
  (frame): frame is Extract<GameServerMessage, { type: 'snapshot' }> => frame.type === 'snapshot',
)
const events = (game: ObservedGame) => snapshots(game).flatMap(({ events }) => events)

test('two browsers observe one Power Bomb cross a hard wall and the next bomb stay ordinary', async ({ browser }, info) => {
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
    await one.getByRole('checkbox', { name: 'Power bomb', exact: true }).check()
    await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await Promise.all([one, two].map((page) => page.getByRole('button', {
      name: 'Ready', exact: true,
    }).click()))
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type)
      .toBe('running')))
    expect(latestSnapshot(games[0])!.room.availableItems).toEqual([
      'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot',
      'shortFlame', 'longFuse', 'reverseControls',
    ])
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await moveTo(one, games[0], { column: 2, row: 1 })
    await one.keyboard.press('Space')
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])?.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'powerBomb',
    })
    for (const game of games) expect(latestSnapshot(game)!.state.arena.blocks)
      .toContainEqual({ column: 3, row: 2 })
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 2, row: 1 })
    await moveTo(one, games[0], { column: 3, row: 1 })
    await Promise.all(games.map((game) => expect.poll(() =>
      visibleActor(latestSnapshot(game), 'one')?.inventory.weapon).toBe('powerBomb')))
    const collectorId = visibleActor(latestSnapshot(games[0]), 'one')!.id
    expect(games.map((game) => latestSnapshot(game)!.state.bombs)).toEqual([[], []])
    await one.screenshot({ path: info.outputPath('power-armed-desktop-en.png'), fullPage: true })
    await two.getByText('Polski', { exact: true }).click()
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({
      path: info.outputPath('power-armed-polish-320-reduced.png'), fullPage: true,
    })

    await moveTo(one, games[0], { column: 2, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await one.keyboard.press('Space')
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 2, row: 1 })
    await moveTo(one, games[0], { column: 3, row: 1 })
    await Promise.all(games.map((game) => expect.poll(() => events(game).find(({ kind }) =>
      kind.type === 'bombPlaced' && kind.bomb.blast === 'power')).toBeDefined()))
    const placements = games.map((game) => events(game).find(({ kind }) =>
      kind.type === 'bombPlaced' && kind.bomb.blast === 'power')!)
    if (placements.some(({ kind }) => kind.type !== 'bombPlaced')) {
      throw new Error('Power placement expected')
    }
    const powerBomb = placements[0].kind.type === 'bombPlaced' ? placements[0].kind.bomb : null
    if (!powerBomb) throw new Error('Power placement expected')
    expect(placements[1]).toEqual(placements[0])
    expect(powerBomb).toMatchObject({
      cell: { column: 1, row: 2 }, owner: collectorId, range: 2, blast: 'power',
      trigger: { type: 'timed', remainingTicks: 180, paused: false },
    })
    const powerBombId = powerBomb.id
    for (const game of games) expect(visibleActor(latestSnapshot(game), 'one')?.alive).toBe(true)
    await Promise.all(games.map((game) => expect.poll(() =>
      visibleActor(latestSnapshot(game), 'one')?.inventory.weapon).toBeNull()))
    await Promise.all(games.map((game) => expect.poll(() => events(game).find(({ kind }) =>
      kind.type === 'bombDetonated' && kind.bomb.id === powerBombId
      && kind.bomb.blast === 'power')).toBeDefined()))
    await Promise.all(games.map((game) => expect.poll(() => snapshots(game).some(({ state }) => {
      const hasFlame = (column: number, row: number) => state.flames.some((flame) =>
        flame.cell.column === column && flame.cell.row === row)
      return hasFlame(3, 2) && !hasFlame(2, 2) && !hasFlame(3, 1)
    })).toBe(true)))
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length).toBe(0)
    for (const game of games) {
      const blocks = latestSnapshot(game)!.state.arena.blocks
      expect(blocks).not.toContainEqual({ column: 3, row: 2 })
      expect(latestSnapshot(game)!.state.arena.terrain[2 * 15 + 2]).toBe('wall')
    }

    await moveTo(one, games[0], { column: 3, row: 1 })
    const priorPlacedIds = games.map((game) => new Set(events(game).flatMap(({ kind }) =>
      kind.type === 'bombPlaced' ? [kind.bomb.id] : [])))
    await one.keyboard.press('Space')
    await Promise.all(games.map((game, index) => expect.poll(() => events(game).find(({ kind }) =>
      kind.type === 'bombPlaced' && !priorPlacedIds[index].has(kind.bomb.id)
      && kind.bomb.blast === 'standard')).toBeDefined()))
    const ordinaryPlacements = games.map((game, index) => events(game).find(({ kind }) =>
      kind.type === 'bombPlaced' && !priorPlacedIds[index].has(kind.bomb.id)
      && kind.bomb.blast === 'standard')!)
    expect(ordinaryPlacements[1]).toEqual(ordinaryPlacements[0])
    if (ordinaryPlacements[0].kind.type !== 'bombPlaced') {
      throw new Error('Fresh ordinary placement expected')
    }
    expect(ordinaryPlacements[0].kind.bomb).toMatchObject({
      cell: { column: 3, row: 1 }, owner: collectorId, range: 2, blast: 'standard',
      trigger: { type: 'timed', remainingTicks: 180, paused: false },
    })
    for (const game of games) expect(visibleActor(latestSnapshot(game), 'one')?.alive).toBe(true)
  } finally { await Promise.all(contexts.map((context) => context.close())) }
})
