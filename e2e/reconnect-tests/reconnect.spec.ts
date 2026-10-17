import AxeBuilder from '@axe-core/playwright'
import { setTimeout as delay } from 'node:timers/promises'
import { expect, test, type Page, type Route, type TestInfo } from '@playwright/test'
import { visibleActor, latestSnapshot, observeGame, type ObservedGame } from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'
import {
  admissionBaseline, disconnectTransport, installTransportProbe, invalidResume,
  sharedFieldsArePrivate, takeover,
} from '../tests/support/reconnect'

test.beforeAll(() => {
  expect(process.env.E2E_MATCH_FIXTURE, 'Configure the documented match/reconnect fixture')
    .toBe('arena-v1-matches')
})

async function joinMatch(pages: Page[], games: ObservedGame[]) {
  const [one, two] = pages
  await Promise.all(pages.map((page) => page.goto('/')))
  await one.getByRole('button', { name: 'Create room', exact: true }).click()
  const code = one.getByLabel('Room invitation code', { exact: true })
  await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
  await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
  await two.getByRole('button', { name: 'Join room', exact: true }).click()
  await one.getByRole('button', { name: 'Ready', exact: true }).click()
  await two.getByRole('button', { name: 'Ready', exact: true }).click()
  for (const game of games) await expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')
}

async function holdSessions(page: Page): Promise<() => void> {
  let release!: () => void
  const gate = new Promise<void>((resolve) => { release = resolve })
  const handler = async (route: Route) => { await gate; await route.continue() }
  await page.route('**/api/v1/game/session', handler)
  return release
}

async function assertPublicOnly(pages: Page[]) {
  for (const page of pages) expect(await sharedFieldsArePrivate(page)).toBe(true)
}

async function attachPublicSummary(testInfo: TestInfo, games: ObservedGame[]) {
  await testInfo.attach('reconnect-public-summary', {
    contentType: 'application/json',
    body: JSON.stringify(games.map((game) => ({
      sockets: game.sockets,
      commands: game.commands.slice(-128),
      rooms: game.frames.filter((frame) => frame.type === 'roomState' || frame.type === 'snapshot' || frame.type === 'admitted')
        .slice(-60).map((frame) => ({ type: frame.type, room: frame.room })),
    }))),
  })
}

test('a browser restores its full live world and action floor before a fenced takeover', async ({ browser }, testInfo) => {
  const contexts = await Promise.all([
    browser.newContext({ viewport: { width: 1440, height: 1050 } }),
    browser.newContext({ viewport: { width: 320, height: 844 }, reducedMotion: 'reduce' }),
  ])
  let release: (() => void) | undefined
  const games: ObservedGame[] = []
  try {
    const pages = await Promise.all(contexts.map((context) => context.newPage()))
    await Promise.all(pages.map(installTransportProbe))
    games.push(...pages.map(observeGame))
    const [one] = pages
    await joinMatch(pages, games)
    const arena = one.getByRole('region', { name: 'Grenadiers courtyard' })
    await arena.click()
    await moveTo(one, games[0], { column: 5, row: 1 })
    const before = latestSnapshot(games[0])!
    const position = visibleActor(before, 'one')!.position
    release = await holdSessions(one)
    await one.keyboard.press('Space')
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length, { intervals: [15] }).toBe(1)
    const bomb = latestSnapshot(games[0])!.state.bombs[0]
    await disconnectTransport(one)
    await expect.poll(() => latestSnapshot(games[1])?.room.seats[0].connection.type, { intervals: [15] })
      .toBe('disconnected')
    await release()
    release = undefined
    await expect.poll(async () => (await admissionBaseline(one))?.actionFloor, { intervals: [15] }).toBe(1)
    const restored = (await admissionBaseline(one))!
    expect(restored.roomId).toBe(before.room.roomId)
    expect(restored.seat).toBe('one')
    expect(restored.hasState).toBe(true)
    expect(restored.tick).toBeGreaterThan(before.room.tick)
    expect(restored.positions.find((actor) => actor.seat === 'one')).toEqual({ seat: 'one', ...position })
    expect(restored.bombIds).toContain(bomb.id)
    expect(restored.stableCapability).toBe(true)
    await arena.click()
    await one.keyboard.down('ArrowLeft')
    try {
      await expect.poll(() => visibleActor(latestSnapshot(games[0]), 'one')!.position.x, { intervals: [15] })
        .toBeLessThanOrEqual(2_560)
    } finally { await one.keyboard.up('ArrowLeft') }
    await expect.poll(() => {
      const state = latestSnapshot(games[0])!.state
      return state.bombs.length + state.flames.length
    }).toBe(0)
    expect(latestSnapshot(games[0])!.state.actors[0].alive).toBe(true)

    // Real focus loss must clear held input; key-up after return cannot replay it.
    await arena.click()
    const commandsBeforeBlur = games[0].commands.length
    await one.keyboard.down('ArrowRight')
    await one.getByRole('button', { name: 'Leave room', exact: true }).focus()
    await expect.poll(() => games[0].commands.slice(commandsBeforeBlur)
      .some((command) => command.type === 'movement' && command.direction === null)).toBe(true)
    await one.keyboard.up('ArrowRight')
    await arena.click()
    await one.keyboard.press('Space')
    await expect.poll(() => games[0].commands.filter((command) => command.type === 'manualAction').map((command) => command.actionId))
      .toEqual([1, 2])
    expect(games[0].commands.filter((command) => command.type === 'resumeRoom')).toHaveLength(1)
    expect(await invalidResume(one, 'wrongCapability')).toBe('resumeRejected')
    expect(await invalidResume(one, 'wrongRoom')).toBe('resumeRejected')
    const replacement = await takeover(one)
    expect(replacement.roomId).toBe(before.room.roomId)
    expect(replacement.seat).toBe('one')
    expect(replacement.actionFloor).toBe(2)
    expect(replacement.stableCapability).toBe(true)
    await expect(one.getByText('This seat was resumed in another tab.', { exact: true })).toBeVisible()
    await expect(one.getByRole('button', { name: 'Create room', exact: true })).toBeEnabled()
    const socketCount = games[0].sockets
    await delay(1_000)
    expect(games[0].sockets).toBe(socketCount)
    await assertPublicOnly(pages)
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    await expect.poll(async () => (await admissionBaseline(one))?.roomId).not.toBe(before.room.roomId)
    for (const [index, page] of pages.entries()) {
      expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([])
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true)
      await page.screenshot({ path: testInfo.outputPath(`recovery-${index}.png`), fullPage: true })
    }
    expect(games[0].commands.filter((command) => command.type === 'resumeRoom')).toHaveLength(4)
  } catch (error) {
    await attachPublicSummary(testInfo, games)
    throw error
  } finally {
    if (release) await release()
    await Promise.all(contexts.map((context) => context.close()))
  }
})

test('two offline tabs stop recovery after a late reserved-seat rejection', async ({ browser }, testInfo) => {
  const contexts = await Promise.all([browser.newContext(), browser.newContext()])
  const releases: (() => void)[] = []
  const games: ObservedGame[] = []
  try {
    const pages = await Promise.all(contexts.map((context) => context.newPage()))
    await Promise.all(pages.map(installTransportProbe))
    games.push(...pages.map(observeGame))
    await joinMatch(pages, games)
    for (const page of pages) releases.push(await holdSessions(page))
    await Promise.all(pages.map(disconnectTransport))
    // Let the real authoritative ten-second reservation run out. No game clock
    // or state is replaced; the eventual server rejection decides the outcome.
    await delay(11_000)
    await Promise.all(releases.splice(0).map((release) => release()))
    for (const page of pages) {
      await expect(page.getByText('This reserved seat is no longer available.', { exact: true })).toBeVisible()
      await expect(page.getByRole('button', { name: 'Create room', exact: true })).toBeEnabled()
    }
    const counts = games.map((game) => game.sockets)
    await delay(1_000)
    expect(games.map((game) => game.sockets)).toEqual(counts)
    expect(games.map((game) => game.commands.filter((command) => command.type === 'resumeRoom').length)).toEqual([1, 1])
    await assertPublicOnly(pages)
  } catch (error) {
    await attachPublicSummary(testInfo, games)
    throw error
  } finally {
    await Promise.all(releases.map((release) => release()))
    await Promise.all(contexts.map((context) => context.close()))
  }
})
