import { expect, test } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'
import type { GameServerMessage } from '../../web/src/game/v8/schema'

test.beforeAll(() => expect(process.env.E2E_REVERSE_CONTROLS_FIXTURE)
  .toBe('arena-v2-reverse-controls'))

const ticks = (game: ObservedGame) => visibleActor(latestSnapshot(game), 'one')?.effects
  .find((effect) => effect.kind === 'reverseControls')?.remainingTicks ?? 0
const room = (game: ObservedGame) => game.frames.filter((frame) =>
  frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room

function collectedAt(game: ObservedGame): number {
  const event = game.frames.flatMap((frame) => frame.type === 'snapshot' ? frame.events : [])
    .find(({ kind }) => kind.type === 'pickupCollected' && kind.kind === 'reverseControls')
  expect(event).toBeDefined()
  return event!.tick
}

function expectExactExpiry(game: ObservedGame, collectionTick: number): void {
  const expiry = collectionTick + 600
  const frames = game.frames.filter((frame): frame is Extract<GameServerMessage, { type: 'snapshot' }> =>
    frame.type === 'snapshot' && frame.state.physicalTick >= collectionTick)
  expect(frames.some((frame) => frame.state.physicalTick < expiry)).toBe(true)
  expect(frames.some((frame) => frame.state.physicalTick >= expiry)).toBe(true)
  for (const frame of frames) {
    const observed = visibleActor(frame, 'one')?.effects
      .find(({ kind }) => kind === 'reverseControls')?.remainingTicks ?? 0
    expect(observed).toBe(Math.max(0, expiry - frame.state.physicalTick))
  }
}

function hasStableReleaseWindow(game: ObservedGame, afterTick: number): boolean {
  const snapshots = game.frames.filter(
    (frame): frame is Extract<GameServerMessage, { type: 'snapshot' }> =>
      frame.type === 'snapshot' && frame.state.physicalTick > afterTick,
  )
  const latest = snapshots.at(-1)
  const position = visibleActor(latest, 'one')?.position
  if (!position) return false
  let stableStart = snapshots.length - 1
  while (stableStart > 0) {
    const observed = visibleActor(snapshots[stableStart - 1], 'one')?.position
    if (observed?.x !== position.x || observed.y !== position.y) break
    stableStart -= 1
  }
  const stable = snapshots.slice(stableStart)
  return stable.length >= 3
    && stable.at(-1)!.state.physicalTick - stable[0].state.physicalTick >= 6
}

test('two browsers hold raw east through Reverse Controls and exact expiry', async ({ browser }, info) => {
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
    await one.getByRole('checkbox', { name: 'Reverse controls', exact: true }).check()
    await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await expect.poll(() => room(games[0])?.settings.enabledItems).toEqual(['reverseControls'])
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await Promise.all([one, two].map((page) => page.getByRole('button', {
      name: 'Ready', exact: true,
    }).click()))
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type)
      .toBe('running')))
    expect(latestSnapshot(games[0])!.room.availableItems).toEqual([
      'heart', 'shield', 'goldenFlame', 'shortFuse', 'shortFlame', 'longFuse', 'reverseControls',
    ])
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await moveTo(one, games[0], { column: 2, row: 1 })
    await one.keyboard.press('Space')
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])?.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'reverseControls',
    })
    await one.screenshot({ path: info.outputPath('reverse-controls-exposed-desktop.png'), fullPage: true })
    await moveTo(one, games[0], { column: 1, row: 1 })

    const commandStart = games[0].commands.length
    await one.keyboard.down('ArrowRight')
    await expect.poll(() => games[0].commands.slice(commandStart)
      .some((command) => command.type === 'movement' && command.direction === 'east')).toBe(true)
    await Promise.all(games.map((game) => expect.poll(() => ticks(game)).toBeGreaterThan(0)))
    const collectionTicks = games.map(collectedAt)
    expect(collectionTicks[0]).toBe(collectionTicks[1])
    await expect.poll(() => visibleActor(latestSnapshot(games[0]), 'one')!.position.x)
      .toBeLessThanOrEqual(1_344)
    const atLeftWall = visibleActor(latestSnapshot(games[0]), 'one')!.position.x
    await Promise.all(games.map((game) => expect.poll(() => ticks(game)).toBe(0)))
    await expect.poll(() => visibleActor(latestSnapshot(games[0]), 'one')!.position.x)
      .toBeGreaterThan(atLeftWall)
    const heldCommands = games[0].commands.slice(commandStart)
      .filter((command) => command.type === 'movement')
    expect(heldCommands.length).toBeGreaterThan(0)
    expect(heldCommands.every(({ direction }) => direction === 'east')).toBe(true)
    const releaseCommandStart = games[0].commands.length
    await one.keyboard.up('ArrowRight')
    await expect.poll(() => games[0].commands.slice(releaseCommandStart).some((command) =>
      command.type === 'movement' && command.direction === null)).toBe(true)
    const releaseTicks = games.map((game) => latestSnapshot(game)!.state.physicalTick)
    await Promise.all(games.map((game, index) => expect.poll(
      () => hasStableReleaseWindow(game, releaseTicks[index]), { intervals: [15] },
    ).toBe(true)))
    expect(visibleActor(latestSnapshot(games[0]), 'one')!.position)
      .toEqual(visibleActor(latestSnapshot(games[1]), 'one')!.position)
    games.forEach((game, index) => expectExactExpiry(game, collectionTicks[index]))
    await two.getByText('Polski', { exact: true }).click()
    await two.screenshot({
      path: info.outputPath('reverse-controls-polish-320-after-release.png'), fullPage: true,
    })
  } finally { await Promise.all(contexts.map((context) => context.close())) }
})
