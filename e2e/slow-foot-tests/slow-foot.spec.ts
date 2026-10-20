import { expect, test } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'
import type { GameServerMessage } from '../../web/src/game/v10/schema'

test.beforeAll(() => expect(process.env.E2E_SLOW_FOOT_FIXTURE).toBe('arena-v2-slow-foot'))

const room = (game: ObservedGame) => game.frames.filter((frame) =>
  frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room
const snapshots = (game: ObservedGame) => game.frames.filter(
  (frame): frame is Extract<GameServerMessage, { type: 'snapshot' }> => frame.type === 'snapshot',
)
const remaining = (frame: Extract<GameServerMessage, { type: 'snapshot' }>) =>
  visibleActor(frame, 'one')?.effects.find(({ kind }) => kind === 'slowFoot')?.remainingTicks ?? 0

function collectionTick(game: ObservedGame): number {
  const event = snapshots(game).flatMap(({ events }) => events)
    .find(({ kind }) => kind.type === 'pickupCollected' && kind.kind === 'slowFoot')
  expect(event).toBeDefined()
  return event!.tick
}

function exactTimer(game: ObservedGame, collectedAt: number): void {
  for (const frame of snapshots(game).filter(({ state }) => state.physicalTick >= collectedAt)) {
    expect(remaining(frame)).toBe(Math.max(0, collectedAt + 600 - frame.state.physicalTick))
  }
}

function sharedDisplacementWindow(games: ObservedGame[], after: number, active: boolean) {
  const peerFrames = new Map(snapshots(games[1]).map((frame) => [frame.state.physicalTick, frame]))
  const eligible = snapshots(games[0]).flatMap((first) => {
    const second = peerFrames.get(first.state.physicalTick)
    if (!second) return []
    const actors = [visibleActor(first, 'one'), visibleActor(second, 'one')]
    return first.state.physicalTick > after && actors.every((actor) => actor !== undefined
      && actor.position.x > 448 && actor.position.x < 3_392)
      && [first, second].every((frame) => Boolean(remaining(frame)) === active)
      ? [[first, second] as const] : []
  })
  for (const first of eligible) {
    const last = eligible.find(([frame]) =>
      frame.state.physicalTick - first[0].state.physicalTick >= 12)
    if (last) return { first, last }
  }
  return undefined
}

function stableWindow(game: ObservedGame, after: number): boolean {
  const frames = snapshots(game).filter(({ state }) => state.physicalTick > after)
  const last = frames.at(-1)
  const position = visibleActor(last, 'one')?.position
  if (!last || !position) return false
  let stableStart = frames.length - 1
  while (stableStart > 0) {
    const observed = visibleActor(frames[stableStart - 1], 'one')?.position
    if (observed?.x !== position.x || observed.y !== position.y) break
    stableStart -= 1
  }
  const stable = frames.slice(stableStart)
  return stable.length >= 3
    && stable.at(-1)!.state.physicalTick - stable[0].state.physicalTick >= 6
}

function expectRate(window: NonNullable<ReturnType<typeof sharedDisplacementWindow>>,
  direction: 'east' | 'west', unitsPerSecond: number): void {
  const ticks = window.last[0].state.physicalTick - window.first[0].state.physicalTick
  for (let peer = 0; peer < 2; peer += 1) {
    const first = visibleActor(window.first[peer], 'one')!
    const last = visibleActor(window.last[peer], 'one')!
    const distance = direction === 'west'
      ? first.position.x - last.position.x : last.position.x - first.position.x
    expect(Math.abs(distance - unitsPerSecond * ticks / 60)).toBeLessThanOrEqual(1)
    expect(last.position.y).toBe(first.position.y)
    expect(last.heading).toBe(direction)
  }
  expect(visibleActor(window.first[1], 'one')!.position)
    .toEqual(visibleActor(window.first[0], 'one')!.position)
  expect(visibleActor(window.last[1], 'one')!.position)
    .toEqual(visibleActor(window.last[0], 'one')!.position)
}

test('two browsers observe one Slow-foot stack and exact restored movement', async ({ browser }, info) => {
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
    await one.getByRole('checkbox', { name: 'Slow foot', exact: true }).check()
    await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await expect.poll(() => room(games[0])?.settings.enabledItems).toEqual(['slowFoot'])
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await Promise.all([one, two].map((page) => page.getByRole('button', {
      name: 'Ready', exact: true,
    }).click()))
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type)
      .toBe('running')))
    expect(latestSnapshot(games[0])!.room.availableItems).toEqual([
      'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame',
      'longFuse', 'reverseControls', 'invisibleBombs', 'teleporter',
    ])
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await moveTo(one, games[0], { column: 2, row: 1 })
    await one.keyboard.press('Space')
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])?.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'slowFoot',
    })
    await one.screenshot({ path: info.outputPath('slow-foot-exposed-desktop-en.png'), fullPage: true })
    await two.getByText('Polski', { exact: true }).click()
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({
      path: info.outputPath('slow-foot-exposed-polish-320-reduced.png'), fullPage: true,
    })

    await moveTo(one, games[0], { column: 3, row: 1 })
    await Promise.all(games.map((game) => expect.poll(() =>
      remaining(latestSnapshot(game)!)).toBeGreaterThan(0)))
    const collected = games.map(collectionTick)
    expect(collected[1]).toBe(collected[0])
    const westCommandStart = games[0].commands.length
    await one.keyboard.down('ArrowLeft')
    await expect.poll(() => games[0].commands.slice(westCommandStart).some((command) =>
      command.type === 'movement' && command.direction === 'west')).toBe(true)
    await expect.poll(() => sharedDisplacementWindow(games, collected[0], true)).toBeDefined()
    expectRate(sharedDisplacementWindow(games, collected[0], true)!, 'west', 1_536)
    await expect.poll(() => visibleActor(latestSnapshot(games[0]), 'one')!.position.x)
      .toBeLessThanOrEqual(384)
    await Promise.all(games.map((game) => expect.poll(() =>
      remaining(latestSnapshot(game)!)).toBe(0)))
    await one.keyboard.up('ArrowLeft')
    const restoredAt = latestSnapshot(games[0])!.state.physicalTick
    const eastCommandStart = games[0].commands.length
    await one.keyboard.down('ArrowRight')
    await expect.poll(() => games[0].commands.slice(eastCommandStart).some((command) =>
      command.type === 'movement' && command.direction === 'east')).toBe(true)
    await expect.poll(() => sharedDisplacementWindow(games, restoredAt, false)).toBeDefined()
    expectRate(sharedDisplacementWindow(games, restoredAt, false)!, 'east', 3_072)
    const releaseCommandStart = games[0].commands.length
    await one.keyboard.up('ArrowRight')
    await expect.poll(() => games[0].commands.slice(releaseCommandStart).some((command) =>
      command.type === 'movement' && command.direction === null)).toBe(true)
    const releasedAt = games.map((game) => latestSnapshot(game)!.state.physicalTick)
    await Promise.all(games.map((game, index) => expect.poll(
      () => stableWindow(game, releasedAt[index]), { intervals: [15] },
    ).toBe(true)))
    games.forEach((game, index) => exactTimer(game, collected[index]))
  } finally { await Promise.all(contexts.map((context) => context.close())) }
})
