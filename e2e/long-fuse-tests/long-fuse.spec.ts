import { expect, test, type Page } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'
import type { GameServerMessage } from '../../web/src/game/v9/schema'

test.beforeAll(() => expect(process.env.E2E_LONG_FUSE_FIXTURE).toBe('arena-v2-long-fuse'))

const ticks = (game: ObservedGame) => visibleActor(latestSnapshot(game), 'one')?.effects
  .find((effect) => effect.kind === 'longFuse')?.remainingTicks ?? 0
const room = (game: ObservedGame) => game.frames.filter((frame) =>
  frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room

type PlacementCursor = { roundId: number; tick: number }

function placedBomb(game: ObservedGame, cursor: PlacementCursor) {
  return game.frames.flatMap((frame) => frame.type === 'snapshot'
    && frame.room.roundId === cursor.roundId ? frame.events : [])
    .find(({ tick, kind }) => tick > cursor.tick && kind.type === 'bombPlaced')?.kind
}

function manualBombs(game: ObservedGame) {
  return game.commands.filter((command) => command.type === 'manualAction')
}

async function plant(page: Page, game: ObservedGame, expectedActionId: number): Promise<number> {
  const commandStart = game.commands.length
  const before = latestSnapshot(game)!
  const cursor = { roundId: before.room.roundId, tick: before.state.physicalTick }
  await page.keyboard.press('Space')
  await expect.poll(() => game.commands.slice(commandStart)
    .filter((command) => command.type === 'manualAction')).toEqual([{
    type: 'manualAction', action: 'bomb', actionId: expectedActionId,
  }])
  await expect.poll(() => {
    const event = placedBomb(game, cursor)
    return event?.type === 'bombPlaced' ? event.bomb.id : undefined
  }).toBeDefined()
  const event = placedBomb(game, cursor)
  if (!event || event.type !== 'bombPlaced') throw new Error('Bomb placement event expected')
  expect(manualBombs(game)).toEqual(Array.from({ length: expectedActionId }, (_, index) => ({
    type: 'manualAction', action: 'bomb', actionId: index + 1,
  })))
  return event.bomb.id
}

function captured(game: ObservedGame, id: number, duration: number): boolean {
  const frame = game.frames.filter((value) => value.type === 'snapshot'
    && value.state.bombs.some((bomb) => bomb.id === id)).at(-1)
  if (!frame || frame.type !== 'snapshot') return false
  const bomb = frame.state.bombs.find((value) => value.id === id)!
  return bomb.trigger.type === 'timed' && !bomb.trigger.paused
    && frame.state.physicalTick - bomb.placedAt + bomb.trigger.remainingTicks === duration
}

function collectedAt(game: ObservedGame): number {
  const event = game.frames.flatMap((frame) => frame.type === 'snapshot' ? frame.events : [])
    .find(({ kind }) => kind.type === 'pickupCollected' && kind.kind === 'longFuse')
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
      .find(({ kind }) => kind === 'longFuse')?.remainingTicks ?? 0
    expect(observed).toBe(Math.max(0, expiry - frame.state.physicalTick))
  }
}

async function expectDetonation(game: ObservedGame, id: number, duration: number): Promise<void> {
  const event = () => game.frames.flatMap((frame) => frame.type === 'snapshot' ? frame.events : [])
    .find(({ kind }) => kind.type === 'bombDetonated' && kind.bomb.id === id)
  await expect.poll(event).toBeDefined()
  const found = event()!
  if (found.kind.type !== 'bombDetonated') throw new Error('detonation expected')
  expect(found.tick - found.kind.bomb.placedAt).toBe(duration)
  expect(found.kind.bomb.trigger).toEqual({ type: 'timed', remainingTicks: 0, paused: false })
}

function expectRunningRoundSurvival(game: ObservedGame, roundId: number): void {
  const snapshot = latestSnapshot(game)!
  expect(snapshot.room.roundId).toBe(roundId)
  expect(snapshot.room.phase.type).toBe('running')
  expect(visibleActor(snapshot, 'one')?.alive).toBe(true)
}

function publicSummary(game: ObservedGame) {
  const snapshot = latestSnapshot(game)
  return {
    retainedFrames: game.frames.length,
    commands: game.commands.filter((command) => command.type === 'movement'
      || command.type === 'manualAction').slice(-128),
    latest: snapshot && {
      roundId: snapshot.room.roundId,
      physicalTick: snapshot.state.physicalTick,
      phase: snapshot.room.phase.type,
      actors: snapshot.state.actors.map((actor) => actor.type === 'visible' ? {
        type: actor.type, seat: actor.seat, position: actor.position, alive: actor.alive,
        liveBombCount: actor.liveBombCount,
      } : { type: actor.type, seat: actor.seat, alive: actor.alive }),
      bombs: snapshot.state.bombs.map((bomb) => ({
        id: bomb.id, cell: bomb.cell, placedAt: bomb.placedAt, trigger: bomb.trigger,
      })),
    },
    bombEvents: game.frames.flatMap((frame) => frame.type === 'snapshot'
      ? frame.events.filter(({ kind }) => kind.type === 'bombPlaced'
        || kind.type === 'bombDetonated').map((event) => {
        if (event.kind.type !== 'bombPlaced' && event.kind.type !== 'bombDetonated') {
          throw new Error('Bomb event expected')
        }
        return {
          tick: event.tick, index: event.index, type: event.kind.type,
          bomb: {
            id: event.kind.bomb.id, cell: event.kind.bomb.cell,
            placedAt: event.kind.bomb.placedAt, trigger: event.kind.bomb.trigger,
          },
        }
      }) : []),
  }
}

async function focusIdentity(page: Page) {
  return page.evaluate(() => {
    const active = document.activeElement
    return {
      documentFocused: document.hasFocus(),
      active: active ? {
        tag: active.tagName.toLowerCase(), role: active.getAttribute('role'),
        ariaLabel: active.getAttribute('aria-label'),
      } : null,
    }
  })
}

test('two browsers collect Long Fuse and restore ordinary bomb timing after its exact deadline', async ({ browser }, info) => {
  const contexts = await Promise.all([
    browser.newContext({ viewport: { width: 1440, height: 1050 } }),
    browser.newContext({ viewport: { width: 320, height: 844 }, reducedMotion: 'reduce' }),
  ])
  const pages: Page[] = []
  const games: ObservedGame[] = []
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    pages.push(one, two)
    await installTransportProbe(one)
    games.push(observeGame(one), observeGame(two))
    await Promise.all([one, two].map((page) => page.goto('/')))
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await one.getByRole('checkbox', { name: 'Heart', exact: true }).uncheck()
    await one.getByRole('checkbox', { name: 'Long fuse', exact: true }).check()
    await one.getByLabel('Bomb fuse').selectOption('120')
    await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await expect.poll(() => room(games[0])?.settings.enabledItems).toEqual(['longFuse'])
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await Promise.all([one, two].map((page) => page.getByRole('button', { name: 'Ready', exact: true }).click()))
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
    expect(latestSnapshot(games[0])!.room.availableItems).toEqual(
      ['heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls'])
    const roundId = latestSnapshot(games[0])!.room.roundId
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await moveTo(one, games[0], { column: 2, row: 1 })
    const exposingBomb = await plant(one, games[0], 1)
    await moveTo(one, games[0], { column: 1, row: 1 }); await moveTo(one, games[0], { column: 1, row: 2 })
    await Promise.all(games.map((game) => expect.poll(() => captured(game, exposingBomb, 120)).toBe(true)))
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])?.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'longFuse',
    })
    await Promise.all(games.map((game) => expectDetonation(game, exposingBomb, 120)))
    games.forEach((game) => expectRunningRoundSurvival(game, roundId))
    await one.screenshot({ path: info.outputPath('long-fuse-exposed-desktop.png'), fullPage: true })
    await moveTo(one, games[0], { column: 1, row: 1 }); await moveTo(one, games[0], { column: 3, row: 1 })
    await Promise.all(games.map((game) => expect.poll(() => ticks(game)).toBeGreaterThan(0)))
    const collectionTicks = games.map(collectedAt)
    expect(collectionTicks[0]).toBe(collectionTicks[1])
    const longBomb = await plant(one, games[0], 2)
    await moveTo(one, games[0], { column: 1, row: 1 }); await moveTo(one, games[0], { column: 1, row: 2 })
    await Promise.all(games.map((game) => expect.poll(() => captured(game, longBomb, 240)).toBe(true)))
    await two.getByText('Polski', { exact: true }).click()
    await two.screenshot({ path: info.outputPath('long-fuse-polish-320-after-escape.png'), fullPage: true })
    await Promise.all(games.map((game) => expectDetonation(game, longBomb, 240)))
    games.forEach((game) => expectRunningRoundSurvival(game, roundId))
    await disconnectTransport(one)
    await expect.poll(async () => (await admissionBaseline(one))?.actionFloor).toBe(2)
    await Promise.all(games.map((game) => expect.poll(() => ticks(game)).toBe(0)))
    games.forEach((game, index) => expectExactExpiry(game, collectionTicks[index]))
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    const ordinary = await plant(one, games[0], 3)
    await Promise.all(games.map((game) => expect.poll(() => captured(game, ordinary, 120)).toBe(true)))
    await Promise.all(games.map((game) => expect.poll(() => visibleActor(latestSnapshot(game), 'one')?.alive).toBe(false)))
    await Promise.all(games.map((game) => expectDetonation(game, ordinary, 120)))
    expect(manualBombs(games[0])).toEqual([1, 2, 3].map((actionId) => ({
      type: 'manualAction', action: 'bomb', actionId,
    })))
  } finally {
    try {
      await info.attach('long-fuse-public-summary', {
        contentType: 'application/json',
        body: JSON.stringify({
          peers: games.map(publicSummary),
          focus: await Promise.all(pages.map((page) => focusIdentity(page)
            .catch(() => ({ documentFocused: false, active: null })))),
        }),
      })
    } finally { await Promise.all(contexts.map((context) => context.close())) }
  }
})
