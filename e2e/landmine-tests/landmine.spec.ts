import { createResponsiveBrowserContexts } from '../tests/support/browser-contexts'
import { expect, test } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'
import { applyItemSettings } from '../tests/support/settings'
import type { GameServerMessage } from '../../web/src/game/v17/schema'

test.beforeAll(() => expect(process.env.E2E_LANDMINE_FIXTURE).toBe('arena-v3-landmine'))

const snapshots = (game: ObservedGame) => game.frames.filter(
  (frame): frame is Extract<GameServerMessage, { type: 'snapshot' }> => frame.type === 'snapshot',
)
const events = (game: ObservedGame) => snapshots(game).flatMap(({ events: value }) => value)
const manualActions = (game: ObservedGame) => game.commands.filter(({ type }) => type === 'manualAction')

test('two browsers observe one persistent mine and its contact detonation', async ({ browser }, info) => {
  const contexts = await createResponsiveBrowserContexts(browser)
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    const games = [observeGame(one), observeGame(two)]
    await Promise.all([one, two].map((page) => page.goto('/')))
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await applyItemSettings(one, games[0], [{ id: 'landmine', label: 'Landmine' }])
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await Promise.all([one, two].map((page) => page.getByRole('button', { name: 'Ready', exact: true }).click()))
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
    expect(latestSnapshot(games[0])!.room.availableItems).toEqual([
      'kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse',
      'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'oilBarrel', 'teleporter',
    ])
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()

    await moveTo(one, games[0], { column: 2, row: 1 })
    await one.keyboard.press('Space')
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])?.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'landmine',
    })
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 2, row: 1 })
    await moveTo(one, games[0], { column: 3, row: 1 })
    await Promise.all(games.map((game) => expect.poll(() =>
      visibleActor(latestSnapshot(game), 'one')?.inventory.weapon).toBe('landmine')))
    const owner = visibleActor(latestSnapshot(games[0]), 'one')!.id
    const capacity = visibleActor(latestSnapshot(games[0]), 'one')!.bombCapacity

    const actionFloor = manualActions(games[0]).length
    await one.keyboard.press('Space')
    await Promise.all(games.map((game) => expect.poll(() => events(game).find(({ kind }) =>
      kind.type === 'bombPlaced' && kind.bomb.trigger.type === 'contact')).toBeDefined()))
    const placements = games.map((game) => events(game).find(({ kind }) =>
      kind.type === 'bombPlaced' && kind.bomb.trigger.type === 'contact')!)
    expect(placements[1]).toEqual(placements[0])
    if (placements[0].kind.type !== 'bombPlaced') throw new Error('Landmine placement expected')
    const mine = placements[0].kind.bomb
    expect(mine).toMatchObject({
      cell: { column: 3, row: 1 }, owner, range: 2, trigger: { type: 'contact' }, blast: 'standard',
    })
    expect(manualActions(games[0])).toHaveLength(actionFloor + 1)
    await Promise.all(games.map((game) => expect.poll(() =>
      visibleActor(latestSnapshot(game), 'one')?.inventory.weapon).toBeNull()))
    await Promise.all(games.map((game) => expect.poll(() => {
      const snapshot = latestSnapshot(game)
      const actor = visibleActor(snapshot, 'one')
      if (!snapshot || !actor) return false
      return (snapshot.state.physicalTick ?? 0) >= mine.placedAt + 6
        && snapshot.state.bombs.some(({ id }) => id === mine.id)
        && actor.alive
        && Math.floor(actor.position.x / 1024) === 3
        && Math.floor(actor.position.y / 1024) === 1
    }).toBe(true)))

    for (const game of games) {
      expect(visibleActor(latestSnapshot(game), 'one')).toMatchObject({
        bombCapacity: capacity, liveBombCount: 0,
      })
    }
    await moveTo(one, games[0], { column: 2, row: 1 })
    await one.screenshot({ path: info.outputPath('landmine-armed-desktop-en.png'), fullPage: true })
    await two.getByText('Polski', { exact: true }).click()
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({ path: info.outputPath('landmine-armed-polish-320-reduced.png'), fullPage: true })

    await Promise.all(games.map((game) => expect.poll(() => {
      const snapshot = latestSnapshot(game)
      return snapshot && snapshot.state.physicalTick > mine.placedAt + 183
        && snapshot.state.bombs.some(({ id }) => id === mine.id)
    }, { timeout: 10_000 }).toBe(true)))
    expect(manualActions(games[0])).toHaveLength(actionFloor + 1)
    for (const game of games) {
      expect(events(game).some(({ kind }) => kind.type === 'bombDetonated' && kind.bomb.id === mine.id)).toBe(false)
    }

    // Retained history may shrink while contact is awaited; fresh IDs prove new placements.
    const priorBombIds = games.map((game) => new Set(events(game).flatMap(({ kind }) =>
      kind.type === 'bombPlaced' ? [kind.bomb.id] : [])))
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await one.keyboard.down('ArrowRight')
    try {
      await Promise.all(games.map((game) => expect.poll(() => snapshots(game).find((snapshot) =>
        snapshot.events.some(({ kind }) => kind.type === 'bombDetonated' && kind.bomb.id === mine.id)
        && snapshot.state.flames.some(({ cell, expiresAt }) => cell.column === mine.cell.column
          && cell.row === mine.cell.row && expiresAt > snapshot.state.physicalTick))).toBeDefined()))
    } finally {
      await one.keyboard.up('ArrowRight')
    }
    for (const [index, game] of games.entries()) {
      const detonation = snapshots(game).find((snapshot) =>
        snapshot.events.some(({ kind }) => kind.type === 'bombDetonated' && kind.bomb.id === mine.id)
        && snapshot.state.flames.some(({ cell, expiresAt }) => cell.column === mine.cell.column
          && cell.row === mine.cell.row && expiresAt > snapshot.state.physicalTick))
      if (!detonation) throw new Error('Active Landmine detonation snapshot expected')
      const originFlame = detonation.state.flames.find(({ cell }) =>
        cell.column === mine.cell.column && cell.row === mine.cell.row)
      expect(originFlame?.expiresAt).toBeGreaterThan(detonation.state.physicalTick)
      expect(detonation.state.bombs.some(({ id }) => id === mine.id)).toBe(false)
      expect(events(game).filter(({ kind }) => kind.type === 'bombPlaced'
        && !priorBombIds[index].has(kind.bomb.id))).toEqual([])
      expect(events(game).filter(({ kind }) => kind.type === 'bombDetonated'
        && kind.bomb.id === mine.id && kind.bomb.trigger.type === 'contact')).toHaveLength(1)
      expect(latestSnapshot(game)!.state.bombs.some(({ id }) => id === mine.id)).toBe(false)
      expect(visibleActor(latestSnapshot(game), 'one')).toMatchObject({
        alive: false, bombCapacity: capacity, liveBombCount: 0,
      })
      expect(visibleActor(latestSnapshot(game), 'two')?.alive).toBe(true)
      expect(events(game).filter(({ kind }) => kind.type === 'actorEliminated')).toHaveLength(1)
      expect(latestSnapshot(game)?.room.phase.type).toBe('result')
      expect(latestSnapshot(game)?.room.lastRound?.result).toEqual({ type: 'winner', seat: 'two' })
    }
    expect(manualActions(games[0])).toHaveLength(actionFloor + 1)
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
