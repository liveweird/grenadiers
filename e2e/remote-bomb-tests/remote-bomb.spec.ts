import { expect, test } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'
import type { GameServerMessage } from '../../web/src/game/v10/schema'

test.beforeAll(() => expect(process.env.E2E_REMOTE_BOMB_FIXTURE).toBe('arena-v2-remote-bomb'))

const snapshots = (game: ObservedGame) => game.frames.filter(
  (frame): frame is Extract<GameServerMessage, { type: 'snapshot' }> => frame.type === 'snapshot',
)
const events = (game: ObservedGame) => snapshots(game).flatMap(({ events: value }) => value)
const manualActions = (game: ObservedGame) => game.commands.filter(({ type }) =>
  type === 'manualAction')

test('two browsers observe one Remote Bomb wait past its fuse and detonate on command', async ({ browser }, info) => {
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
    await one.getByRole('checkbox', { name: 'Remote bomb', exact: true }).check()
    await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await Promise.all([one, two].map((page) => page.getByRole('button', {
      name: 'Ready', exact: true,
    }).click()))
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type)
      .toBe('running')))
    expect(latestSnapshot(games[0])!.room.availableItems).toEqual([
      'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse',
      'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'teleporter',
    ])
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()

    await moveTo(one, games[0], { column: 2, row: 1 })
    await one.keyboard.press('Space')
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])?.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'remoteBomb',
    })
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 2, row: 1 })
    await moveTo(one, games[0], { column: 3, row: 1 })
    await Promise.all(games.map((game) => expect.poll(() =>
      visibleActor(latestSnapshot(game), 'one')?.inventory.weapon).toBe('remoteBomb')))
    const collectorId = visibleActor(latestSnapshot(games[0]), 'one')!.id

    const placementActionFloor = manualActions(games[0]).length
    await one.keyboard.press('Space')
    await moveTo(one, games[0], { column: 2, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await Promise.all(games.map((game) => expect.poll(() => events(game).find(({ kind }) =>
      kind.type === 'bombPlaced' && kind.bomb.trigger.type === 'remote')).toBeDefined()))
    const placements = games.map((game) => events(game).find(({ kind }) =>
      kind.type === 'bombPlaced' && kind.bomb.trigger.type === 'remote')!)
    expect(placements[1]).toEqual(placements[0])
    if (placements[0].kind.type !== 'bombPlaced') throw new Error('Remote placement expected')
    const remote = placements[0].kind.bomb
    expect(remote).toMatchObject({
      cell: { column: 3, row: 1 }, owner: collectorId, range: 2, blast: 'standard',
      trigger: { type: 'remote' },
    })
    expect(manualActions(games[0])).toHaveLength(placementActionFloor + 1)
    await Promise.all(games.map((game) => expect.poll(() =>
      visibleActor(latestSnapshot(game), 'one')?.inventory.weapon).toBeNull()))

    await one.screenshot({ path: info.outputPath('remote-armed-desktop-en.png'), fullPage: true })
    await two.getByText('Polski', { exact: true }).click()
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({
      path: info.outputPath('remote-armed-polish-320-reduced.png'), fullPage: true,
    })

    const quietActionFloor = manualActions(games[0]).length
    await Promise.all(games.map((game) => expect.poll(() => {
      const snapshot = latestSnapshot(game)
      return snapshot && snapshot.state.physicalTick > remote.placedAt + 183
        && snapshot.state.bombs.some(({ id }) => id === remote.id)
    }, { timeout: 10_000 }).toBe(true)))
    expect(manualActions(games[0])).toHaveLength(quietActionFloor)
    for (const game of games) {
      expect(events(game).some(({ kind }) => kind.type === 'bombDetonated'
        && kind.bomb.id === remote.id)).toBe(false)
    }

    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    // The observer retains a rolling frame window: compare causal IDs, not shrinking totals.
    const priorBombIds = games.map((game) => new Set(events(game).flatMap(({ kind }) =>
      kind.type === 'bombPlaced' ? [kind.bomb.id] : [])))
    await one.keyboard.press('Space')
    await Promise.all(games.map((game) => expect.poll(() => events(game).find(({ kind }) =>
      kind.type === 'bombDetonated' && kind.bomb.id === remote.id)).toBeDefined()))
    expect(manualActions(games[0])).toHaveLength(quietActionFloor + 1)
    for (const [index, game] of games.entries()) {
      expect(events(game).filter(({ kind }) => kind.type === 'bombPlaced'
        && !priorBombIds[index].has(kind.bomb.id))).toEqual([])
      expect(latestSnapshot(game)!.state.bombs.some(({ id }) => id === remote.id)).toBe(false)
      expect(visibleActor(latestSnapshot(game), 'one')?.inventory.weapon).toBeNull()
    }

    await expect.poll(() => latestSnapshot(games[0])!.state.flames.length).toBe(0)
    const ordinaryActionFloor = manualActions(games[0]).length
    await one.keyboard.press('Space')
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 2, row: 1 })
    await moveTo(one, games[0], { column: 3, row: 1 })
    await Promise.all(games.map((game, index) => expect.poll(() => events(game).find(({ kind }) =>
      kind.type === 'bombPlaced' && !priorBombIds[index].has(kind.bomb.id)
      && kind.bomb.id !== remote.id && kind.bomb.trigger.type === 'timed')).toBeDefined()))
    const ordinary = games.map((game, index) => events(game).find(({ kind }) =>
      kind.type === 'bombPlaced' && !priorBombIds[index].has(kind.bomb.id)
      && kind.bomb.id !== remote.id && kind.bomb.trigger.type === 'timed')!)
    expect(ordinary[1]).toEqual(ordinary[0])
    if (ordinary[0].kind.type !== 'bombPlaced') throw new Error('Timed placement expected')
    const ordinaryBomb = ordinary[0].kind.bomb
    expect(ordinaryBomb).toMatchObject({
      cell: { column: 1, row: 2 }, owner: collectorId, range: 2, blast: 'standard',
      trigger: { type: 'timed', remainingTicks: 180, paused: false },
    })
    expect(ordinaryBomb.id).not.toBe(remote.id)
    expect(manualActions(games[0])).toHaveLength(ordinaryActionFloor + 1)
    await Promise.all(games.map((game) => expect.poll(() => events(game).find(({ kind }) =>
      kind.type === 'bombDetonated' && kind.bomb.id === ordinaryBomb.id)).toBeDefined()))
    for (const game of games) expect(visibleActor(latestSnapshot(game), 'one')?.alive).toBe(true)
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
