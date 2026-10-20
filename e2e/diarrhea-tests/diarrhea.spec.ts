import { expect, test } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'
import type { GameServerMessage } from '../../web/src/game/v10/schema'

test.beforeAll(() => expect(process.env.E2E_DIARRHEA_FIXTURE).toBe('arena-v2-diarrhea'))

const ticks = (game: ObservedGame) => visibleActor(latestSnapshot(game), 'one')?.effects
  .find((effect) => effect.kind === 'diarrhea')?.remainingTicks ?? 0
const room = (game: ObservedGame) => game.frames.filter((frame) =>
  frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room
const snapshots = (game: ObservedGame) => game.frames.filter(
  (frame): frame is Extract<GameServerMessage, { type: 'snapshot' }> => frame.type === 'snapshot',
)

function collectionTick(game: ObservedGame): number {
  const event = snapshots(game).flatMap(({ events }) => events)
    .find(({ kind }) => kind.type === 'pickupCollected' && kind.kind === 'diarrhea')
  expect(event).toBeDefined()
  return event!.tick
}

function automaticBomb(game: ObservedGame, collectedAt: number) {
  return snapshots(game).flatMap(({ events }) => events)
    .find(({ kind, tick }) => kind.type === 'bombPlaced' && tick === collectedAt + 1)
}

async function expectAutomaticDetonation(game: ObservedGame, id: number): Promise<void> {
  const find = () => snapshots(game).flatMap(({ events }) => events)
    .find(({ kind }) => kind.type === 'bombDetonated' && kind.bomb.id === id)
  await expect.poll(find).toBeDefined()
  const event = find()
  if (!event || event.kind.type !== 'bombDetonated') throw new Error('automatic detonation expected')
  expect(event.tick).toBe(event.kind.bomb.placedAt + 120)
  expect(event.kind.bomb).toMatchObject({
    id, trigger: { type: 'timed', remainingTicks: 0, paused: false }, blast: 'standard',
  })
}

test('two browsers observe Diarrhea auto placement and death reset', async ({ browser }, info) => {
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
    await one.getByRole('checkbox', { name: 'Diarrhea', exact: true }).check()
    await one.getByLabel('Bomb fuse').selectOption('120')
    await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await expect.poll(() => room(games[0])?.settings.enabledItems).toEqual(['diarrhea'])
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await Promise.all([one, two].map((page) => page.getByRole('button', {
      name: 'Ready', exact: true,
    }).click()))
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type)
      .toBe('running')))
    expect(latestSnapshot(games[0])!.room.availableItems).toEqual([
      'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse',
      'reverseControls',
    ])
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await moveTo(one, games[0], { column: 2, row: 1 })
    await one.keyboard.press('Space')
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])?.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'diarrhea',
    })
    await one.screenshot({ path: info.outputPath('diarrhea-exposed-desktop-en.png'), fullPage: true })
    await two.getByText('Polski', { exact: true }).click()
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({
      path: info.outputPath('diarrhea-exposed-polish-320-reduced.png'), fullPage: true,
    })

    const manualActions = games[0].commands.filter(({ type }) => type === 'manualAction').length
    expect(manualActions).toBe(1)
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 3, row: 1 })
    await Promise.all(games.map((game) => expect.poll(() => ticks(game)).toBeGreaterThan(0)))
    const collected = games.map(collectionTick)
    expect(collected[0]).toBe(collected[1])
    await Promise.all(games.map((game, index) => expect.poll(
      () => automaticBomb(game, collected[index]), { intervals: [15] },
    ).toBeDefined()))
    const automaticIds = games.map((game, index) => {
      const event = automaticBomb(game, collected[index])!
      if (event.kind.type !== 'bombPlaced') throw new Error('automatic placement expected')
      expect(event.kind.bomb).toMatchObject({
        owner: visibleActor(latestSnapshot(game), 'one')!.id,
        placedAt: collected[index] + 1,
        trigger: { type: 'timed', remainingTicks: 120, paused: false },
        blast: 'standard',
      })
      return event.kind.bomb.id
    })
    expect(automaticIds[1]).toBe(automaticIds[0])
    expect(games[0].commands.filter(({ type }) => type === 'manualAction')).toHaveLength(manualActions)
    await Promise.all(games.map((game) => expectAutomaticDetonation(game, automaticIds[0])))
    await Promise.all(games.map((game) => expect.poll(() =>
      visibleActor(latestSnapshot(game), 'one')?.alive).toBe(false)))
    expect(games[0].commands.filter(({ type }) => type === 'manualAction')).toHaveLength(manualActions)
    for (const game of games) expect(ticks(game)).toBe(0)
    const firstRound = latestSnapshot(games[0])!.room.roundId
    await Promise.all(games.map((game) => expect.poll(() => {
      const latest = latestSnapshot(game)
      return latest?.room.phase.type === 'running' && latest.room.roundId > firstRound
    }).toBe(true)))
    for (const game of games) {
      expect(ticks(game)).toBe(0)
      expect(latestSnapshot(game)!.state.bombs).toEqual([])
    }
  } finally { await Promise.all(contexts.map((context) => context.close())) }
})
